use std::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Sender, Receiver, TryRecvError},
    time::Duration,
    io::{Read, Write},
};
use anyhow::Result;
use log::{info, error};

pub struct KdctlServer {
    shutdown_rx: Receiver<()>,
}

impl KdctlServer {
    pub fn new() -> (Self, Sender<()>) {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let server = Self { shutdown_rx };
        (server, shutdown_tx)
    }

    pub fn run(&self, port: u16) -> Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        info!("TCP server listening on port {}", port);
        
        loop {
            match self.shutdown_rx.try_recv() {
                Ok(_) => {
                    info!("Shutting down TCP server");
                    return Ok(());
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    error!("Shutdown channel disconnected");
                    return Ok(());
                }
            }
            
            match listener.accept() {
                Ok((stream, addr)) => {
                    info!("Client connected from: {}", addr);
                    self.handle_client(stream)?;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }

    fn handle_client(&self, mut stream: TcpStream) -> Result<()> {
        let mut buf = [0u8; 1024];
        match stream.read(&mut buf) {
            Ok(n) => {
                let received = String::from_utf8_lossy(&buf[..n]);
                info!("Received: {}", received);
                
                if received.contains("ping") {
                    stream.write_all(b"pong")?;
                }
            }
            Err(e) => error!("Read error: {}", e),
        }
        Ok(())
    }
}