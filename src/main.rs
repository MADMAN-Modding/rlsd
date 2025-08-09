use std::env;

use crossterm::style::Stylize;
use rlsd::{
    config::client::ClientConfig,
    constants::{self, get_client_config_path, get_server_config_path},
    input,
    json_handler::{self, read_client_config_json, read_json_as_value, write_json_from_value, write_server_config},
    socket_handling::{self, command_type::Commands, server::Receiver, client},
    stats_handling::{
        database::{self, get_all_device_uids, get_device_name_from_uid},
        stats_loop,
    },
    tui,
};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    constants::setup();

    let args: Vec<String> = env::args().collect();

    let args = args.to_vec();

    let database = database::start_db().await;

    match args.get(1).map_or("--help", |v| v) {
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

-a | --admin => Adds a device as an admin using the supplied id: rlsd -a <ID>

-r | --remove => Removes the supplied id from the db (use --list to get the id): rlsd --remove <ID>    

-rr | --remove-remote => (admin only) Removes the supplied id from the db on the configured server (use -rl to get the id): rlsd -rl <ID>

-rl | --remote-list => (admin only) Lists all the devices on the server and their ids (does not include admin ids for security)

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
            match args.get(2) {
                Some(id) => println!("{}", database::remove_device(&database, id).await),
                None => eprintln!("Please specify a device id")
            }
        }
        // Remove a device on the remote server (admin) 
        "-rr" | "--remove-remote" => {
            let removed_device_id = match args.get(2) {
                Some(id) => id,
                None => return
            };

            let payload = json!({
                "deviceID": sha256::digest(read_client_config_json("deviceID")),
                "removedDeviceID": removed_device_id
            });

            println!("{}", client::send(Commands::REMOVE, payload));
        }
        // List the devices on the remote server (admin)
        "-rl" | "--remote-list" => {
            let sha_device_id = sha256::digest(read_client_config_json("deviceID"));

            println!("{}", client::send(Commands::LIST, json!({"deviceID": sha_device_id})));
        }
        // Configure settings for the client
        "--config" => {
            match args.get(2).map_or("", |v| v) {
                "name" => {
                    match args.get(3) {
                        Some(device_name) => {
                            json_handler::write_client_config("deviceName", Value::String(device_name.to_owned()));

                            let device_id = json_handler::read_client_config_json("deviceID");

                            let payload = json!({
                                "deviceID": device_id,
                                "deviceName": device_name
                            });

                            let msg = client::send(Commands::RENAME, payload);

                            println!("{msg}")

                        },
                        None => println!("Please supply the name of your device.")
                    }
                },
                "server-addr" => {
                    match args.get(3) {
                        Some(v) => {
                            let addr = if v.find(":").is_none() {
                                format!("{}:51347", v)
                            } else {
                                v.to_string()
                            };

                            json_handler::write_client_config("serverAddr", Value::String(addr))
                        },
                        None => println!("Please supply the ip address of your server machine")
                    }
                }
                _ => println!("Invalid format, use the following: rlsd --config <setting> <value>\nPossibly settings: name, server-ip")
            }
        },
        "-a" | "--admin" => {
            let admin_id = match args.get(2) {
                Some(id) => id.to_owned(),
                None => return
            };

            let config = read_json_as_value(&get_server_config_path());

            let mut config = config.get("adminIDs").unwrap().as_array().unwrap().to_owned();

            config.push(Value::String(sha256::digest(admin_id.clone())));

            write_server_config("adminIDs", serde_json::Value::Array(config));

            println!("Added: {admin_id} to the admin list");
        }
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

    let device_id = socket_handling::client::setup(&server_addr);

    let client_conf = ClientConfig::new(device_id, device_name, server_addr);

    write_json_from_value(&get_client_config_path(), client_conf.to_json());

    println!("Device info:\n{}", client_conf.to_string().green().bold())
}
