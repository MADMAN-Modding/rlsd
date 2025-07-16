pub mod constants;
pub mod socket_handling {
    pub mod command_type;
    pub mod data_receiver;
    pub mod data_sender;
}
pub mod stats_handling {
    pub mod database;
    pub mod device_info;
    pub mod stats_getter;
}
pub mod json_handler;
