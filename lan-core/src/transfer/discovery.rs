use std::collections::HashMap;
use std::io::{self, Write};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::custom_types::{Control, Peer};
use crate::threadpool;

pub fn discover() {
    let mut peers: HashMap<SocketAddr, Peer> = HashMap::new();

    let (tx, rx) = mpsc::channel::<Control>();

    let pool = threadpool::get_thread_pool();

    // initialize listener thread
    let listener_handle = pool.execute(move || {
        initialize_listener("0.0.0.0:8787", tx);
    });
    // initialize writer thread
    let writer_handle = pool.execute(move || {
        initialize_writer(&mut peers, rx);
    });

    // process persistence
    match listener_handle.recv(){
        Ok(_) => {},
        Err(e)=> eprintln!("{}",e)
    };
    writer_handle.recv().unwrap();
}

fn initialize_listener(addr: &str, tx: mpsc::Sender<Control>) {
    // convert addr to string for socket binding
    let addr = addr.to_string();
    let socket = match UdpSocket::bind(&addr) {
        Ok(s) => s,
        Err(e) => {
            // if socket fails to establish,
            tx.send(Control::Stop(format!("Socket Error: {e}")))
                .unwrap();
            return;
        }
    };
    let mut packet_buf = [0u8; 2048];
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    loop {
        match socket.recv_from(&mut packet_buf) {
            Ok((pkt_size, socket_addr)) => {
                let ip = socket_addr.ip();
                let msg = String::from_utf8_lossy(&packet_buf[..pkt_size]).to_string();
                let split_msg = msg.split('|').collect::<Vec<&str>>();
                let name = split_msg[0].trim().to_string();
                let port = split_msg[2].trim()
                    .parse::<u16>()
                    .unwrap();
                let peer = Peer::new(name, ip, port);
                tx.send(Control::Data(peer)).unwrap();
            }
            Err(e) => {
                tx.send(Control::Stop(format!("UDP Error: {e}"))).unwrap();
                break;
            }
        };
    }
    tx.send(Control::Stop(format!("Listener Stopped!")))
        .unwrap();
}
fn initialize_writer(peers: &mut HashMap<SocketAddr, Peer>, rx: mpsc::Receiver<Control>) {
    while let Ok(control) = rx.recv() {
        match control {
            Control::Data(peer) => {
                peers
                    .entry(SocketAddr::new(peer.get_ip().to_owned(),peer.get_tcp_port()))
                    .and_modify(|p| p.last_seen = Instant::now())
                    .or_insert(peer);

                peers.retain(|_, p| p.last_seen.elapsed() < Duration::from_secs(4));
            }
            Control::Stop(error) => {
                println!("{}", error);
                break;
            }
        }
        // print peers
        print!("\x1b[J");
        for peer in peers.values() {
            println!("-> {} | {} | {}",peer.get_name(), peer.get_ip(), peer.get_tcp_port());
        }
        print!("\x1b[{}A", peers.len());
        // println!();
        io::stdout().flush().unwrap();
    }

}
