pub mod config {
    pub mod client;
    pub mod server;
}

pub mod constants;
pub mod macros;

pub mod socket_handling {
    pub mod command_type;
    pub mod data_receiver;
    pub mod data_sender;
}

pub mod stats_handling {
    pub mod conversions;
    pub mod database;
    pub mod device_info;
    pub mod stats_getter;
    pub mod stats_loop;
}

pub mod json_handler;
pub mod tui;