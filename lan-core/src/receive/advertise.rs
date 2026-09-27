use std::net::UdpSocket;

use crate::custom_types::Control;

fn build_message(name:&str, ip_addr:&str,port:&str)->String{
    format!("{}\t| {} | {}",name,ip_addr,port)
}

fn send_message_once(udp_socket: &UdpSocket, msg:&str, addr:&str)->std::io::Result<usize>{
    udp_socket.send_to(msg.as_bytes(), addr)
}

pub fn advertise(port:String, receiver: std::sync::mpsc::Receiver<Control>) {
    let ip_addr = crate::utils::get_local_ip().unwrap().to_string();

    // subnet-directed broadcast - routes to the correct interface
    let broadcast_addr = {
        let mut parts: Vec<&str> = ip_addr.split('.').collect();
        parts[3] = "255";
        format!("{}:8787", parts.join("."))
    };

    let socket = UdpSocket::bind(format!("{}:0", ip_addr)).unwrap();
    socket.set_broadcast(true).unwrap();

    let name = match std::env::var("COMPUTERNAME"){
        Ok(n) => n,
        Err(_) => String::from("Lan_Drop")
    };

    let msg = build_message(&name,&ip_addr,&port);
    println!("{}",msg);

    loop{
        // try and listen for stop signal from receiver, if received, break the loop and stop advertising
        if let Ok(Control::Stop(_)) = receiver.try_recv() {
            break;
        }


        send_message_once(&socket,&msg, &broadcast_addr).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_message_builder(){
        let ip_addr = crate::utils::get_local_ip().unwrap().to_string();
        let message = build_message("Lancus",&ip_addr,"9000");
        println!("built message: {}",message);
        let msg = format!("Lancus\t| {} | 9000",ip_addr);
        assert_eq!(message,msg);
    }

    #[test]
    fn test_send_once(){
        let listener = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let message = "LAN_DROP|name=Lancus|port=9000";

        send_message_once(&sender, message, addr.to_string().as_str()).unwrap();

        let mut buff = [0; 1024];
        let (len,_) = listener.recv_from(&mut buff).unwrap();
        let received = std::str::from_utf8(&buff[..len]).unwrap();
        
        println!("Sent message: {}", message);
        println!("Received message: {}", received);
        
        assert_eq!(message,received);
    }
}