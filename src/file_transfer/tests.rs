#[cfg(test)]
pub mod test {
    #[tokio::test]
    async fn test_transfer() {
        use crate::file_transfer::send::send_file;
        use tokio::net::TcpListener;
        use crate::file_transfer::receive::receive_file;
        // This is a placeholder for actual tests.
        // In a real-world scenario, you would use a testing framework
        // and possibly mock the network interactions.
        println!("Testing file transfer...");

        // Start server
        tokio::spawn(async {
            let socket = TcpListener::bind("0.0.0.0:8080")
                .await
                .map_err(|e| {
                    println!("Unable to bind ip {e:?}");
                    return;
                })
                .unwrap();

            loop {
                match socket.accept().await {
                    Ok((stream, addr)) => {
                        println!("New connection from {:?}", addr);

                        send_file(stream, "test/send.txt").await;
                    }
                    Err(e) => {
                        println!("Failed to accept connection: {:?}", e);
                    }
                }
            }
        });

        // Connect to server as client
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait for server to start

        let socket = TcpListener::bind("0.0.0.0:8080").await.unwrap();

        loop {
            match socket.accept().await {
                Ok(socket) => {async {receive_file("test/received.txt", socket.0).await}.await; break},
                Err(_) => todo!(),
            }
        }

        

        // Compare files
        let sent_file = tokio::fs::read("test/send.txt")
            .await
            .expect("Unable to read sent file.");
        let received_file = tokio::fs::read("test/received.txt")
            .await
            .expect("Unable to read received file.");

        assert_eq!(sent_file, received_file, "Files do not match!");
    }

    #[tokio::test]
    async fn test_socket_host() {
        use tokio::net::TcpListener;

        let socket = TcpListener::bind("0.0.0.0:8080").await.map_err(|e| {
            println!("Unable to bind ip {e:?}");
            return;
        });

        assert_eq!(socket.is_ok(), true, "Socket binding failed");
    }
}
