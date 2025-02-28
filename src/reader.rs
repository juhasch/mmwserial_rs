use pyo3::prelude::*;
use serialport::SerialPort;
use std::time::Duration;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

use crate::types::{Header, RadarPacket, MAGIC_WORD};

// Helper function to convert serialport errors to PyErr
fn to_py_err(e: impl std::error::Error) -> PyErr {
    pyo3::exceptions::PyIOError::new_err(format!("Serial port error: {}", e))
}

#[pyclass]
pub struct RadarReader {
    port: Box<dyn SerialPort>,
    buffer: Vec<u8>,
}

#[pymethods]
impl RadarReader {
    #[new]
    pub fn new(port_name: &str) -> PyResult<Self> {
        let port = serialport::new(port_name, 115200 * 9)
            .timeout(Duration::from_millis(10))  // Shorter timeout
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(to_py_err)?;

        Ok(RadarReader { 
            port,
            buffer: Vec::with_capacity(4096),  // Pre-allocate buffer
        })
    }

    /// Read a complete radar packet
    pub fn read_packet(&mut self) -> PyResult<Option<RadarPacket>> {
        // Find magic word
        if !self.find_magic_word()? {
            return Ok(None);
        }

        // Read header
        let header = self.read_header()?;
        
        // Read data
        let data_len = header.total_packet_len as usize - 40; // 40 = magic(8) + header(32)
        self.buffer.resize(data_len, 0);
        
        // Try to read all data at once
        match self.port.read_exact(&mut self.buffer) {
            Ok(_) => Ok(Some(RadarPacket { 
                header, 
                data: self.buffer[..data_len].to_vec() 
            })),
            Err(e) => {
                // Clear any remaining data in the buffer
                if let Err(clear_err) = self.port.clear(serialport::ClearBuffer::Input) {
                    return Err(to_py_err(clear_err));
                }
                Err(to_py_err(e))
            }
        }
    }

    fn find_magic_word(&mut self) -> PyResult<bool> {
        let mut buffer = [0u8; 1];
        let mut matched = 0;
        let mut retries = 0;
        
        while retries < 3 {  // Try up to 3 times to find magic word
            for _ in 0..10000 {  // Search within each retry
                match self.port.read_exact(&mut buffer) {
                    Ok(_) => {
                        if buffer[0] == MAGIC_WORD[matched] {
                            matched += 1;
                            if matched == MAGIC_WORD.len() {
                                return Ok(true);
                            }
                        } else {
                            matched = if buffer[0] == MAGIC_WORD[0] { 1 } else { 0 };
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        continue;
                    }
                    Err(e) => {
                        return Err(to_py_err(e));
                    }
                }
            }
            retries += 1;
            // Clear input buffer and try again
            if let Err(e) = self.port.clear(serialport::ClearBuffer::Input) {
                return Err(to_py_err(e));
            }
        }
        
        Ok(false)
    }

    fn read_header(&mut self) -> PyResult<Header> {
        let mut header_data = vec![0u8; 32];
        self.port.read_exact(&mut header_data).map_err(to_py_err)?;

        let mut rdr = Cursor::new(header_data);
        
        Ok(Header {
            magic: MAGIC_WORD.to_vec(),
            version: rdr.read_u32::<LittleEndian>().unwrap(),
            total_packet_len: rdr.read_u32::<LittleEndian>().unwrap(),
            platform: rdr.read_u32::<LittleEndian>().unwrap(),
            frame_number: rdr.read_u32::<LittleEndian>().unwrap(),
            time_cpu_cycles: rdr.read_u32::<LittleEndian>().unwrap(),
            num_detected_obj: rdr.read_u32::<LittleEndian>().unwrap(),
            num_tlv: rdr.read_u32::<LittleEndian>().unwrap(),
        })
    }
} 