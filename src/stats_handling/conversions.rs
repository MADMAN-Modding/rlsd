pub fn format_bytes(bytes: i64) -> i64 {
    if bytes > 1024 {
        format_bytes(bytes / 1024)
    } else {
        bytes
    }
}