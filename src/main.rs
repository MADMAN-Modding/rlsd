use std::env;

use crossterm::style::Stylize;
use rlsd::{
    config::client::Client,
    constants::{self, get_client_config_path},
    input,
    json_handler::write_json_from_value,
    socket_handling::{self, data_receiver::Receiver},
    stats_handling::{
        database::{self, get_all_device_uids, get_device_name_from_uid},
        stats_loop,
    },
    tui,
};

#[tokio::main]
async fn main() {
    constants::setup();

    let args: Vec<String> = env::args().collect();

    let database = database::start_db().await;

    match args.to_vec().get(1).unwrap().as_str() {
        // Help

        // List, lists all the uids and their friendly names
        "-l" | "--list" => {
            let ids = get_all_device_uids(&database).await;

            for id in ids {
                println!("{}: {}", get_device_name_from_uid(&database, &id).await, id)
            }
        }
        // Setup, sets the client config and gets the uid
        "--setup" => setup(),
        // Client, 1 minute loops for sending data
        "-c" | "--client" => stats_loop::start_stats_loop().await,
        // Remove, removes the supplied id from the local database
        "-r" | "--remove" => {
            match args.to_vec().get(2) {
                Some(id) => database::remove_device(&database, id).await,
                None => eprintln!("Please specify a device id")
            }
        }
        // Server, starts the socket on a separate thread and then launches the TUI
        "-s" | "--server" => {
            let db_clone = database.clone();

            let receiver_handle = tokio::spawn(async move {
                let mut receiver = Receiver::new(db_clone);
                receiver.start().await.unwrap();
            });



            tui::start_tui(&database).await.unwrap();

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
