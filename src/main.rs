use std::env;

use rlsd::{data_receiver::{self, Receiver}, data_sender};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    match args.to_vec().get(1).unwrap().as_str() {
        "-c" => data_sender::send(),
        "-s" => {
            let mut receiver = Receiver {stop: false};    
            data_receiver::Receiver::start(&mut receiver).unwrap();
        },
        _ => println!("Not an option.")
    }
}
