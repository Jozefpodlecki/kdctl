use std::{
    net::TcpStream,
    io::{Read, Write},
};
use anyhow::Result;
use rmp_serde::{to_vec, from_slice};
use byteorder::{WriteBytesExt, ReadBytesExt, LittleEndian};

pub struct KdctlStream(TcpStream);

impl KdctlStream {
    pub fn connect(addr: &str) -> Result<Self> {
        Ok(Self(TcpStream::connect(addr)?))
    }

    pub fn send<T: serde::Serialize>(&mut self, msg: &T) -> Result<()> {
        let bytes = to_vec(msg)?;
        self.0.write_u32::<LittleEndian>(bytes.len() as u32)?;
        self.0.write_all(&bytes)?;
        self.0.flush()?;
        Ok(())
    }

    pub fn recv<T: serde::de::DeserializeOwned>(&mut self) -> Result<T> {
        let len = self.0.read_u32::<LittleEndian>()?;
        let mut buf = vec![0u8; len as usize];
        self.0.read_exact(&mut buf)?;
        let msg = from_slice(&buf)?;
        Ok(msg)
    }
}