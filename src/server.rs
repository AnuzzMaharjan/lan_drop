mod discoverable;

use std::{fs, fs::File, thread};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::string::ToString;

const ROOT_PATH:&str = "D:/rust/projects/lan_drop/";

pub fn read_file_name(ip_addr: std::net::IpAddr, port:String) {
    let listener = TcpListener::bind(ip_addr.to_string() + ":" + port.as_str()).unwrap();

    // after listening, announce
    thread::spawn(|| {
        discoverable::announce(port);
    });
    
    //start tcp server
    thread::spawn(|| {
        tcp_server(listener);
    });
    
}

fn tcp_server(listener:TcpListener){
    println!("Reading file...");
    let mut len_buf = [0u8; 4];
    
    for stream in listener.incoming(){
        let mut stream = stream.unwrap();

        stream.read_exact(&mut len_buf).unwrap();
        let name_len = u32::from_be_bytes(len_buf) as usize;

        let mut name_buf = vec![0u8; name_len];
        stream.read_exact(&mut name_buf).unwrap();

        let mut filename = String::from_utf8(name_buf).unwrap();

        println!("{}", filename);

        if filename.contains("/") {
            filename = filename.split("/").last().unwrap().to_string();
        }

        read_store_file_stream(&filename,&mut stream).unwrap();
    }
}

fn read_store_file_stream(filename:&String,stream: &mut TcpStream)->std::io::Result<()>{
    if let Err(_) = fs::read_dir(ROOT_PATH.to_string() + "uploads") {
        fs::create_dir(ROOT_PATH.to_string() + "uploads")?;
    };
    let mut file = File::create(ROOT_PATH.to_string() + "uploads/" + filename)?;
    let mut buffer = [0u8; 8192];

    loop{
        let n = stream.read(&mut buffer)?;
        if n==0 {
            break;
        }
        file.write_all(&buffer[..n])?;
    }
    Ok(())
}