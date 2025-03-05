use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use thiserror::Error;
use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use socket2::{Socket, Domain, Protocol, Type};

#[derive(Error, Debug)]
pub enum UDPError {
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
    #[error("Timeout while reading frame")]
    Timeout,
    #[error("Incomplete frame read: expected {expected} bytes, got {received}")]
    IncompleteFrame { expected: usize, received: usize },
}

impl From<UDPError> for PyErr {
    fn from(err: UDPError) -> PyErr {
        match err {
            UDPError::SocketError(e) => PyIOError::new_err(e.to_string()),
            UDPError::Timeout => PyIOError::new_err("Timeout while reading frame"),
            UDPError::IncompleteFrame { expected, received } => {
                PyIOError::new_err(format!("Incomplete frame read: expected {} bytes, got {}", expected, received))
            }
        }
    }
}

#[pyclass]
pub struct UDPReader {
    socket: UdpSocket,
    frame_size: usize,
    timeout: Duration,
}

#[pymethods]
impl UDPReader {
    #[new]
    pub fn new(interface: &str, port: u16, frame_size: usize, timeout_ms: u64) -> PyResult<Self> {
        let addr = format!("{}:{}", interface, port).parse::<SocketAddr>()
            .map_err(|e| UDPError::SocketError(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;
        
        // Create socket with socket2 for more control
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(UDPError::SocketError)?;
            
        // Set socket options
        socket.set_reuse_address(true)
            .map_err(UDPError::SocketError)?;
            
        // Set receive buffer size (default is usually 65536)
        socket.set_recv_buffer_size(65536)
            .map_err(UDPError::SocketError)?;
            
        // Bind the socket
        socket.bind(&addr.into())
            .map_err(UDPError::SocketError)?;
            
        // Convert to std::net::UdpSocket
        let socket: UdpSocket = socket.into();
        
        // Set read timeout
        socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))
            .map_err(UDPError::SocketError)?;
        
        Ok(Self {
            socket,
            frame_size,
            timeout: Duration::from_millis(timeout_ms),
        })
    }

    pub fn read_frame(&self) -> PyResult<Vec<u8>> {
        let mut buffer = vec![0u8; self.frame_size];
        let received = self.socket.recv(&mut buffer)
            .map_err(UDPError::SocketError)?;
        
        if received != self.frame_size {
            return Err(UDPError::IncompleteFrame {
                expected: self.frame_size,
                received,
            }.into());
        }
        
        Ok(buffer)
    }

    pub fn read_frames(&self, num_frames: usize) -> PyResult<Vec<Vec<u8>>> {
        let mut frames = Vec::with_capacity(num_frames);
        
        for _ in 0..num_frames {
            match self.read_frame() {
                Ok(frame) => frames.push(frame),
                Err(e) => {
                    if e.to_string().contains("Timeout") {
                        break;
                    }
                    return Err(e);
                }
            }
        }
        
        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_reader() {
        let reader = UDPReader::new("127.0.0.1", 12345, 1024, 1000).unwrap();
        // Add more tests as needed
    }
} 