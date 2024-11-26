mod bus;
mod gpio;

use pyo3::prelude::*;

/// An I/O device communication module written in Rust.
#[pymodule]
fn r2d2(m: &Bound<'_, PyModule>) -> PyResult<()> {
    bus::init_pymodule(m)?;
    m.add_class::<gpio::GPIO>()?;
    Ok(())
}
