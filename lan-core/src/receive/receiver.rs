use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::custom_types::ErrorMessage;

pub fn receiver(listener: &TcpListener, file_save_path: &str, filename:Option<&str>) {
    println!("Initializing receiver...");

    let mut len_buf = [0u8; 4];

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        stream.read_exact(&mut len_buf).unwrap();
        let name_len = u32::from_be_bytes(len_buf) as usize;

        let mut name_buf = vec![0u8; name_len];
        stream.read_exact(&mut name_buf).unwrap();

        let mut filepath = String::from_utf8(name_buf).unwrap();
        println!("Name: {}", filepath);

        filepath = check_path(&filepath);

        let final_filename = match filename {
            Some(name) => name.to_string(), // explicit override
            None => Path::new(&filepath)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(), // fallback from filepath
        };

        match read_store_file_stream(&final_filename,file_save_path,&mut stream) {
            Err(err) => {
            eprintln!("{}", err.get_message());
            return;
            },
            Ok(msg) => println!("{}", msg),
        }
    }
}

fn read_store_file_stream(filename:&str, file_save_path:&str,stream: &mut TcpStream) -> Result<String,ErrorMessage> {
    if !fs::exists(file_save_path).unwrap() {
        return Err(ErrorMessage::new(format!("File not found: {}", file_save_path), ErrorKind::NotFound));
    };

    if let Err(e) = fs::create_dir_all(PathBuf::from(file_save_path).join("uploads")) {
        return Err(ErrorMessage::new(format!("Cannot create directory: {}", file_save_path), e.kind()));
    };
    let mut file = match File::create(PathBuf::from(file_save_path).join("uploads").join(filename)) {
        Ok(f) => f,
        Err(e) => return Err(ErrorMessage::new(format!("file {} creation failed!\nPermission Denied: {}",filename,file_save_path),e.kind()))
    };
    let mut buffer = [0u8; 8192];
    loop{
        let n = match stream.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => return Err(ErrorMessage::new(format!("Connection unexpectedly closed: {}",e),e.kind()))
        };
        if n==0 {
            break;
        }
        if let Err(e) =  file.write_all(&buffer[..n]) {
            return Err(ErrorMessage::new(format!("Connection unexpectedly closed: {}",e),e.kind()))
        };
    }
    Ok(format!("File saved: {}/{}", file_save_path, filename))
}
#[cfg(target_os = "windows")]
fn check_path(filepath: &str) -> String {
    if filepath.contains("\\") {
        filepath.replace("\\", "/")
    } else {
        filepath.to_string()
    }
}
