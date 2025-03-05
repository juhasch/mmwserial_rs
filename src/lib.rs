use pyo3::prelude::*;

mod types;
mod reader;
mod udp;

/// A Python module implemented in Rust.
#[pymodule]
fn mmwserial(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("RadarReader", _py.get_type::<reader::RadarReader>())?;
    m.add_class::<udp::UDPReader>()?;
    Ok(())
}

pub use reader::RadarReader;
pub use types::*;
pub use udp::UDPReader; 