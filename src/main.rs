use rlsd::{app::App, constants::conversions::byte, stats};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // println!("System CPU Usage: {:.3}", stats::get_user_cpu_usage());
    println!("RAM Usage: {:.2} GiB", (stats::get_ram_usage() as f64) / (byte::GIBIBYTE));

    App::setup().ok();
}
