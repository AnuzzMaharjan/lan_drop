use std::collections::HashMap;
use std::net::{IpAddr, UdpSocket};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::custom_types::{Control,Peer};

/// Starts the peer discovery system.
///
/// This function sets up the shared state and communication channels needed
/// for peer discovery:
/// - Initializes a thread-safe peer map (`Arc<Mutex<HashMap<IpAddr, Peer>>>`)
///   to store discovered peers.
/// - Creates an `mpsc` channel for passing `Control` messages between threads.
/// - Spawns:
///   - A **listener thread** that binds to UDP port, receives packets,
///     parses them into `Peer` objects, and sends them as `Control::Data`.
///   - A **writer thread** that consumes `Control` messages, updates the peer
///     map, and prunes peers that have not been seen in the last 10 seconds.
///
/// The function returns immediately after spawning the threads; the threads
/// continue running in the background. The returned `JoinHandle`s are currently
/// unused, but could be joined later to wait for thread completion.

pub fn discover() {
    let peers: Arc<Mutex<HashMap<IpAddr, Peer>>> = Arc::new(Mutex::new(HashMap::new()));

    let (tx, rx) = mpsc::channel::<Control>();
    let (ack_tx, ack_rx) = mpsc::channel::<()>();
    let (ack_listener_tx, ack_listener_rx) = mpsc::channel::<()>();
    let (_, shutdown_rx) = mpsc::channel::<()>();

    // initializes listener thread
    let _listener_handle = initialize_listener("0.0.0.0:8787",tx,ack_listener_tx,shutdown_rx);
    ack_rx.recv().unwrap();
    // initializes writer thread
    let _writer_handle = initialize_writer(peers.clone(), rx, ack_tx);
    ack_listener_rx.recv().unwrap();
}
fn initialize_listener(
    addr: &str,
    tx: mpsc::Sender<Control>,
    ack_tx: mpsc::Sender<()>,
    shutdown_rx: mpsc::Receiver<()>,
) -> thread::JoinHandle<()> {
    let addr = addr.to_string();
    thread::spawn(move || {
        let socket = match UdpSocket::bind(&addr) {
            Ok(s) => {
                // if socket established, send the acknowledgement
                ack_tx.send(()).unwrap();
                s
            }
            Err(e) => {
                // if socket fails to establish,
                tx.send(Control::Stop(format!("Socket Error: {e}")))
                    .unwrap();
                return;
            }
        };
        let mut packet_buf = [0u8; 2048];
        socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        loop {
            // if shutdown signal received, signal to the writer thread to stop execution
            if shutdown_rx.try_recv().is_ok() {
                tx.send(Control::Stop(String::from("Stop signal received!"))).unwrap();
                break;
            }
            match socket.recv_from(&mut packet_buf) {
                Ok((pkt_size, socket_addr)) => {
                    let ip = socket_addr.ip();
                    let msg = String::from_utf8_lossy(&packet_buf[..pkt_size]).to_string();
                    let split_msg = msg.split('|').collect::<Vec<&str>>();
                    let name = split_msg[1].split("=").collect::<Vec<&str>>()[1].to_string();
                    let port = split_msg[2].split("=").collect::<Vec<&str>>()[1]
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
    })
}
fn initialize_writer(
    peers: Arc<Mutex<HashMap<IpAddr, Peer>>>,
    rx: mpsc::Receiver<Control>,
    ack_tx: mpsc::Sender<()>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(control) = rx.recv() {
            match control {
                Control::Data(peer) => {
                    {
                        peers
                            .lock()
                            .unwrap()
                            .entry(*peer.get_ip())
                            .and_modify(|p| p.last_seen = Instant::now())
                            .or_insert(Peer::new(peer.get_name().clone(), peer.get_ip().clone(), peer.get_tcp_port()));
                    }
                    peers
                        .lock()
                        .unwrap()
                        .retain(|_, p| p.last_seen.elapsed() < Duration::from_secs(2));

                    for peer in peers.lock().unwrap().values() {
                        println!("-> {} | {}", peer.get_ip(), peer.get_name());
                    }
                    ack_tx.send(()).unwrap();
                }
                Control::Stop(error) => {
                    println!("{}", error);
                    break;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_writer_updates_peers() {
        use std::sync::mpsc;
        use std::thread;

        let peers: Arc<Mutex<HashMap<IpAddr, Peer>>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = mpsc::channel::<Control>();
        let (ack_tx, ack_rx) = mpsc::channel::<()>();

        let writer_handle = initialize_writer(peers.clone(), rx, ack_tx);

        tx.send(Control::Data(Peer::new(
            "test".to_string(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            8787,
        )))
        .unwrap();
        ack_rx.recv().unwrap();
        {
            println!("first len: {}", peers.lock().unwrap().len());
            for p in peers.lock().unwrap().values() {
                println!("first print: {:#?}", p);
            }
        }

        thread::sleep(Duration::from_secs(1));
        println!("thread sleep: 1s");

        tx.send(Control::Data(Peer::new(
            "tes1".to_string(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
            9876,
        )))
        .unwrap();
        ack_rx.recv().unwrap();
        {
            for p in peers.lock().unwrap().values() {
                println!("second print: {:#?}", p);
            }
        }

        thread::sleep(Duration::from_secs(1));
        println!("thread sleep: 1s");

        tx.send(Control::Data(Peer::new(
            "tes2".to_string(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)),
            9877,
        )))
        .unwrap();
        ack_rx.recv().unwrap();
        {
            for p in peers.lock().unwrap().values() {
                println!("third print: {:#?}", p);
            }
        }

        tx.send(Control::Stop(String::new())).unwrap();

        assert_eq!(peers.lock().unwrap().len(), 2);

        writer_handle.join().unwrap();
    }

    #[test]
    fn test_listener_receives_packet() {
        let (tx, rx) = mpsc::channel::<Control>();
        let (ack_tx, ack_rx) = mpsc::channel::<()>();
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

        let listener_handle = initialize_listener("127.0.0.1:8787",tx, ack_tx, shutdown_rx);
        ack_rx.recv().unwrap();

        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let msg = "LAN_DROP|name=Lancus|port=9999";
        socket.send_to(msg.as_bytes(), "127.0.0.1:8787").unwrap();

        let control = rx.recv().unwrap_or_else(|e| {
            println!("{:?}", e);
            Control::Stop(String::from("Nope!!"))
        });

        shutdown_tx.send(()).unwrap();
        listener_handle.join().unwrap();

        match control {
            Control::Data(peer) => {
                println!("{:#?}", peer);
                assert_eq!(*peer.get_name(), String::from("Lancus"));
                assert_eq!(peer.get_tcp_port(), 9999);
                assert_eq!(*peer.get_ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
            }
            Control::Stop(err) => panic!("Listener stopped unexpectedly: {}", err),
        }
    }
    #[test]
    fn test_listener_writer_combined() {
        let peers: Arc<Mutex<HashMap<IpAddr, Peer>>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = mpsc::channel::<Control>();
        let (ack_tx, ack_rx) = mpsc::channel::<()>();
        let (ack_listener_tx, ack_listener_rx) = mpsc::channel::<()>();
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

        let listener_handler = initialize_listener("127.0.0.1:8788",tx, ack_listener_tx,shutdown_rx);
        ack_listener_rx.recv().unwrap();

        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let msg = "LAN_DROP|name=Lancus|port=9999";
        socket.send_to(msg.as_bytes(), "127.0.0.1:8788").unwrap();

        let _writer_handler = initialize_writer(peers.clone(), rx, ack_tx);
        ack_rx.recv().unwrap();

        // shutdown after the writing is acknowledged
        shutdown_tx.send(()).unwrap();
        listener_handler.join().unwrap();

        for p in peers.lock().unwrap().values() {
            println!("received peer: {:#?}", p);
        }

        let peer = Peer::new("Lancus".to_string(), IpAddr::V4(Ipv4Addr::LOCALHOST), 9999);

        assert_eq!(peers.lock().unwrap().len(), 1);
        assert_eq!(
            peers
                .lock()
                .unwrap()
                .get(&IpAddr::V4(Ipv4Addr::LOCALHOST))
                .unwrap()
                .get_name(),
            peer.get_name()
        );
        assert_eq!(
            peers
                .lock()
                .unwrap()
                .get(&IpAddr::V4(Ipv4Addr::LOCALHOST))
                .unwrap()
                .get_tcp_port(),
            peer.get_tcp_port()
        );
    }
}
