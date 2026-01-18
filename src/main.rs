mod server;
mod client;

use std::fs::File;
use std::net::{IpAddr, UdpSocket};
use std::process::exit;
use std::str::FromStr;

fn get_local_ip() -> std::net::IpAddr {
    let socket = UdpSocket::bind("0.0.0.0:7878").unwrap();
    socket.connect("8.8.8.8:80").unwrap();
    let local_addr = socket.local_addr().unwrap();
    println!("Listening on {}", local_addr.ip());
    local_addr.ip()
}

fn send(ip_port:&str, file_path:&str) {
    let (ip,port) = match ip_port.split_once(":") {
        Some((ip_str,port_str)) => {
            if IpAddr::from_str(ip_str).is_err() {
                eprintln!("Invalid IP address: {}", ip_str);
                exit(1);
            }
            let port:u16 = match port_str.parse() {
                Ok(p) => p,
                Err(_) => {eprintln!("Invalid port number: {}", port_str); exit(1);}
            };
            (ip_str.to_string(), port.to_string())
        },
        None => {
            eprintln!("Invalid format, expected <ip:port>");
            exit(1);
        }
    };

    if let Err(e) = File::open(file_path) {
        eprintln!("Failed to open {}: {}",file_path, e);
        exit(1);
    }
    
    client::send_file_name(file_path , &ip, &port);
}

fn serve(port:&str){
    let port = match port.parse::<u16>() {
        Ok(p) => p,
        Err(_) => { eprintln!("Invalid port number!");exit(1) },
    };

    println!("Starting server on {}", port);
    let local_ip = get_local_ip();
    server::read_file_name(local_ip, port);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args[1].as_str() {
        "send" => send(&args[2],&args[3]),
        "serve" => serve(&args[2]),
        _ => {
            eprintln!(
                "Usage:
              lan_drop serve <port>
              lan_drop send <ip:port> <file>"
            );
            exit(1);
        }
    }
}
