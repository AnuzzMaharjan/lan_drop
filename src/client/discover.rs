use std::collections::HashMap;
use std::io::Write;
use std::net::{IpAddr, UdpSocket};
use std::time::Instant;

#[derive(Clone)]
struct Peer {
    name: String,
    ip: IpAddr,
    tcp_port: u16,
    last_seen: u64,
}
pub fn discover(){
    let mut peers:HashMap<IpAddr,Peer> = HashMap::new();

    println!("Available receiver...");

    let socket = UdpSocket::bind("0.0.0.0:8787").unwrap();

    let mut packet_buf = [0u8; 512];
    let timer = Instant::now();
    loop {
        let (pkt_size,socket_addr) = socket.recv_from(&mut packet_buf).unwrap();

        let ip = socket_addr.ip();
        let msg = String::from_utf8_lossy(&packet_buf[..pkt_size]).to_string();
        print!("-> {} \t| active\nReceived packet: {} bytes from {}\n",msg,pkt_size, socket_addr);

        let msg = msg.split("|").collect::<Vec<&str>>();

        peers.entry(ip).and_modify(|p| p.last_seen = timer.elapsed().as_secs()).or_insert(Peer{
            name: msg[1].split("=").collect::<Vec<&str>>()[1].to_string(),
            ip,
            tcp_port: msg[2].split("=").collect::<Vec<&str>>()[1].parse::<u16>().unwrap(),
            last_seen: timer.elapsed().as_secs(),
        });

        for peer in peers.clone().values() {
            if (timer.elapsed().as_secs() - peer.last_seen) > (peer.last_seen + 10)  {
                peers.remove(&peer.ip);
            }
        }


        std::io::stdout().flush().unwrap();
    }
}