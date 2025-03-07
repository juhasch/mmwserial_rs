use pyo3::prelude::*;
use serialport::SerialPort;
use std::time::{Duration, Instant};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::types::{Header, RadarPacket, MAGIC_WORD};

// Helper function to convert serialport errors to PyErr
fn to_py_err(e: impl std::error::Error) -> PyErr {
    pyo3::exceptions::PyIOError::new_err(format!("Serial port error: {}", e))
}

/// Read available bytes into the read buffer
fn read_available(port: &mut Box<dyn SerialPort>, read_buffer: &mut VecDeque<u8>) -> PyResult<usize> {
    let mut temp_buf = [0u8; 4096];  // Increased buffer size to handle full packets
    let mut total_read = 0;
    let start = Instant::now();
    
    // Try to read multiple times with a short timeout
    while start.elapsed() < Duration::from_millis(10) {  // Reduced read window for faster frames
        // Check for Python interrupt
        Python::with_gil(|py| py.check_signals())?;
        
        match port.read(&mut temp_buf) {
            Ok(n) if n > 0 => {
                read_buffer.extend(&temp_buf[..n]);
                total_read += n;
                // If we got a full buffer, immediately try to read more
                if n == temp_buf.len() {
                    continue;
                }
            }
            Ok(_) => {
                if total_read > 0 {
                    std::thread::sleep(Duration::from_micros(50));  // Even shorter delay
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if total_read > 0 {
                    std::thread::sleep(Duration::from_micros(50));  // Even shorter delay
                }
            }
            Err(e) => return Err(to_py_err(e)),
        }
    }
    
    Ok(total_read)
}

/// Read exact number of bytes from read buffer, waiting if necessary
fn read_exact_from_buffer(
    port: &mut Box<dyn SerialPort>,
    read_buffer: &mut VecDeque<u8>,
    buf: &mut [u8],
    timeout: Duration,
    debug: bool,
) -> PyResult<bool> {
    let start = Instant::now();
    let mut filled = 0;
    let mut consecutive_empty_reads = 0;
    let mut last_read_time = start;
    let mut last_filled = 0;
    let mut stall_time = start;
    
    // Calculate minimum completion ratio based on packet size
    let min_completion_ratio = if buf.len() <= 200 {
        0.95  // Small packets need 95% completion
    } else if buf.len() <= 300 {
        0.85  // Medium packets need 85% completion for faster processing
    } else {
        0.75  // Large packets can accept 75% completion for faster processing
    };

    while filled < buf.len() {
        // Check for Python interrupt
        Python::with_gil(|py| py.check_signals())?;
        
        // First try to fill from read buffer
        while filled < buf.len() && !read_buffer.is_empty() {
            buf[filled] = read_buffer.pop_front().unwrap();
            filled += 1;
            
            // Periodically check for interrupts during buffer processing
            if filled % 100 == 0 {
                Python::with_gil(|py| py.check_signals())?;
            }
        }
        
        if filled == buf.len() {
            return Ok(true);
        }

        // Update progress tracking
        if filled > last_filled {
            last_filled = filled;
            stall_time = Instant::now();
        }

        // If we've made good progress but stalled, consider accepting
        let completion_ratio = filled as f32 / buf.len() as f32;
        if completion_ratio >= min_completion_ratio && // Met size-based threshold
           stall_time.elapsed() > Duration::from_millis(5) && // Further reduced stall time
           last_read_time.elapsed() > Duration::from_millis(3) { // Further reduced wait time
            // Zero-fill remaining bytes
            for i in filled..buf.len() {
                buf[i] = 0;
            }
            return Ok(true);
        }
        
        // Check timeout
        if start.elapsed() > timeout {
            if debug {
                println!("Buffer read timeout after filling {}/{} bytes", filled, buf.len());
            }
            return Ok(false);
        }
        
        // Read more data
        let bytes_read = read_available(port, read_buffer)?;
        if bytes_read > 0 {
            last_read_time = Instant::now();
            consecutive_empty_reads = 0;
        } else {
            consecutive_empty_reads += 1;
            if consecutive_empty_reads >= 3 {
                std::thread::sleep(Duration::from_micros(250));  // Shorter sleep
            }
        }
    }
    
    Ok(true)
}

/// RadarReader structure for handling serial communication with the radar
#[pyclass(unsendable)]
pub struct RadarReader {
    port: Box<dyn SerialPort>,
    buffer: Vec<u8>,
    read_buffer: VecDeque<u8>,
    header_buffer: Vec<u8>,
    debug: bool,
    last_frame_time: Option<Instant>,
    _unsendable: Rc<()>,
}

#[pymethods]
impl RadarReader {
    #[new]
    #[pyo3(signature = (port_name, baudrate=1036800, debug=None))]
    pub fn new(port_name: &str, baudrate: u32, debug: Option<bool>) -> PyResult<Self> {
        let port = serialport::new(port_name, baudrate)  // Use the provided baudrate
            .timeout(Duration::from_millis(1))  // Very short timeout for more responsive reads
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(to_py_err)?;

        Ok(RadarReader { 
            port,
            buffer: Vec::with_capacity(8192),     // Increased buffer capacity
            read_buffer: VecDeque::with_capacity(8192), // Increased buffer capacity
            header_buffer: vec![0u8; 32],         // Fixed header buffer
            debug: debug.unwrap_or(false),
            last_frame_time: None,
            _unsendable: Rc::new(()),
        })
    }

    /// Read a complete radar packet
    pub fn read_packet(&mut self) -> PyResult<Option<RadarPacket>> {
        let start_time = Instant::now();
        let global_timeout = Duration::from_millis(150);  // Reduced global timeout for faster frames
        
        // Find magic word
        if !self.find_magic_word()? {
            return Ok(None);
        }

        // Check global timeout
        if start_time.elapsed() > global_timeout {
            if self.debug {
                println!("Global timeout exceeded while finding magic word");
            }
            return Ok(None);
        }

        // Read header
        if !read_exact_from_buffer(&mut self.port, &mut self.read_buffer, &mut self.header_buffer, 
                                 Duration::from_millis(25), self.debug)? {  // Reduced header timeout
            if self.debug {
                println!("Failed to read header");
            }
            return Ok(None);
        }

        // Check global timeout
        if start_time.elapsed() > global_timeout {
            if self.debug {
                println!("Global timeout exceeded while reading header");
            }
            return Ok(None);
        }

        let header = {
            let mut rdr = Cursor::new(&self.header_buffer);
            Header {
                magic: MAGIC_WORD.to_vec(),
                version: rdr.read_u32::<LittleEndian>().unwrap(),
                total_packet_len: rdr.read_u32::<LittleEndian>().unwrap(),
                platform: rdr.read_u32::<LittleEndian>().unwrap(),
                frame_number: rdr.read_u32::<LittleEndian>().unwrap(),
                time_cpu_cycles: rdr.read_u32::<LittleEndian>().unwrap(),
                num_detected_obj: rdr.read_u32::<LittleEndian>().unwrap(),
                num_tlv: rdr.read_u32::<LittleEndian>().unwrap(),
            }
        };

        if self.debug {
            println!("Header read: total_len={}, frame={}, objs={}, tlv={}", 
                    header.total_packet_len, header.frame_number,
                    header.num_detected_obj, header.num_tlv);
        }
        
        // Validate header
        if !self.validate_header(&header) {
            self.read_buffer.clear();
            return Ok(None);
        }
        
        // Calculate expected data length
        let expected_data_len = header.total_packet_len as usize - 40; // 40 = magic(8) + header(32)
        
        // Adjust timeout based on packet size, but ensure we don't exceed global timeout
        let remaining_time = global_timeout.saturating_sub(start_time.elapsed());
        let packet_timeout = Duration::from_millis(50 + (expected_data_len as u64 / 100) * 10);
        let timeout = std::cmp::min(remaining_time, packet_timeout);
        
        // Resize buffer and read data
        self.buffer.resize(expected_data_len, 0);
        if !read_exact_from_buffer(&mut self.port, &mut self.read_buffer, &mut self.buffer, 
                                 timeout, self.debug)? {
            if self.debug {
                println!("Failed to read packet data");
            }
            return Ok(None);
        }

        self.last_frame_time = Some(start_time);
        Ok(Some(RadarPacket { 
            header, 
            data: self.buffer[..expected_data_len].to_vec() 
        }))
    }

    fn validate_header(&self, header: &Header) -> bool {
        // Basic sanity checks with more detailed validation
        let valid = header.total_packet_len >= 40 && // Must be at least magic + header size
                   header.total_packet_len <= 4096 && // Reasonable max size
                   header.total_packet_len % 32 == 0 && // Must be multiple of 32
                   header.num_detected_obj <= 100 && // Reasonable max objects
                   header.num_tlv <= 10; // Reasonable max TLV sections

        if !valid && self.debug {
            println!("Invalid header values:");
            println!("  total_packet_len: {} (valid: {} <= x <= {} && x % 32 == 0)", 
                    header.total_packet_len, 40, 4096);
            println!("  num_detected_obj: {} (valid: <= 100)", header.num_detected_obj);
            println!("  num_tlv: {} (valid: <= 10)", header.num_tlv);
            println!("  frame_number: {}", header.frame_number);
            println!("  platform: {}", header.platform);
            println!("  version: {}", header.version);
            
            // Additional validation details
            if header.total_packet_len < 40 {
                println!("  ERROR: Packet length too small (min 40 bytes needed)");
            }
            if header.total_packet_len % 32 != 0 {
                println!("  ERROR: Packet length not multiple of 32");
            }
            if header.total_packet_len > 4096 {
                println!("  ERROR: Packet length exceeds max size");
            }
        }

        valid
    }

    fn find_magic_word(&mut self) -> PyResult<bool> {
        let start_time = Instant::now();
        let mut matched = 0;
        let mut total_bytes = 0;
        let mut discarded_bytes = 0;
        let frame_period = Duration::from_millis(100);  // Updated frame period to 100ms
        
        // If we have a last frame time and it's too early for next frame, wait
        if let Some(last_time) = self.last_frame_time {
            let elapsed = start_time.duration_since(last_time);
            if elapsed < frame_period.mul_f32(0.65) {  // Reduced wait time to 65% of frame period
                std::thread::sleep(Duration::from_millis(2));  // Shorter sleep for faster response
                return Ok(false);
            }
        }

        // Try to read until we either find the magic word or timeout
        while start_time.elapsed() < frame_period {
            // Read available data if buffer is getting low
            if self.read_buffer.len() < MAGIC_WORD.len() {
                read_available(&mut self.port, &mut self.read_buffer)?;
            }
            
            // Try to match magic word from buffer
            while let Some(&byte) = self.read_buffer.front() {
                total_bytes += 1;
                
                if byte == MAGIC_WORD[matched] {
                    matched += 1;
                    self.read_buffer.pop_front();
                    
                    if matched == MAGIC_WORD.len() {
                        if self.debug {
                            if discarded_bytes > 0 {
                                println!("Discarded {} bytes before magic word", discarded_bytes);
                            }
                            println!("Found magic word after {} bytes in {:?}", 
                                    total_bytes, start_time.elapsed());
                        }
                        return Ok(true);
                    }
                } else {
                    // If we were matching but failed, we need to recheck the current byte
                    if matched > 0 {
                        matched = if byte == MAGIC_WORD[0] { 1 } else { 0 };
                    } else {
                        // Only discard the byte if we're not matching at all
                        self.read_buffer.pop_front();
                        discarded_bytes += 1;
                    }
                }
            }

            // Small sleep to prevent tight loop
            std::thread::sleep(Duration::from_micros(100));
        }
        
        if self.debug {
            println!("Failed to find magic word after {:?} (discarded {} bytes, buffer size {})", 
                    start_time.elapsed(), discarded_bytes, self.read_buffer.len());
        }
        
        Ok(false)
    }
} 