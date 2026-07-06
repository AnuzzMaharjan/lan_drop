use bincode::{self, config};
use std::{
    fs::File, io::{BufReader, Read, Write}, net::TcpStream, sync::{Arc, Mutex}, thread, time::{Duration, Instant}
};

use crate::{
    custom_types::{ErrorMessage, FileMetaData, SendFileData}, threadpool, utils::{display_progress, filepath_contains_filename, standardize_path}
};

pub fn send_file(to_be_sent_data: &SendFileData) -> Result<(), ErrorMessage> {
    let unix_path = standardize_path(to_be_sent_data.get_filepath());
    
    if !filepath_contains_filename(&unix_path) {
        return Err(ErrorMessage::new(
            "Invalid filepath!".to_string(),
            std::io::ErrorKind::InvalidInput,
        ));
    }
    if File::open(&unix_path).is_err() {
        return Err(ErrorMessage::new(
            "File not found!".to_string(),
            std::io::ErrorKind::NotFound,
        ));
    }

    // wrap the unix path in Arc and Mutex to share between threads after standardization and validation
    let unix_path = Arc::new(Mutex::new(unix_path));

    // let pool = threadpool::get_thread_pool();
    // let thread_safe_unix_path = Arc::clone(&unix_path);


    let mut stream = match TcpStream::connect((
        to_be_sent_data.get_ip().to_string().as_str(),
        to_be_sent_data.get_tcp_port(),
    )) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to receiver!");
            return Err(ErrorMessage::new(
                "Failed to connect to receiver!".to_string(),
                e.kind(),
            ));
        }
    };

    println!(
        "Connected to receiver at {}:{}",
        to_be_sent_data.get_ip(),
        to_be_sent_data.get_tcp_port()
    );

    let (filename, file_size) = {
        let path_guard = &unix_path.lock().unwrap();
        let filename = path_guard.split("/").last().unwrap().to_string();
        let file_size = File::open(path_guard.as_str()).unwrap().metadata().unwrap().len();
        (filename, file_size)
    }; 

    let metadata = FileMetaData {
        filename: filename,
        file_size: file_size,
        chunk_size: {
            if file_size < 256*1024 {
                file_size as u32
            } else {
             256*1024 as u32
            }
        }
    };

    let serialized_metadata: Vec<u8> =
        bincode::encode_to_vec(&metadata, config::standard()).unwrap();
    let metadata_size = serialized_metadata.len() as u32;

    if let Err(e) = stream.write_all(&metadata_size.to_be_bytes()) {
        return Err(ErrorMessage::new(
            "Failed to send metadata size!".to_string(),
            e.kind(),
        ));
    }

    if let Err(e) = stream.write_all(&serialized_metadata) {
        return Err(ErrorMessage::new(
            "Failed to send metadata!".to_string(),
            e.kind(),
        ));
    };

    // flush the stream before sending the file
    if let Err(e) = stream.flush() {
        return Err(ErrorMessage::new(
            "Failed to flush stream!".to_string(),
            e.kind(),
        ));
    };

    if let Err(e) = stream_file(&to_be_sent_data.get_filepath(), &mut stream, &metadata) {
        return Err(ErrorMessage::new(
            "Failed to send file data!".to_string(),
            e.kind(),
        ));
    };

    Ok(())
}

fn stream_file(file_path: &str, stream: &mut TcpStream, metadata: &FileMetaData) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let filesize = file.metadata()?.len();

    let mut reader = BufReader::new(file);
    let size_of_chunk = metadata.chunk_size as usize;
    let mut buffer = vec![0u8; size_of_chunk];

    let total_bytes = 0;
    let start = Instant::now();

    let mut chunk_hashes = [0u8;32];

    loop {
        let n = reader.read(&mut buffer)?;
        println!("Read {:?} bytes from file...", n);
        if n == 0 {
            break;
        }

        chunk_hashes = lan_engine::combine_hashes(&chunk_hashes, &lan_engine::hash_chunk(&buffer[..n]));

        stream.write_all(&buffer[..n])?;

        display_progress("Sent", total_bytes, n, filesize.try_into().unwrap(), &start);
    }

    println!("File sent successfully!");
    // small delay to ensure the receiver has received all the file data before sending the chunk hashes
    thread::sleep(Duration::from_millis(200));
    stream.write_all(&chunk_hashes)?;

    Ok(())
}

