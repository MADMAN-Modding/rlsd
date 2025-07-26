use std::{thread, time::Duration};
use systemstat::{Platform, System};

use crate::{
    json_handler::read_client_config_json,
    socket_handling::{command_type::Commands, data_sender},
    stats_handling::{
        device_info::Device,
        stats_getter::{
            get_cpu_usage, get_network_in, get_network_out, get_processes, get_ram_total,
            get_ram_usage, get_unix_timestamp,
        },
    },
};

pub async fn start_stats_loop() {
    thread::spawn(|| {
        let device_id = read_client_config_json("deviceID");

        let device_name = read_client_config_json("deviceName");

        loop {
            let sys = &System::new();

            let device = Device::new(
                &device_id,
                &device_name,
                get_ram_usage(sys),
                get_ram_total(sys),
                get_cpu_usage(sys),
                get_processes(),
                get_network_in(sys),
                get_network_out(sys),
                get_unix_timestamp(),
            );

            data_sender::send(Commands::INPUT, device.to_json());
            thread::sleep(Duration::from_secs(60));
        }
    })
    .join()
    .unwrap();
}
