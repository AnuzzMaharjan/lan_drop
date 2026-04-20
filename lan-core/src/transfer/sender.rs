use std::{fs::File, io::{BufReader, Read, Write}, net::TcpStream, time::Instant};
use bincode::{self, config};

use lan_engine::MerkleTree;

use crate::{custom_types::{ErrorMessage, FileMetaData, SendFileData}, utils::{filepath_contains_filename, standardize_path}};


pub fn send_file(to_be_sent_data:&SendFileData)->Result<(), ErrorMessage>{
    let unix_path = standardize_path(to_be_sent_data.get_filepath());

    if !filepath_contains_filename(&unix_path) {
        return Err(ErrorMessage::new("Invalid filepath!".to_string(), std::io::ErrorKind::InvalidInput));
    }

    if File::open(&unix_path).is_err() {
        return Err(ErrorMessage::new("File not found!".to_string(), std::io::ErrorKind::NotFound));
    }

    let mtree = match MerkleTree::new(&unix_path) {
        Ok((tree, _)) => tree,
        Err(e) => {
            return Err(ErrorMessage::new("Failed to create Merkle Tree!".to_string(), e.kind()));
        }
    };

    let metadata = FileMetaData {
        filename: unix_path.split("/").last().unwrap().to_string(),
        file_size: File::open(&unix_path).unwrap().metadata().unwrap().len(),
        merkle_root: mtree.get_root_hash().unwrap(),
        chunk_size: 8192
    };

    let mut stream = match TcpStream::connect((to_be_sent_data.get_ip().to_string().as_str(), to_be_sent_data.get_tcp_port())) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to receiver!");
            return Err(ErrorMessage::new("Failed to connect to receiver!".to_string(), e.kind()));
        }
    };


    println!("Connected to receiver at {}:{}", to_be_sent_data.get_ip(), to_be_sent_data.get_tcp_port());
    println!("Metadata: {:#?}", metadata);

    let serialized_metadata: Vec<u8> = bincode::encode_to_vec(&metadata, config::standard()).unwrap();
    let metadata_size = serialized_metadata.len() as u32;

    if let Err(e) = stream.write_all(&metadata_size.to_be_bytes()) {
        return Err(ErrorMessage::new("Failed to send metadata size!".to_string(), e.kind()));
    }

    if let Err(e) = stream.write_all(&serialized_metadata) {
        return Err(ErrorMessage::new("Failed to send metadata!".to_string(), e.kind()));
    };

    // flush the stream before sending the file
    if let Err(e) = stream.flush() {
        return Err(ErrorMessage::new("Failed to flush stream!".to_string(), e.kind()));
    };

    if let Err(e) = stream_file(&to_be_sent_data.get_filepath(), &mut stream) {
        return Err(ErrorMessage::new("Failed to send file data!".to_string(), e.kind()));
    };

    Ok(())
}

fn stream_file(file_path: &str, stream: &mut TcpStream) -> std::io::Result<()>{
    let file = File::open(file_path)?;
    let filesize = file.metadata()?.len();
    
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    let mut total_bytes = 0;
    let start = Instant::now();

    loop {       
        let n = reader.read(&mut buffer)?;
        println!("Read {:?} bytes from file...", n);
        if n==0{
            break;
        }
        stream.write_all(&buffer[..n])?;

        total_bytes += n;
        let elapsed = start.elapsed().as_secs_f64();
        let mbps = (total_bytes as f64 / 1024.0 / 1024.0) / elapsed;
        let percent = (total_bytes as f64 / filesize as f64) * 100.0;
        println!("\rSent: {} B | {:.2}% ({:.2} MB/s)", total_bytes, percent, mbps);
        std::io::stdout().flush()?;
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
        assert_eq!(1,1);
    }
}