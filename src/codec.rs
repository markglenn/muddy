use std::ops::Index;

use tokio_util::codec::Decoder;

pub struct TelnetCodec;

pub enum TelnetCommand {
    Will(u8),
    Wont(u8),
    Do(u8),
    Dont(u8),
}

pub enum TelnetFrame {
    Command(TelnetCommand),
    Data(Vec<u8>),
}

impl Decoder for TelnetCodec {
    type Item = TelnetFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        match src[0] {
            0xFF => {
                if src.len() < 3 {
                    return Ok(None);
                }

                let command = src.split_to(3);
                let command = TelnetCommand::Will(command[2]);

                Ok(Some(TelnetFrame::Command(command)))
            }
            _ => {
                let pos = src.iter().position(|&b| b == 0xFF).unwrap_or(src.len());
                let data = src.split_to(pos).to_vec();

                Ok(Some(TelnetFrame::Data(data)))
            }
        }
    }
}
