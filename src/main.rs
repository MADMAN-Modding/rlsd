use std::env;

use rlsd::socket_handling::{data_receiver::{self, Receiver}, data_sender};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Not enough arguments!");
        return;
    }

    let con_type = args.get(1).unwrap();
    
    let command = args.get(2).unwrap();

    match args.to_vec().get(1).unwrap().as_str() {
        "-c" => data_sender::send(command),
        "-s" => {
            let mut receiver = Receiver {exit: false};    
            data_receiver::Receiver::start(&mut receiver).unwrap();
        },
        _ => println!("Not an option.")
    }
}
