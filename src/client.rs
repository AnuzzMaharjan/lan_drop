pub mod discover;

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::{TcpStream};
use std::process::exit;
use std::time::Instant;

pub fn stream_file(file_path: &str, stream: &mut TcpStream) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let filesize = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    let mut total_bytes = 0;
    let start = Instant::now();

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        stream.write_all(&buffer[..n])?;

        total_bytes += n;
        let elapsed = start.elapsed().as_secs_f64();
        let mbps = (total_bytes as f64 / 1024.0 / 1024.0) / elapsed;
        let percent = (total_bytes as f64 / filesize as f64) * 100.0;

        print!("\rSent: {} B | {:.1} | {:.2} MB/s", total_bytes, percent, mbps);
        std::io::stdout().flush()?;
    }
    println!();
    Ok(())
}
pub fn send_file_name(filename: &str, ip: &str, port: &str) {
    println!("Send file name: {}", filename);
    let mut stream = match TcpStream::connect(ip.to_string()+":"+port) {
        Ok(s) => s,
        Err(e) => {eprintln!("Failed to connect: {}",e);exit(1);}
    };

    let name_bytes = filename.as_bytes();
    let len = name_bytes.len() as u32;
    stream.write_all(&len.to_be_bytes()).unwrap();

    stream.write_all(name_bytes).unwrap();


    stream_file(filename, &mut stream).unwrap();

    stream.flush().unwrap();
}
