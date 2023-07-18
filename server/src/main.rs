mod commands;

use crate::commands::process_client_request;
use futures::{SinkExt, StreamExt};
use lib::codec::MessageCoder;
use std::env;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_env();

    let addr = env::args()
        .skip(1)
        .next()
        .unwrap_or("127.0.0.1:8000".to_owned());
    let listener = TcpListener::bind(&addr).await?;
    log::info!("server listening on :{}", addr);
    loop {
        let (socket, addr) = listener.accept().await?;
        let _ = socket.nodelay();
        handle_client(socket).await.unwrap();
        log::info!("accepted connection from: {}", addr);
    }
}

fn init_env() {
    env::set_var("RUST_LOG", "server=trace");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}

async fn handle_client(socket: TcpStream) -> anyhow::Result<()> {
    let (mut writer, mut reader) = MessageCoder.framed(socket).split();
    tokio::spawn(async move {
        while let Some(val) = reader.next().await {
            let reply = val.and_then(|x| {
                process_client_request(x)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
            });
            match reply {
                Ok(reply) => {
                    let _ = writer.send(reply).await;
                }
                Err(err) => {
                    log::error!("meet up with an error->{:?}", err);
                }
            };
        }
    });
    Ok(())
}
