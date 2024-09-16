use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    // Shared state for active connections
    let connections = Arc::new(Mutex::new(HashSet::new()));

    // Bind a TCP listener to the localhost on port 8080
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server running on 127.0.0.1:8080");

    // Handle Ctrl + C signal gracefully
    let shutdown_signal = signal::ctrl_c();
    tokio::pin!(shutdown_signal);

    // Periodic task to print active connections
    let connections_clone = Arc::clone(&connections);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let connections = connections_clone.lock().unwrap();
            println!("Active connections: {:?}", *connections);
        }
    });

    loop {
        tokio::select! {
            // Accept new connections
            Ok((mut socket, addr)) = listener.accept() => {
                println!("New connection from: {}", addr);
                connections.lock().unwrap().insert(addr);

                // Spawn a new task for each connection
                let connections_clone = Arc::clone(&connections);
                tokio::spawn(async move {
                    let mut buffer = [0; 1024];  // Buffer to store incoming data

                    // Read data from the client
                    match socket.read(&mut buffer).await {
                        Ok(n) if n == 0 => {
                            println!("Connection closed by: {}", addr);
                            connections_clone.lock().unwrap().remove(&addr);
                            return;  // Client disconnected
                        }
                        Ok(n) => {
                            // Convert the message to uppercase and add "!!!"
                            let message = String::from_utf8_lossy(&buffer[..n]);
                            println!("Message received: {}", message);
                            let response = format!("{}!!!", message.to_uppercase());

                            // Send the response back to the client
                            if let Err(e) = socket.write_all(response.as_bytes()).await {
                                println!("Failed to send response to {}: {}", addr, e);
                            }
                        }
                        Err(e) => {
                            println!("Failed to read from socket; err = {:?}", e);
                            connections_clone.lock().unwrap().remove(&addr);
                            return;
                        }
                    }
                });
            }

            // Handle Ctrl + C shutdown signal
            _ = &mut shutdown_signal => {
                println!("Shutdown signal received. Closing server...");
                break;
            }
        }
    }

    println!("Server shut down.");
}
