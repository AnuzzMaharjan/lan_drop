use crate::custom_types::{Control, ErrorMessage, FileMetaData};
use crate::utils::{display_progress, filepath_contains_filename};
use bincode::config;
use std::fs::File;
use std::fs::{self};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Instant;

pub fn receiver(
    listener: &TcpListener,
    file_save_path: String,
    filename: Option<String>,
    sender: std::sync::mpsc::Sender<Control>,
) {
    println!("Initializing receiver...");

    // early check if the save path has a filename
    if filepath_contains_filename(&file_save_path) {
        eprintln!("Invalid save path! Save path should be a directory, not a file.");
        std::process::exit(1);
    }

    let mut stream = match listener.accept() {
        Ok((str_res, addr_res)) => {
            println!("Connection established from: {}", addr_res.ip());

            str_res
        }
        Err(e) => {
            eprintln!("Failed to accept connection: {}", e);
            return;
        }
    };

    // meta data size worth 4 bytes
    let mut metadata_len_buf = [0u8; 4];
    stream.read_exact(&mut metadata_len_buf).unwrap();

    // construct metadata from metadata size
    let mut metadata_buf = vec![0u8; u32::from_be_bytes(metadata_len_buf) as usize];
    stream.read_exact(&mut metadata_buf).unwrap();
    let (metadata, metadata_size): (FileMetaData, usize) =
        match bincode::decode_from_slice(&metadata_buf, config::standard()) {
            Ok((meta, size)) => (meta, size),
            Err(e) => {
                eprintln!("Failed to decode metadata: {}", e);
                std::process::exit(1);
            }
        };
    println!(
        "Received metadata length: {} bytes",
        metadata_size
    );

    // sanity check for metadata size
    if metadata_size != metadata_buf.len() {
        eprintln!("Corrupted metadata received! Terminating...");
        std::process::exit(1);
    }

    // if the filename is specified, use it, otherwise get from the metadata
    let filename = match filename {
        Some(name) => {
            println!("Provided filename: {}", name);
            name
        }
        None => {
            // fallback if there is no filename explicitly specified
            metadata.filename.clone()
        }
    };

    match read_store_file_stream(&filename[..], &file_save_path, &metadata, &mut stream) {
        Err(err) => {
            eprintln!("{}", err.get_message());
            eprintln!("Error Kind: {:?}", err.get_error());
            return;
        }
        Ok(msg) => println!("{}", msg),
    }

    sender
        .send(Control::Stop(
            "Process complete! Stopping advertiser...".to_string(),
        ))
        .unwrap();
}

fn get_directory_chain(filepath: &str) -> PathBuf {
    let filepath = check_path(filepath);
    if filepath_contains_filename(&filepath) {
        let mut parts = filepath.split("/").collect::<Vec<&str>>();
        let _ = parts.pop();
        let chain = PathBuf::from(parts.join("/"));
        chain
    } else {
        PathBuf::from(filepath)
    }
}

fn create_directory(file_save_path: &str) -> Result<PathBuf, ErrorMessage> {
    let directory_chain = get_directory_chain(file_save_path);
    if let Err(e) = fs::create_dir_all(&directory_chain) {
        Err(ErrorMessage::new(
            format!("Cannot create directory: {}", file_save_path),
            e.kind(),
        ))
    } else {
        Ok(directory_chain)
    }
}

fn read_store_file_stream(
    filename: &str,
    file_save_path: &str,
    metadata: &FileMetaData,
    stream: &mut TcpStream,
) -> Result<String, ErrorMessage> {
    // create a temporary file in the temporary directory
    let filename_prefix = PathBuf::from(filename)
        .file_prefix()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let temp_file_path = PathBuf::from(file_save_path).join(format!("{}.tmp", filename_prefix));
    let mut temp_file = match file_create(temp_file_path.to_str().unwrap()) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    
    let total_bytes = 0;
    let mut chunk_hashes = [0u8; 32];
    let start = Instant::now();
    // buffer as per the chunk size sent from the sender
    let mut buffer = vec![0u8; metadata.chunk_size as usize];
    loop {
        let n = match stream.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                return Err(ErrorMessage::new(
                    format!("Connection unexpectedly closed: {}", e),
                    e.kind(),
                ));
            }
        };
        if n == 0 {
            break;
        }

        chunk_hashes = lan_engine::combine_hashes(&chunk_hashes, &lan_engine::hash_chunk(&buffer[..n]));

        display_progress(
            "Received",
            total_bytes,
            n,
            metadata.file_size.try_into().unwrap(),
            &start,
        );

        if let Err(e) = temp_file.write_all(&buffer[..n]) {
            return Err(ErrorMessage::new(
                format!("Connection unexpectedly closed: {}", e),
                e.kind(),
            ));
        };
    }

    // read the chunk hashes sent from the sender
    let mut chunk_hash_buffer = [0u8; 32];
    let received_chunk_hashes = match stream.read(&mut chunk_hash_buffer) {
        Ok(_) => chunk_hashes,
        Err(e) => {
            return Err(ErrorMessage::new(
                format!("Failed to read chunk hashes: {}", e),
                e.kind(),
            ));
        }
    };


    // file operations after temp file is stored
    file_operations(&temp_file_path, filename, file_save_path, &chunk_hashes, &received_chunk_hashes)?;

    Ok(format!("File saved: {} / {}", file_save_path, filename))
}

fn file_operations(
    temp_file_path: &PathBuf,
    filename: &str,
    file_save_path: &str,
    chunk_hashes: &[u8; 32],
    received_chunk_hashes: &[u8; 32],
) -> Result<(), ErrorMessage> {

    // corrupted file if the hashes does not match
    if received_chunk_hashes != chunk_hashes {
        // cleanup the temp file and directory if the file is corrupted
        cleanup_temp_file(&temp_file_path);

        return Err(ErrorMessage::new(
            "File corrupted!".to_string(),
            std::io::ErrorKind::InvalidData,
        ));
    }
    // actual save path for the file after transfer is complete
    let save_directory = create_directory(file_save_path)?;
    let path_to_file = save_directory.join(filename);

    // attempt to move the file
    if let Err(e) = fs::rename(&temp_file_path, &path_to_file) {
        // if moving the file fails, attempt to copy and delete the temp file
        if let Err(e) = fs::copy(&temp_file_path, &path_to_file) {
            return Err(ErrorMessage::new(
                format!("Failed to save file: {}", e),
                e.kind(),
            ));
        }
        // cleanup if the file is partially moved
        cleanup_temp_file(&temp_file_path);

        return Err(ErrorMessage::new(
            format!("Failed to save file: {}", e),
            e.kind(),
        ));
    } else {
        // cleanup if the file is successfully moved
        cleanup_temp_file(&temp_file_path);
        println!(
            "File saved successfully at: {}",
            path_to_file.to_str().unwrap()
        );
    }
    Ok(())
}

fn file_create(filepath: &str) -> Result<File, ErrorMessage> {
    match File::create(filepath) {
        Ok(f) => Ok(f),
        Err(e) => Err(ErrorMessage::new(
            format!("file creation failed!\nPermission Denied At: {}", filepath),
            e.kind(),
        )),
    }
}

fn cleanup_temp_file(temp_path: &PathBuf) {
    // attempt to remove the temp file, if it exists
    if temp_path.exists() {
        let _ = fs::remove_file(temp_path);
    }
}

#[cfg(target_os = "windows")]
fn check_path(filepath: &str) -> String {
    if filepath.contains("\\") {
        filepath.replace("\\", "/")
    } else {
        filepath.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_path_windows_style() {
        let input = "c:\\users\\test\\file.txt";
        let output = check_path(input);
        assert_eq!(output, "c:/users/test/file.txt");
    }

    #[test]
    fn test_filepath_contains_filename() {
        let filepath1 = "D:/test/file.txt";
        let filepath2 = "D:\\users\\test";
        let filepath3 = "D:\\users\\test test\\file.txt";
        assert_eq!(filepath_contains_filename(filepath1), true);
        assert_eq!(filepath_contains_filename(filepath2), false);
        assert_eq!(filepath_contains_filename(filepath3), true);
    }
    #[test]
    fn test_get_directory_chain() {
        let filepath1 = "D:/test/file.txt";
        let filepath2 = "D:\\users\\test";
        let chain1 = get_directory_chain(filepath1);
        let chain2 = get_directory_chain(filepath2);
        assert_eq!(chain1.to_str().unwrap(), "D:/test");
        assert_eq!(chain2.to_str().unwrap(), "D:/users/test");
    }
}
