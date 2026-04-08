use std::{fs::File, io::{BufReader, Read, Write}, net::TcpStream, time::Instant};

use crate::custom_types::{ErrorMessage, SendFileData};


pub fn send_file(to_be_sent_data:&SendFileData)->Result<(), ErrorMessage>{
    let mut stream = match TcpStream::connect((to_be_sent_data.get_ip().to_string().as_str(), to_be_sent_data.get_tcp_port())) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to receiver!");
            return Err(ErrorMessage::new("Failed to connect to receiver!".to_string(), e.kind()));
        }
    };

    let name_bytes = to_be_sent_data.get_filename().as_bytes();
    let name_length = name_bytes.len() as u32;

    if let Err(e) = stream.write_all(&name_length.to_be_bytes()) {
        return Err(ErrorMessage::new("Failed to send file name length!".to_string(), e.kind()));
    };

    if let Err(e) = stream.write_all(name_bytes) {
        return Err(ErrorMessage::new("Failed to send file name!".to_string(), e.kind()));
    };

    if let Err(e) = stream.flush() {
        return Err(ErrorMessage::new("Failed to flush stream!".to_string(), e.kind()));
    };

    if let Err(e) = stream_file(&to_be_sent_data.get_filename(), &mut stream) {
        return Err(ErrorMessage::new("Failed to send file data!".to_string(), e.kind()));
    };

    Ok(())
}

fn stream_file(file_path: &str, stream: &mut TcpStream) -> std::io::Result<()>{
    let file = File::open(file_path)?;
    let filesize = file.metadata()?.len();
    
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    let mut total_bytes = 0;
    let start = Instant::now();

    loop {       
        let n = reader.read(&mut buffer)?;
        
        if n==0{
            break;
        }

        stream.write_all(&buffer[..n])?;

        total_bytes += n;
        let elapsed = start.elapsed().as_secs_f64();
        let mbps = (total_bytes as f64 / 1024.0 / 1024.0) / elapsed;
        let percent = (total_bytes as f64 / filesize as f64) * 100.0;
        println!("\rSent: {} B | {:.2}% ({:.2} MB/s)", total_bytes, percent, mbps);
        std::io::stdout().flush()?;
    }
    
    Ok(())
}