use rmp_serde::to_vec;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Identify {
        client_name: String,
        version: String,
        capabilities: Vec<String>,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Shutdown
}

pub struct Frame(pub u32, pub Vec<u8>);

impl ClientMessage {
    pub fn to_frame(&self) -> Result<Frame, rmp_serde::encode::Error> {
        let data = to_vec(self)?;
        Ok(Frame(data.len() as u32, data))
    }
}