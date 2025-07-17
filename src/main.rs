use std::env;

use rlsd::{
    socket_handling::{
        data_receiver::{self, Receiver},
        data_sender,
    },
    stats_handling::{
        database,
        device_info::Device,
        stats_getter::{get_cpu_usage, get_network_in, get_network_out, get_processes, get_ram_total, get_ram_usage, get_unix_timestamp},
    },
};
use systemstat::{Platform, System};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let database = database::start_db().await;

    let sys = System::new();

    let device: Device = Device {
        device_id: "TESTID".to_string(),
        device_name: "Socket Test Device".to_string(),
        ram_used: get_ram_usage(&sys),
        ram_total: get_ram_total(&sys),
        cpu_usage: get_cpu_usage(&sys),
        processes: get_processes(),
        network_in: get_network_in(&sys),
        network_out: get_network_out(&sys),
        time: get_unix_timestamp(),
    };

    let command = args.get(2).unwrap_or(&"ERROR".to_string()).to_string();

    match args.to_vec().get(1).unwrap().as_str() {
        "-c" => data_sender::send(command, device.to_json()),
        "-s" => {
            let mut receiver = Receiver { exit: false, database: database };
            data_receiver::Receiver::start(&mut receiver).await.unwrap();
        }
        _ => println!("Not an option."),
    }
}
