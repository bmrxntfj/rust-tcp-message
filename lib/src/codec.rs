use bytes::{Buf, Bytes, BytesMut};
use std::mem::size_of;
use tokio_util::codec::{Decoder, Encoder};

use crate::message::Message;

pub struct MessageCoder;

impl Encoder<Message> for MessageCoder {
    type Error = std::io::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> std::io::Result<()> {
        let idbuf = msg.id.to_le_bytes();
        let size = (idbuf.len() + msg.body.len()) as u16;
        let sizebuf = size.to_le_bytes();

        buf.reserve(sizebuf.len() + idbuf.len() + msg.body.len());
        buf.extend(sizebuf);
        buf.extend(idbuf);
        buf.extend(msg.body.iter());

        Ok(())
    }
}

const MAX: u16 = u16::MAX;

impl Decoder for MessageCoder {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> std::io::Result<Option<Message>> {
        let size = size_of::<u16>();
        if buf.len() > size {
            let len = u16::from_le_bytes(buf[0..size].try_into().unwrap()) as usize;
            if len > MAX as usize {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Frame of length {} is too large.", len),
                ));
            }
            if buf.len() >= size + len {
                let messagebuf: &[u8] = &buf[size..size + len];

                let message = Message::new(
                    u16::from_le_bytes(messagebuf[0..size].try_into().unwrap()),
                    Bytes::copy_from_slice(&messagebuf[size..]),
                );

                buf.advance(size + len);
                Ok(Some(message))
            } else {
                buf.reserve(size + len - buf.len());
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
