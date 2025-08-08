use std::env;

use crossterm::style::Stylize;
use rlsd::{
    config::client::Client,
    constants::{self, get_client_config_path},
    input,
    json_handler::{self, write_json_from_value},
    socket_handling::{self, command_type::Commands, data_receiver::Receiver, data_sender},
    stats_handling::{
        database::{self, get_all_device_uids, get_device_name_from_uid},
        stats_loop,
    },
    tui,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    constants::setup();

    let args: Vec<String> = env::args().collect();

    let vec_args = args.to_vec();

    let database = database::start_db().await;

    match vec_args.get(1).map_or("--help", |v| v) {
        // Help
        "-h" | "--help" => {
            println!(
"Welcome to RLSD help page!

-h | --help => Prints this message

-l | --list => Lists all device uids and their names in the db (run as the user that runs the server)

--setup => Sets up the client config, used in the install script

-c | --client => Runs rlsd in client or daemon mode to send data to the server

-s | --server => Runs the rlsd server on 0.0.0.0:51347 and launches the TUI

-st | --server-notui => Runs the rlsd server on 0.0.0.0:51347 without the TUI

-r | --remove => Removes the supplied id from the db (use --list to get the id): rlsd --remove <ID>    

--config => Configure the server address and device name of the client:
    rlsd --config <name, server-addr> <value>"
            )
        },
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
        "--config" => {
            match vec_args.get(2).map_or("", |v| v) {
                "name" => {
                    match vec_args.get(3) {
                        Some(device_name) => {
                            json_handler::write_client_config("deviceName", device_name);

                            let device_id = json_handler::read_client_config_json("deviceID");

                            let payload = json!({
                                "deviceID": device_id,
                                "deviceName": device_name
                            });

                            let msg = data_sender::send(Commands::RENAME, payload);

                            println!("{msg}")

                        },
                        None => println!("Please supply the name of your device.")
                    }
                },
                "server-addr" => {
                    match vec_args.get(3) {
                        Some(v) => {
                            let addr = if v.find(":").is_none() {
                                format!("{}:51347", v)
                            } else {
                                v.to_string()
                            };

                            json_handler::write_client_config("serverAddr", addr)
                        },
                        None => println!("Please supply the ip address of your server machine")
                    }
                }
                _ => println!("Invalid format, use the following: rlsd --config <setting> <value>\nPossibly settings: name, server-ip")
            }
        },
        // Server, starts the socket on a separate thread and then launches the TUI
        "-s" | "--server" => {
            let db_clone = database.clone();

            let receiver_handle = tokio::spawn(async move {
                let mut receiver = Receiver::new(db_clone, false);
                receiver.start().await.unwrap();
            });

            tui::start_tui(&database).await.unwrap();

            receiver_handle.await.unwrap();

            loop {}
        }
        // Start the server with no TUI
        "-st" | "--server-notui" => {
            let mut receiver = Receiver::new(database, true);
            receiver.start().await.unwrap();

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
