use regex::Regex;

pub fn filepath_contains_filename(filepath: &str) -> bool {
    let filename_regex = Regex::new(r"(?i)^[\w\s,:\\/-]+\.[A-Za-z]+$").unwrap();
    filename_regex.is_match(filepath)
}

pub fn get_local_ip() -> std::io::Result<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    socket.local_addr().map(|a| a.ip())
}

#[cfg(target_os = "windows")]
pub fn standardize_path(filepath: &str) -> String {
    if filepath.contains("\\") {
        filepath.replace("\\", "/")
    } else {
        filepath.to_string()
    }
}