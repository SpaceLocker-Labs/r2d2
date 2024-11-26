//! # I2C Bus
//! I2C bus implementation.

use embedded_hal::i2c::{I2c, Operation};
use linux_embedded_hal::I2cdev;
use pyo3::{exceptions::PyValueError, prelude::*};

/// I2CBus(address=0x01, path="/dev/i2c-1")
///
/// I2C bus definition.
#[pyclass]
pub struct I2CBus {
    /// The device address
    pub address: u8,
    /// The device path
    pub path: String,
    /// The inner I2C bus
    inner: I2cdev,
}

#[pymethods]
impl I2CBus {
    /// Creates a new I2C bus.
    #[new]
    #[pyo3(signature = (address, path))]
    fn new(address: u8, path: String) -> PyResult<Self> {
        let inner = match I2cdev::new(&path) {
            Ok(bus) => bus,
            Err(_) => {
                return Err(PyValueError::new_err(format!(
                    "Failed to open I2C bus at {}",
                    path
                )))
            }
        };

        Ok(I2CBus {
            address,
            path,
            inner,
        })
    }

    /// Reads data from the I2C bus.
    #[pyo3(signature = (length))]
    fn read(&mut self, length: u16) -> PyResult<Vec<u8>> {
        let mut read_buffer = vec![0u8; length as usize];
        let mut ops = [Operation::Read(&mut read_buffer)];

        match self.inner.transaction(self.address, &mut ops) {
            Ok(_) => Ok(read_buffer.as_mut_slice().to_vec()),
            Err(_) => Err(PyValueError::new_err("Failed to read from I2C bus")),
        }
    }

    /// Writes data to the I2C bus.
    #[pyo3(signature = (data))]
    fn write(&mut self, mut data: Vec<u8>) -> PyResult<()> {
        let mut ops = [Operation::Write(data.as_mut_slice())];

        match self.inner.transaction(self.address, &mut ops) {
            Ok(_) => Ok(()),
            Err(_) => Err(PyValueError::new_err("Failed to write to I2C bus")),
        }
    }
}
