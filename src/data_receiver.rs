use std::{io::Read, net::{TcpListener, TcpStream}};

use serde_json::Value;

use crate::json_handler::read_json_from_buf;

pub struct Receiver {
    pub stop: bool,
}

impl Receiver {
    pub fn start(&mut self) -> std::io::Result<()> {
        match TcpListener::bind("127.0.0.1:8080") {
            Ok(listener) => self.handle_connection(listener),
            Err(e) => return Err(e)
        };

        Ok(())
    }
    
    fn handle_connection(&mut self, listener: TcpListener) {
        for stream in listener.incoming() {
            // If the incoming traffic is valid then process it, otherwise print an error and continue to the next loop
            match stream {
                Ok(stream) => {
                    process_request(stream);
                }
                Err(e) => {
                    eprintln!("Failed to handle the connection: {e}");
                    continue;
                }
            }

            if self.stop {
                break;
            }
        }
    }

    fn stop(&mut self) {
        self.stop = true;
    }
}


fn process_request(mut stream: TcpStream) {
    let mut buf= [0; 1024];

    stream.read(&mut buf).unwrap();

    let raw_string = String::from_utf8_lossy(&buf).trim().to_string();
    let string: String = raw_string.chars().filter(|&c| c != '\u{0000}').collect();

    println!("{}", string);

    let json: Value = serde_json::from_str(string.as_str()).unwrap();

    println!("{}", read_json_from_buf("test", json));
}