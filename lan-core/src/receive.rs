mod advertise;
mod receiver;

use std::net::TcpListener;

use lan_engine::ThreadPool;

use crate::{custom_types::Control, utils::get_local_ip};

pub fn init_receiver(port: String, file_save_path: Option<String>, filename: Option<String>) {
        let local_ip = match get_local_ip() {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Failed to get local IP address: {}", e);
            return;
        }
    };
    println!("Starting receiver on {}:{}", local_ip, port);


    let file_save_path = match file_save_path {
        Some(path) => path,
        None => {
            eprintln!("File save path not specified! Usage: cargo run --bin rec -p <port> <file_save_path> [filename]");
            std::process::exit(1);
        }
    };
    
    let listener = TcpListener::bind(format!("{}:{}", local_ip, port)).expect("Failed to bind to address");

    // message channel
    let (sender,receiver) = std::sync::mpsc::channel::<Control>();

    let pool = ThreadPool::new(2);

    pool.execute(move || {
        receiver::receiver(&listener, file_save_path, filename, sender);
    });

    pool.execute(move || {
        advertise::advertise(port, receiver);
    });
}
