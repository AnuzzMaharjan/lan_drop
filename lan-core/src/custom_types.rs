use std::io::ErrorKind;
use std::net::IpAddr;
use std::time::Instant;
use bincode::{Decode, Encode};

#[derive(Clone,Debug)]
pub struct Peer {
    name: String,
    ip: IpAddr,
    tcp_port: u16,
    pub last_seen: Instant,
}
impl Peer {
    pub fn new(name: String, ip: IpAddr, tcp_port: u16) -> Peer {
        Peer {
            name,
            ip,
            tcp_port,
            last_seen: Instant::now(),
        }
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_ip(&self) -> &IpAddr {
        &self.ip
    }
    pub fn get_tcp_port(&self) -> u16 {
        self.tcp_port
    }
}

#[derive(Debug)]
pub enum Control {
    Data(Peer),
    Stop(String),
}

pub struct ErrorMessage {
    message : String,
    error: ErrorKind
}

impl ErrorMessage {
    pub fn new(message: String, error: ErrorKind) -> ErrorMessage {
        ErrorMessage {
            message,
            error
        }
    }
    pub fn get_message(&self) -> &String {
        &self.message
    }
    pub fn get_error(&self) -> &ErrorKind {
        &self.error
    }
}

pub struct SendFileData{
    filepath: String,
    ip: IpAddr,
    tcp_port: u16
}

impl SendFileData {
    pub fn new(filepath: String, ip: IpAddr, tcp_port: u16) -> SendFileData {
        SendFileData {
            filepath,
            ip,
            tcp_port
        }
    }
    pub fn get_filepath(&self) -> &String {
        &self.filepath
    }
    pub fn get_ip(&self) -> &IpAddr {
        &self.ip
    }
    pub fn get_tcp_port(&self) -> u16 {
        self.tcp_port
    }
}

#[derive(Encode,Decode,Debug)]
pub struct FileMetaData {
    pub filename: String,
    pub file_size: u64,
    pub merkle_root: [u8;32],
    pub chunk_size: u32
}

impl Drop for FileMetaData {
    fn drop(&mut self) {
        println!("FileMetaData dropped: {}", self.filename);
    }
}
