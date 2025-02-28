use pyo3::prelude::*;

mod types;
mod reader;

/// A Python module implemented in Rust.
#[pymodule]
fn mmwserial(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<reader::RadarReader>()?;
    Ok(())
}

pub use reader::RadarReader;
pub use types::*; 