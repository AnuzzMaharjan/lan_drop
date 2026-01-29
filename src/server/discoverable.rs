use std::net::{ UdpSocket };
pub fn announce(port:String){
    let broadcast_addr = "255.255.255.255:8787";

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_broadcast(true).unwrap();

    let msg = "LAN_DROP|name=Lancus|port=".to_string()+port.as_str();

    println!("Broadcast: {}",msg);
    loop {
        socket.send_to(msg.as_bytes(), broadcast_addr).unwrap();
        println!("Sent {} bytes...", msg.len());
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}