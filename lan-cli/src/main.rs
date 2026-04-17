use std::process::Command;

fn launch_receiver(
    ip_addr: String,
    port: String,
    file_save_path: String,
    filename: Option<String>,
) {
    let full_recv_command = format!(
        "cargo run --bin rec -- {} {} {} {}",
        ip_addr,
        port,
        file_save_path,
        filename.unwrap_or_default()
    );

    let full_adv_command = format!("cargo run --bin adv -- {} {}", ip_addr, port);

    Command::new("cmd")
        .args(["/C", "start", "cmd", "/K", &full_recv_command])
        .spawn()
        .expect("Failed to launch receiver window");

    Command::new("cmd")
        .args(["/C", "start", "cmd", "/K", &full_adv_command])
        .spawn()
        .expect("Failed to launch advertiser window");
}

fn controller(args: Vec<String>) {
    if args.len() < 2 {
        eprintln!("Usage: lan-cli <command> [options]");
        return;
    }

    match args[1].as_str() {
        "discover" => {
            println!("Discovering devices...");
            lan_core::discover();
        }
        "receive" => {
            println!("Receiving file...");
            if args.len() < 5 {
                eprintln!("Usage: lan-cli receive <ip> <port> <file_save_path> [filename]");
                return;
            }
            let ip_addr = args[2].clone();
            let port = args[3].clone();
            let file_save_path = args[4].clone();
            let filename = args.get(5).cloned();
            launch_receiver(ip_addr, port, file_save_path, filename);
        },
        "send" => {
            println!("Sending file...");
            if args.len() < 5 {
                eprintln!("Usage: lan-cli send <ip> <port> <file_path>");
                return;
            }
            let ip_addr = args[2].clone();
            let port = args[3].clone();
            let file_path = args[4].clone();
            if let Err(e) = lan_core::sender(ip_addr, port, file_path) {
                eprintln!("Error sending file: {}", e.get_message());
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Available commands: discover, receive");
        }
    }
}
fn take_arguments() -> Vec<String> {
    std::env::args().collect()
}

fn main() {
    let args = take_arguments();
    controller(args);
}
