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

pub fn display_progress(
    op_type: &str,
    mut total_bytes: usize,
    current_bytes: usize,
    filesize: usize,
    start: &std::time::Instant,
) {
    total_bytes += current_bytes;
    let elapsed = start.elapsed().as_secs_f64();
    let mbps = (total_bytes as f64 / (1024.0 * 1024.0)) / elapsed;
    let percent = (total_bytes as f64 / filesize as f64) * 100.0;
    println!(
        "\r{}: {} B | {:.2}% ({:.2} MB/s)",
        op_type, total_bytes, percent, mbps
    );
}

#[cfg(target_os = "windows")]
pub fn standardize_path(filepath: &str) -> String {
    if filepath.contains("\\") {
        filepath.replace("\\", "/")
    } else {
        filepath.to_string()
    }
}
