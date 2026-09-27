use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}};

#[derive(Debug, PartialEq, Eq)]
pub struct Node{
    freq: usize,
    byte: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeapNode(Node);

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.freq.cmp(&self.0.freq)
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn generate_codes(node: &Node, prefix: String, codes: &mut HashMap<u8, String>) {
    if let Some(b) = node.byte {
        codes.insert(b, prefix);
    } else {
        if let Some(left) = &node.left {
            generate_codes(left, prefix.clone() + "0", codes);
        }
        if let Some(right) = &node.right {
            generate_codes(right, prefix.clone() + "1", codes);
        }
    }
}

pub fn build_huffman_tree(freq_vec: Vec<(char, usize)>) -> Node {
    let mut heap = BinaryHeap::new();

    for (ch, freq) in freq_vec {
        let node = Node {
            freq,
            byte: Some(ch as u8),
            left: None,
            right: None,
        };
        heap.push(HeapNode(node));
    }
    
    while heap.len() > 1 {
        let left = heap.pop().unwrap().0;
        let right = heap.pop().unwrap().0;
        let parent = Node {
            freq: left.freq + right.freq,
            byte: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right))
        };

        heap.push(HeapNode(parent));
    }
    heap.pop().unwrap().0
}

pub fn create_frequency_table(filepath: &str)-> (Vec<(char, usize)>, Vec<u8>){
    let mut frequency_table: HashMap<char, usize> = HashMap::new();

    let read_bytes = read_the_file(filepath);

    for char in read_bytes.iter() {
        let char = *char as char;
        let count = frequency_table.entry(char).or_insert(0);
        *count += 1;
    }

    let  freq_vec: Vec<(char, usize)> = frequency_table.into_iter().collect();

    (freq_vec, read_bytes)
}

fn read_the_file(filepath: &str) -> Vec<u8> {
    let read_buffer = std::fs::read(filepath).expect("Failed to read file");
    read_buffer
}

pub fn encode(read_bytes: Vec<u8>, codes: &HashMap<u8, String>) -> (usize, Vec<u8>) {
    let mut string_bits = String::new();
    
    for byte in read_bytes.iter() {
        let code = codes.get(byte).unwrap();
        string_bits.push_str(code);
    }

    let padding = (8- (string_bits.len() % 8)) % 8;
    for _ in 0..padding {
        string_bits.push('0');
    }
    
    let mut encoded_bytes: Vec<u8> = Vec::new();
    for chunked_bits in string_bits.chars().collect::<Vec<char>>().chunks(8){
        let byte = u8::from_str_radix(&chunked_bits.iter().collect::<String>(), 2).unwrap();
        encoded_bytes.push(byte);
    }

    (padding, encoded_bytes)
}

pub fn decode(bitstream: &str, root: &Node) -> Vec<u8> {
    let mut result = Vec::new();
    let mut current = root;

    for bit in bitstream.chars(){
        current = if bit == '0' {
            current.left.as_ref().unwrap()
        } else {
            current.right.as_ref().unwrap()
        };

        if let Some(byte) = current.byte {
            result.push(byte);
            current = root;
        }
    }

    result
}