use std::env;

use rlsd::{
    socket_handling::{
        data_receiver::{self, Receiver},
        data_sender,
    },
    stats_handling::{
        database,
        device_info::{self, Device},
        stats_getter::{get_cpu_usage, get_ram_usage, get_unix_timestamp},
    },
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let database = database::start_db().await;

    let device: Device = Device {
        device_id: "TESTID".to_string(),
        device_name: "Test Device".to_string(),
        ram_used: get_ram_usage(),
        ram_total: 1,
        cpu_usage: get_cpu_usage(),
        processes: 0,
        network_in: 0,
        network_out: 0,
        time: get_unix_timestamp(),
    };

    match device_info::input_data(device, database).await {
        Ok(_) => {},
        Err(e) => eprintln!("{e}")
    };

    if args.len() < 3 {
        eprintln!("Not enough arguments!");
        return;
    }

    let command = args.get(2).unwrap();

    match args.to_vec().get(1).unwrap().as_str() {
        "-c" => data_sender::send(command),
        "-s" => {
            let mut receiver = Receiver { exit: false };
            data_receiver::Receiver::start(&mut receiver).unwrap();
        }
        _ => println!("Not an option."),
    }
}
