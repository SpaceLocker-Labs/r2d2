//! # Bus
//! 
//! Bus module.

use pyo3::prelude::*;

mod i2c;
mod serial;

pub(crate) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<serial::SerialBus>()?;
    module.add_class::<i2c::I2CBus>()?;

    Ok(())
}