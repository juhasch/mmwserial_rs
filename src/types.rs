use pyo3::prelude::*;
use std::fmt;

/// Message types used in TI mmWave radar output
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    DetectedPoints = 1,
    RangeProfile = 2,
    NoiseProfile = 3,
    AzimutStaticHeatMap = 4,
    RangeDopplerHeatMap = 5,
    Stats = 6,
    DetectedPointsSideInfo = 7,
    AzimutElevationStaticHeatMap = 8,
    TemperatureStats = 9,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Magic word used to identify start of packet
pub const MAGIC_WORD: [u8; 8] = [0x02, 0x01, 0x04, 0x03, 0x06, 0x05, 0x08, 0x07];

/// Header structure for radar packets
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Header {
    #[pyo3(get)]
    pub magic: Vec<u8>,
    #[pyo3(get)]
    pub version: u32,
    #[pyo3(get)]
    pub total_packet_len: u32,
    #[pyo3(get)]
    pub platform: u32,
    #[pyo3(get)]
    pub frame_number: u32,
    #[pyo3(get)]
    pub time_cpu_cycles: u32,
    #[pyo3(get)]
    pub num_detected_obj: u32,
    #[pyo3(get)]
    pub num_tlv: u32,
}

#[pymethods]
impl Header {
    #[new]
    pub fn new() -> Self {
        Header {
            magic: MAGIC_WORD.to_vec(),
            version: 0,
            total_packet_len: 0,
            platform: 0,
            frame_number: 0,
            time_cpu_cycles: 0,
            num_detected_obj: 0,
            num_tlv: 0,
        }
    }
}

/// TLV (Type-Length-Value) header structure
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct TlvHeader {
    #[pyo3(get)]
    pub typ: u32,
    #[pyo3(get)]
    pub length: u32,
}

#[pymethods]
impl TlvHeader {
    #[new]
    pub fn new() -> Self {
        TlvHeader {
            typ: 0,
            length: 0,
        }
    }
}

/// Complete radar packet structure
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct RadarPacket {
    #[pyo3(get)]
    pub header: Header,
    #[pyo3(get)]
    pub data: Vec<u8>,
}

#[pymethods]
impl RadarPacket {
    #[new]
    pub fn new(header: Header, data: Vec<u8>) -> Self {
        RadarPacket { header, data }
    }
} 