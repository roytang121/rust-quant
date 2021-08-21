use std::error::Error;

use tokio::net::TcpStream;

use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

async fn create_websocket(
    url: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
    let (socket, response) = connect_async(url::Url::parse(url)?).await?;
    println!("Connected to conn");
    println!("Response HTTP code {}", response.status());
    for (header, value) in response.headers() {
        println!("* {}: {}", header, value.to_str()?);
    }
    Ok(socket)
}

pub async fn connect_wss_async(
    url: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
    let socket = create_websocket(url).await?;
    // socket.send(Message::Text(init_message)).await?;
    Ok(socket)
}
