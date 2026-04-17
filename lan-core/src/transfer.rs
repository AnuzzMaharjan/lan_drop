mod discovery;
mod sender;

pub use discovery::discover;

use crate::custom_types::{ErrorMessage, SendFileData};

pub fn sender(ip_addr: String, port: String, file_path: String) -> Result<(), ErrorMessage> {
    let send_file_data = SendFileData::new(
        file_path,
        ip_addr.parse().expect("Invalid IP address!"),
        port.parse().expect("Invalid port!")
    );

    sender::send_file(&send_file_data)
}