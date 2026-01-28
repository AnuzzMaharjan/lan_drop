use std::collections::HashMap;
use std::io::{ErrorKind, Write};
use std::net::{IpAddr, UdpSocket};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Peer {
    name: String,
    ip: IpAddr,
    tcp_port: u16,
    last_seen: Instant,
}
pub fn discover() -> Result<(), Box<dyn std::error::Error>> {
    let mut peers: HashMap<IpAddr, Peer> = HashMap::new();

    println!("Available receiver...");

    let socket = UdpSocket::bind("0.0.0.0:8787")?;

    let mut packet_buf = [0u8; 512];

    socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    loop {
        match socket.recv_from(&mut packet_buf) {
            Ok((pkt_size, socket_addr)) => {
                let ip = socket_addr.ip();
                let msg = String::from_utf8_lossy(&packet_buf[..pkt_size]).to_string();
                let split_msg = msg.split("|").collect::<Vec<&str>>();

                if !peers.contains_key(&ip) {
                    println!("-> {} | {}",ip,msg);
                }

                peers
                    .entry(ip)
                    .and_modify(|p| p.last_seen = Instant::now())
                    .or_insert(Peer {
                        name: split_msg[1].split("=").collect::<Vec<&str>>()[1].to_string(),
                        ip,
                        tcp_port: split_msg[2].split("=").collect::<Vec<&str>>()[1]
                            .parse::<u16>()
                            .unwrap(),
                        last_seen: Instant::now(),
                    });
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                    peers.iter().filter(|&(_,p)| p.last_seen.elapsed() > Duration::from_secs(10)).for_each(|(ip,_)| println!("Expired: {}",ip));

                    peers.retain(|_, p| p.last_seen.elapsed() <= Duration::from_secs(10));
                }else{
                    return Err(e.into());
                }
            }
        };
        std::io::stdout().flush().unwrap();
    }
}
