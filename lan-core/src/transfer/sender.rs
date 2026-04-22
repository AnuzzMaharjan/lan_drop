use bincode::{self, config};
use std::{
    fs::File, io::{BufReader, Read, Write}, net::TcpStream, sync::{Arc, Mutex}, time::Instant
};

use lan_engine::MerkleTree;

use crate::{
    custom_types::{ErrorMessage, FileMetaData, SendFileData}, threadpool, utils::{display_progress, filepath_contains_filename, standardize_path}
};

pub fn send_file(to_be_sent_data: &SendFileData) -> Result<(), ErrorMessage> {
    let unix_path = standardize_path(to_be_sent_data.get_filepath());
    
    if !filepath_contains_filename(&unix_path) {
        return Err(ErrorMessage::new(
            "Invalid filepath!".to_string(),
            std::io::ErrorKind::InvalidInput,
        ));
    }
    if File::open(&unix_path).is_err() {
        return Err(ErrorMessage::new(
            "File not found!".to_string(),
            std::io::ErrorKind::NotFound,
        ));
    }

    // wrap the unix path in Arc and Mutex to share between threads after standardization and validation
    let unix_path = Arc::new(Mutex::new(unix_path));

    let pool = threadpool::get_thread_pool();
    let (sender, receiver) = std::sync::mpsc::channel::<Result<MerkleTree, ErrorMessage>>();
    let thread_safe_unix_path = Arc::clone(&unix_path);

    pool.execute(move || {
        let mtree = match MerkleTree::new(&thread_safe_unix_path.lock().unwrap()) {
            Ok((tree, _)) => tree,
            Err(e) => {
                sender.send(Err(ErrorMessage::new(
                    "Failed to create Merkle Tree!".to_string(),
                    e.kind(),
                ))).unwrap();
                return;
            }
        };
        sender.send(Ok(mtree)).unwrap();
    });


    let mut stream = match TcpStream::connect((
        to_be_sent_data.get_ip().to_string().as_str(),
        to_be_sent_data.get_tcp_port(),
    )) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to receiver!");
            return Err(ErrorMessage::new(
                "Failed to connect to receiver!".to_string(),
                e.kind(),
            ));
        }
    };

    println!(
        "Connected to receiver at {}:{}",
        to_be_sent_data.get_ip(),
        to_be_sent_data.get_tcp_port()
    );

    // blocks until the Merkle Tree is created in the other thread
    let mtree = receiver.recv().unwrap()?;

    let (filename, file_size) = {
        let path_guard = &unix_path.lock().unwrap();
        let filename = path_guard.split("/").last().unwrap().to_string();
        let file_size = File::open(path_guard.as_str()).unwrap().metadata().unwrap().len();
        (filename, file_size)
    };

    let metadata = FileMetaData {
        filename: filename,
        file_size: file_size,
        merkle_root: mtree.get_root_hash().unwrap(),
        chunk_size: {
            if file_size < 10*1024*1024 {
                file_size as u32
            } else {
             10*1024*1024 as u32
            }
        }
    };

    let serialized_metadata: Vec<u8> =
        bincode::encode_to_vec(&metadata, config::standard()).unwrap();
    let metadata_size = serialized_metadata.len() as u32;

    if let Err(e) = stream.write_all(&metadata_size.to_be_bytes()) {
        return Err(ErrorMessage::new(
            "Failed to send metadata size!".to_string(),
            e.kind(),
        ));
    }

    if let Err(e) = stream.write_all(&serialized_metadata) {
        return Err(ErrorMessage::new(
            "Failed to send metadata!".to_string(),
            e.kind(),
        ));
    };

    // flush the stream before sending the file
    if let Err(e) = stream.flush() {
        return Err(ErrorMessage::new(
            "Failed to flush stream!".to_string(),
            e.kind(),
        ));
    };

    if let Err(e) = stream_file(&to_be_sent_data.get_filepath(), &mut stream) {
        return Err(ErrorMessage::new(
            "Failed to send file data!".to_string(),
            e.kind(),
        ));
    };

    Ok(())
}

fn stream_file(file_path: &str, stream: &mut TcpStream) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let filesize = file.metadata()?.len();

    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    let total_bytes = 0;
    let start = Instant::now();

    loop {
        let n = reader.read(&mut buffer)?;
        println!("Read {:?} bytes from file...", n);
        if n == 0 {
            break;
        }
        stream.write_all(&buffer[..n])?;

        display_progress(total_bytes, n, filesize.try_into().unwrap(), &start);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree() {
        const TEST_PATH: &str = "D:/rust/projects/lan_drop_v3/lan-cli/testtt.txt";

        let (tree, leaf_hashes) = MerkleTree::new(TEST_PATH).unwrap();
        println!("Merkle Tree: {:#?}", tree);
        println!("Leaf Hashes: {:#?}", leaf_hashes);
        assert_eq!(1, 1);
    }
}
