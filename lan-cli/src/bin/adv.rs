
fn main(){
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 3 {
        eprintln!("Usage: cargo run --bin adv <ip> <port>");
        return;
    }

    let ip_addr = args[1].clone();
    let port = args[2].clone();

    lan_core::advertise(ip_addr, port);
}