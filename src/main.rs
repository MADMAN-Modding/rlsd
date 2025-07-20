use std::env;

use crossterm::style::Stylize;
use rlsd::{
    config::client::Client,
    constants::{self, get_client_config_path},
    input,
    json_handler::{read_client_config_json, write_json_from_value},
    socket_handling::{self, command_type::Commands, data_receiver::Receiver, data_sender},
    stats_handling::{
        database,
        device_info::Device,
        stats_getter::{
            get_cpu_usage, get_network_in, get_network_out, get_processes, get_ram_total,
            get_ram_usage, get_unix_timestamp,
        },
        stats_loop,
    },
    tui,
};
use systemstat::{Platform, System};

#[tokio::main]
async fn main() {
    constants::setup();

    let args: Vec<String> = env::args().collect();

    let database = database::start_db().await;

    match args.to_vec().get(1).unwrap().as_str() {
        // Message mode, just used for testing
        "-m" | "--message" => {
            let sys = &System::new();

            let data = Device::new(
                read_client_config_json("deviceID"),
                read_client_config_json("deviceName"),
                get_ram_usage(sys),
                get_ram_total(sys),
                get_cpu_usage(sys),
                get_processes(),
                get_network_in(sys),
                get_network_out(sys),
                get_unix_timestamp(),
            );

            data_sender::send(Commands::INPUT, data.to_json());
        }
        // Setup mode
        "-s" | "--setup" => setup(),
        // Client mode, 1 minute loops for sending data
        "-c" | "--client" => stats_loop::start_stats_loop().await,
        // Server mod
        "--server" => {
            let db_clone = database.clone();

            let receiver_handle = tokio::spawn(async move {
                let mut receiver = Receiver::new(db_clone);
                receiver.start().await.unwrap();
            });

            // for device_id in database::get_all_device_uids(&database).await.iter() {
            //     println!(
            //         "{}:{}",
            //         device_id,
            //         database::get_device_name_from_uid(&database, device_id).await
            //     )
            // }

            tui::start_tui(database).unwrap();

            receiver_handle.await.unwrap();

            loop {}
        }
        _ => println!("Not an option."),
    }
}

/// This function is used to setup a new device to connect to a server
pub fn setup() {
    let device_name = input!("Name for your device to be shown: ");

    let mut server_addr = input!("IP of the server machine (No CIDR)");

    if server_addr.find(":").is_none() {
        server_addr = format!("{}:51347", server_addr);
    }

    let device_id = socket_handling::data_sender::setup(&server_addr);

    let client_conf = Client::new(device_id, device_name, server_addr);

    write_json_from_value(&get_client_config_path(), client_conf.to_json());

    println!("Device info:\n{}", client_conf.to_string().green().bold())
}
