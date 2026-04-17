use std::net::TcpListener;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 4 {
        eprintln!("Usage: cargo run --bin rec <ip> <port> <file_save_path> [filename]");
        return;
    }

    println!("Starting receiver on {}:{}", args[1], args[2]);

    let listener = TcpListener::bind(format!("{}:{}", args[1], args[2])).expect("Failed to bind to address");

    lan_core::receiver(&listener, args[3].clone(), args.get(4).cloned());
}