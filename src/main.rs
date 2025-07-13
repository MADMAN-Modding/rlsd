use rlsd::{constants::conversions::byte, stats};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    println!("User CPU Usage: {}", stats::get_user_cpu_usage());
    println!("RAM Usage: {:.2} GiB", (stats::get_ram_usage() as f64) / (byte::GIBIBYTE));
}
