use std::net::{IpAddr, UdpSocket};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

#[derive(Debug)]
struct Peer  {
    name: String,
    ip: IpAddr,
    tcp_port: u16,
    last_seen: Instant,
}
impl Peer {
    fn new(name: String, ip: IpAddr, tcp_port: u16) -> Peer {
        Peer {
            name,
            ip,
            tcp_port,
            last_seen: Instant::now(),
        }
    }
}

enum Control{
    Data(Peer),
    Stop(String)
}

pub fn discover() {
    let peers: Vec<Peer> = Vec::new();

    let (tx,rx) = mpsc::channel::<Control>();

    let listener_handle = thread::spawn(move || {
        let socket = match UdpSocket::bind("0.0.0.0:8787") {
            Ok(s) => s,
            Err(e) => {
                tx.send(Control::Stop(format!("Socket Error: {e}"))).unwrap();
                return;
            }
        };
        let mut packet_buf = [0u8; 2048];
        loop {
            match socket.recv_from(&mut packet_buf) {
                Ok((pkt_size, socket_addr)) => {
                    let ip = socket_addr.ip();
                    let msg = String::from_utf8_lossy(&packet_buf[..pkt_size]).to_string();
                    let split_msg =  msg.split('|').collect::<Vec<&str>>();
                    let name = split_msg[1].split("=").collect::<Vec<&str>>()[1].to_string();
                    let port = split_msg[2].split("=").collect::<Vec<&str>>()[1]
                        .parse::<u16>()
                        .unwrap();
                    let peer = Peer::new(name,ip,port);
                    tx.send(Control::Data(peer)).unwrap();
                }
                Err(e) => {
                    tx.send(Control::Stop(format!("UDP Error: {e}"))).unwrap();
                    break;
                }
            };
        }
    });
    let writer_handle = thread::spawn(move || {
        while let Ok(control) = rx.recv() {
            match control {
                Control::Data(peer) => {
                    println!("{:#?}", peer);
                },
                Control::Stop(error) => {
                    println!("{}", error);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use super::*;
    #[test]
   fn test_writer_updates_peers(){
        use std::sync::mpsc;
        use std::thread;

        let (tx,rx) = mpsc::channel::<Control>();

        let writer_handle = thread::spawn(move || {
            let mut peers = Vec::new();
            while let Ok(msg) = rx.recv() {
                match msg{
                    Control:: Data(peer) => {
                        peers.push(peer)
                    },
                    Control::Stop(_) => break,
                }
            }
            peers
        });

        tx.send(Control::Data(
            Peer::new(
                "test".to_string(),
                IpAddr::V4(Ipv4Addr::new(127,0,0,1)),
                8787)
        )).unwrap();
        tx.send(Control::Data(
            Peer::new(
                "tes2".to_string(),
                IpAddr::V4(Ipv4Addr::new(127,0,0,2)),
                9876)
        )).unwrap();
        tx.send(Control::Stop(String::new())).unwrap();

        let peers = writer_handle.join().unwrap();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].name, "test");
        assert_eq!(peers[0].ip, "127.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(peers[0].tcp_port, 8787);
        assert_eq!(peers[1].name, "tes2");
        assert_eq!(peers[1].ip, "127.0.0.2".parse::<IpAddr>().unwrap());
        assert_eq!(peers[1].tcp_port, 9876);
    }
}