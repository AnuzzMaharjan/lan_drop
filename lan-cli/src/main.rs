// launch receiver with arguments from command line
fn launch_receiver(args: Vec<String>) {
    let port = args.iter().position(|x| x == "-p").and_then(|i| args.get(i + 1)).unwrap_or_else(|| {
        eprintln!("Port not specified! Usage: lan-cli -p <port> <file_save_path> [filename]");
        std::process::exit(1);
    });
    let file_save_path = args.iter().position(|x| x == "-f").and_then(|i| args.get(i + 1)).unwrap_or_else(|| {
        eprintln!("file save path not specified! Usage: lan-cli -p <port> <file_save_path> [filename]");
        std::process::exit(1);
    });
    let filename = args.iter().position(|x| x == "-f").and_then(|i| args.get(i + 2)).unwrap_or_else(|| {
        eprintln!("Filename not specified! Usage: lan-cli -p <port> <file_save_path> [filename]");
        std::process::exit(1);
    }) ;

    lan_core::init_receiver(
        port.to_owned(),
        Some(file_save_path.to_owned()),
        Some(filename.to_owned()),
    );
}

fn launch_sender(args: Vec<String>) {
    let ip_addr = args[2].clone().parse().expect("Invalid IP address!");
    let port = args[3].clone();
    let file_path = args[4].clone();
    if let Err(e) = lan_core::sender(ip_addr, port, file_path) {
        eprintln!("Error sending file: {}", e.get_message());
    } else {
        println!("File sent successfully!");
    }
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
                eprintln!("Usage: lan-cli receive -p <port> -f <file_save_path> [filename]");
                return;
            }

            launch_receiver(args);
        }
        "send" => {
            println!("Sending file...");
            if args.len() < 5 {
                eprintln!("Usage: lan-cli send <ip> <port> <file_path>");
                return;
            }

            launch_sender(args);
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
    
    // initialize the thread pool at the start of the program
    lan_core::get_thread_pool();

    controller(args);
}
