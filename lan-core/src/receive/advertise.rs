use std::net::UdpSocket;

fn build_message(name:&str,port:&str)->String{
    format!("LAN_DROP|name={}|port={}",name,port)
}

fn send_message_once(udp_socket: &UdpSocket, msg:&str, addr:&str)->std::io::Result<usize>{
    udp_socket.send_to(msg.as_bytes(), addr)
}

pub fn advertise(name:String,port:String){
    let broadcast_addr = "255.255.255.255:8787";

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_broadcast(true).unwrap();

    let msg = build_message(&name,&port);
    println!("{}",msg);

    loop{
        send_message_once(&socket,&msg, broadcast_addr).unwrap();
        println!("Sent {} bytes...",msg.len());
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_message_builder(){
        let message = build_message("Lancus","9000");
        println!("built message: {}",message);
        assert_eq!(message,"LAN_DROP|name=Lancus|port=9000");
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