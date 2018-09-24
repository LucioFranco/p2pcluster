use bytes::{BufMut, BytesMut};
use serde_json;
use std::io;
use tokio::codec::{Decoder, Encoder};

#[derive(Serialize, Deserialize)]
pub enum Msg {
    Connect,
}

pub struct MsgCodec;

impl Decoder for MsgCodec {
    type Item = Msg;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Msg>> {
        if buf.len() > 0 {
            let decode_msg = serde_json::from_slice(&buf[..])?;
            Ok(Some(decode_msg))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for MsgCodec {
    type Item = Msg;
    type Error = io::Error;

    fn encode(&mut self, data: Msg, buf: &mut BytesMut) -> io::Result<()> {
        let bytes = serde_json::to_vec(&data)?;
        buf.put(&bytes[..]);
        Ok(())
    }
}
