use std::env;

use crossterm::style::Stylize;
use rlsd::{
    config::client::Client,
    constants::{self, get_client_config_path},
    input,
    json_handler::write_json_from_value,
    socket_handling::{self, data_receiver::Receiver},
    stats_handling::database,
};

#[tokio::main]
async fn main() {
    constants::setup();

    let args: Vec<String> = env::args().collect();

    let database = database::start_db().await;

    // let sys = System::new();

    // let device: Device = Device {
    //     device_id: "TEST".to_string(),
    //     device_name: "Socket Test Device".to_string(),
    //     ram_used: get_ram_usage(&sys),
    //     ram_total: get_ram_total(&sys),
    //     cpu_usage: get_cpu_usage(&sys),
    //     processes: get_processes(),
    //     network_in: get_network_in(&sys),
    //     network_out: get_network_out(&sys),
    //     time: get_unix_timestamp(),
    // };

    match args.to_vec().get(1).unwrap().as_str() {
        "-s" | "--setup" => setup(),
        "--server" => {
            let mut receiver = Receiver::new(database);

            receiver.start().await.ok();
        }
        _ => println!("Not an option."),
    }
}

/// This function is used to setup a new device to connect to a server
pub fn setup() {
    let device_name = input!("Name for your device to be shown: ");

    let server_addr = format!("{}:51347", input!("IP of the server machine (no port/CIDR)"));

    let device_id = socket_handling::data_sender::setup(&server_addr);

    let client_conf = Client::new(device_id, device_name, server_addr);

    write_json_from_value(&get_client_config_path(), client_conf.to_json());

    println!("Device info: {}", client_conf.to_string().green().bold())
}
