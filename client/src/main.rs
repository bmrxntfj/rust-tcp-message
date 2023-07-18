use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_util::codec::Decoder;

use lib::codec::MessageCoder;
use lib::message::{Message,BusinessRequest, BusinessResponse};
use tokio::sync::oneshot::Sender;

lazy_static! {
    static ref RESPONSE_SENDER_MAP: Arc<RwLock<HashMap<u16, Sender<Message>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_env();

    let socket = TcpStream::connect("127.0.0.1:8000").await?;
    let _ = socket.nodelay();
    let mut id = 1u16;
    let (mut writer, mut reader) = MessageCoder.framed(socket).split();
    tokio::spawn(async move {
        while let Some(reply) = reader.next().await {
            match reply {
                Ok(reply) => {
                    //fast release rwlock by scope:{}
                    {
                        let mut mr = RESPONSE_SENDER_MAP.write().unwrap();
                        //get `sender`'s ownship by remove of hashmap
                        let m = mr.remove(&reply.id).unwrap();
                        let _ = m.send(reply);
                    }
                }
                Err(err) => {
                    log::error!("{:?}", err);
                }
            };
        }
    });

    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read");
        let input = input.trim().to_owned();
        let r = serde_json::to_string(&BusinessRequest {
            wait: 3,
            order_no: input,
        })?;
        let message = Message::new(id, Bytes::copy_from_slice(r.as_bytes()));
        writer.feed(message).await?;
        writer.flush().await?;
        let (tx, rx) = tokio::sync::oneshot::channel::<Message>();
        //fast release rwlock by scope:{}
        {
            let mut mw = RESPONSE_SENDER_MAP.write().unwrap();
            mw.insert(id, tx);
        }

        let time_result = tokio::time::timeout(Duration::from_secs(3), rx).await;
        match time_result {
            Ok(Ok(reply)) => {
                let res = serde_json::from_slice::<BusinessResponse>(reply.body.as_ref())?;
                log::trace!("response:{:?}", res);
            }
            Ok(Err(err)) => log::error!("error:{:?}", err),
            Err(err) => log::error!("timeout:{:?}", err),
        }
        id += 1;
    }
}

fn init_env() {
    env::set_var("RUST_LOG", "client=trace");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}
