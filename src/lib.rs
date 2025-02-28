use pyo3::prelude::*;

mod types;
mod reader;

/// A Python module implemented in Rust.
#[pymodule]
fn mmwserial(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("RadarReader", _py.get_type::<reader::RadarReader>())?;
    Ok(())
}

pub use reader::RadarReader;
pub use types::*; 