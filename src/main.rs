use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    println!("Attempting to start server");

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server running on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("New connection from: {}", addr);

        tokio::spawn(async move {
            let mut buffer = [0; 1024]; // Buffer to store incoming data
            let n = socket.read(&mut buffer).await.unwrap();

            if n == 0 {
                return;
            }

            let message = String::from_utf8_lossy(&buffer[..n]);
            let response = format!("{}!!!", message.to_uppercase());

            socket.write_all(response.as_bytes()).await.unwrap();
        });
    }
}
