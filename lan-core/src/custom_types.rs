use std::io::ErrorKind;
use std::net::IpAddr;
use std::time::Instant;

#[derive(Debug)]
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
    filename: String,
    ip: IpAddr,
    tcp_port: u16
}

impl SendFileData {
    pub fn new(filename: String, ip: IpAddr, tcp_port: u16) -> SendFileData {
        SendFileData {
            filename,
            ip,
            tcp_port
        }
    }
    pub fn get_filename(&self) -> &String {
        &self.filename
    }
    pub fn get_ip(&self) -> &IpAddr {
        &self.ip
    }
    pub fn get_tcp_port(&self) -> u16 {
        self.tcp_port
    }
}