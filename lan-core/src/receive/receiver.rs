use crate::custom_types::ErrorMessage;
use rand;
use rand::RngExt;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{PathBuf};

pub fn receiver(
    listener: &TcpListener,
    file_save_path: String,
    filename: Option<String>
) {
    println!("Initializing receiver...");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(_) => {
                eprintln!("Tcp stream error!");
                continue;
            }
        };

        // consume the file path sent by the sender 
        let filepath = get_file_path(&mut stream);
        
        // if the filename is specified, use it, otherwise try to get it from the filepath, if that fails, create a random name
        let filename = match filename.clone() {
            Some(name) => {
                println!("Provided filename: {}", name);
                name
            }
            None => {
                // fallback if there is no filename explicitly specified
                get_file_name(&filepath[..])
            }
        };
        
        match read_store_file_stream(&filename[..], &file_save_path, &mut stream) {
            Err(err) => {
                eprintln!("{}", err.get_message());
                return;
            }
            Ok(msg) => println!("{}", msg),
        }
    }
}

fn filepath_contains_filename(filepath: &str) -> bool {
    let filename_regex = Regex::new(r"(?i)^[\w\s,:\\/-]+\.[A-Za-z]+$").unwrap();
    filename_regex.is_match(filepath)
}

fn get_file_name(filepath: &str) -> String {
    // checking for filename
    if filepath_contains_filename(filepath) {
        // split the filepath to get the name
        let filename = filepath.split("/").last().unwrap().to_string();
        filename
    } else {
        create_random_name()
    }
}

// final filename if someones stupid enough
fn create_random_name() -> String {
    let mut rng = rand::rng();
    let random_filename: String = (0..10)
        .map(|_| rng.sample(rand::distr::Alphanumeric) as char)
        .collect();
    random_filename + ".txt"
}

fn get_file_path(stream: &mut TcpStream) -> String {
    let mut len_buf = [0u8; 4];
    // first: length of the file path string
    stream.read_exact(&mut len_buf).unwrap();
    let path_len = u32::from_be_bytes(len_buf) as usize;

    let mut path_buf = vec![0u8; path_len];
    // second: filepath
    stream.read_exact(&mut path_buf).unwrap();
    let mut filepath = String::from_utf8(path_buf).unwrap();

    println!("File Path: {}", filepath);

    // normalize the file path
    filepath = check_path(filepath.as_str());
    filepath
}

fn get_directory_chain(filepath:&str)->PathBuf{
    let filepath = check_path(filepath);
    if filepath_contains_filename(&filepath) {
        let mut parts = filepath.split("/").collect::<Vec<&str>>();
        let _ = parts.pop();
        let chain = PathBuf::from(parts.join("/"));
        chain
    }else{
        PathBuf::from(filepath)
    }
}

fn create_directory(file_save_path:&str) -> Result<PathBuf,ErrorMessage> {
    let directory_chain = get_directory_chain(file_save_path);
    if let Err(e) = fs::create_dir_all(&directory_chain) {
        Err(ErrorMessage::new(
            format!("Cannot create directory: {}", file_save_path),
            e.kind(),
        ))
    }else{
        Ok(directory_chain)
    }
}

fn read_store_file_stream(
    filename: &str,
    file_save_path: &str,
    stream: &mut TcpStream,
) -> Result<String, ErrorMessage> {
    let directory = create_directory(file_save_path)?;
    let path_to_file = directory.join(filename);
    let mut file = match File::create(&path_to_file)
    {
        Ok(f) => f,
        Err(e) => {
            return Err(ErrorMessage::new(
                format!(
                    "file creation failed!\nPermission Denied At: {}",
                    path_to_file.to_str().unwrap()
                ),
                e.kind(),
            ));
        }
    };
    let mut buffer = [0u8; 8192];
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
        if let Err(e) = file.write_all(&buffer[..n]) {
            return Err(ErrorMessage::new(
                format!("Connection unexpectedly closed: {}", e),
                e.kind(),
            ));
        };
    }
    Ok(format!("File saved: {} / {}", file_save_path, filename))
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
    fn test_filepath_contains_filename(){
        let filepath1 = "D:/test/file.txt";
        let filepath2 = "D:\\users\\test";
        let filepath3 = "D:\\users\\test test\\file.txt";
        assert_eq!(filepath_contains_filename(filepath1), true);
        assert_eq!(filepath_contains_filename(filepath2), false);
        assert_eq!(filepath_contains_filename(filepath3), true);
    }

    #[test]
    fn test_get_file_name(){
        let filepath1 = "D:/test/file.txt";
        let filepath2 = "D:\\users\\test";
        assert_eq!(get_file_name(filepath1), "file.txt");
        assert_eq!(get_file_name(filepath2).ends_with(".txt"), true);
    }
    
    #[test]
    fn test_create_random_name(){
        let name1 = create_random_name();
        let name2 = create_random_name();
        assert_eq!(name1.len(), 14);
        assert_eq!(name2.len(), 14);
        assert_eq!(name1.ends_with(".txt"), true);
        assert_eq!(name2.ends_with(".txt"), true);
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_get_directory_chain(){
        let filepath1 = "D:/test/file.txt";
        let filepath2 = "D:\\users\\test";
        let chain1 = get_directory_chain(filepath1);
        let chain2 = get_directory_chain(filepath2);
        assert_eq!(chain1.to_str().unwrap(), "D:/test");
        assert_eq!(chain2.to_str().unwrap(), "D:/users/test");
    }
}
