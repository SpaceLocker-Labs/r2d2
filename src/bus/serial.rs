//! # Serial Bus
//!
//! Serial bus implementation.

use std::time::{Duration, Instant};

use embedded_hal_nb::nb::block;
use embedded_hal_nb::serial::{Read, Write};
use linux_embedded_hal::serialport::DataBits;
use linux_embedded_hal::serialport::Parity;
use linux_embedded_hal::serialport::StopBits;
use linux_embedded_hal::serialport::{self, SerialPort};
use linux_embedded_hal::Serial;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// SerialBus(port_name="/dev/ttyUSB0", baud_rate=9600, byte_size=8, parity="None", stop_bits=1)
///
/// Serial bus definition.
#[pyclass]
pub struct SerialBus {
    /// The port name
    pub port_name: String,
    /// The baud rate
    pub baud_rate: u32,
    /// The byte size
    pub byte_size: DataBits,
    /// The parity
    pub parity: Parity,
    /// The stop bits
    pub stop_bits: StopBits,
    /// The internal bus
    inner: Option<Serial>,
}

#[pymethods]
impl SerialBus {
    /// Creates a new serial bus.
    #[new]
    #[pyo3(signature = (port_name, baud_rate, byte_size, parity, stop_bits))]
    fn new(
        port_name: String,
        baud_rate: u32,
        byte_size: Option<u8>,
        parity: Option<String>,
        stop_bits: Option<u8>,
    ) -> PyResult<Self> {
        let byte_size = match byte_size {
            Some(byte_size) => DataBits::try_from(byte_size).unwrap(),
            None => DataBits::Eight,
        };

        let parity = match parity {
            Some(parity) => match parity.as_str() {
                "None" => Parity::None,
                "Odd" => Parity::Odd,
                "Even" => Parity::Even,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid parity value: {}",
                        parity
                    )))
                }
            },
            None => Parity::None,
        };

        let stop_bits = match stop_bits {
            Some(stop_bits) => match StopBits::try_from(stop_bits) {
                Ok(value) => value,
                Err(_) => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid stop bits value: {}",
                        stop_bits
                    )))
                }
            },
            None => StopBits::One,
        };

        Ok(SerialBus {
            port_name,
            baud_rate,
            byte_size,
            parity,
            stop_bits,
            inner: None,
        })
    }

    /// Opens the serial bus.
    #[pyo3(signature = ())]
    fn open(&mut self) -> PyResult<()> {
        if self.inner.is_some() {
            return Ok(());
        }

        let builder = serialport::new(&self.port_name, self.baud_rate)
            .data_bits(self.byte_size)
            .parity(self.parity)
            .stop_bits(self.stop_bits)
            .flow_control(serialport::FlowControl::None);
        self.inner = match Serial::open_from_builder(builder) {
            Ok(port) => Some(port),
            Err(e) => {
                return Err(PyValueError::new_err(format!(
                    "Failed to open serial port: {:?}",
                    e
                )))
            }
        };
        Ok(())
    }

    /// Closes the serial bus.
    #[pyo3(signature = ())]
    fn close(&mut self) -> PyResult<()> {
        self.inner = None;
        Ok(())
    }

    /// Reads data from the serial bus.
    #[pyo3(signature = (length, timeout))]
    fn read(&mut self, length: u16, timeout: f32) -> PyResult<Vec<u8>> {
        let mut data = Vec::new();
        let start_time = Instant::now();
        let timeout_duration = Duration::from_secs_f32(timeout);

        let port = match self.inner.as_mut() {
            Some(port) => port,
            None => return Err(PyValueError::new_err(format!("Port not open"))),
        };

        for _ in 0..length {
            if start_time.elapsed() > timeout_duration {
                break;
            }

            match block!(port.read()) {
                Ok(byte) => data.push(byte),
                Err(e) => {
                    return Err(PyValueError::new_err(format!(
                        "Failed to read data: {:?}",
                        e
                    )))
                }
            }
        }

        Ok(data)
    }

    /// Writes data to the serial bus.
    #[pyo3(signature = (data))]
    fn write(&mut self, data: Vec<u8>) -> PyResult<()> {
        let inner = match self.inner.as_mut() {
            Some(port) => port,
            None => return Err(PyValueError::new_err(format!("Port not open"))),
        };

        for byte in data {
            match block!(inner.write(byte)) {
                Ok(_) => (),
                Err(e) => {
                    return Err(PyValueError::new_err(format!(
                        "Failed to write data: {:?}",
                        e
                    )))
                }
            }
        }
        Ok(())
    }

    /// Flushes the serial bus.
    #[pyo3(signature = ())]
    fn flush(&mut self) -> PyResult<()> {
        let inner = match self.inner.as_mut() {
            Some(port) => port,
            None => return Err(PyValueError::new_err(format!("Port not open"))),
        };

        match inner.flush() {
            Ok(_) => Ok(()),
            Err(e) => Err(PyValueError::new_err(format!(
                "Failed to flush data: {:?}",
                e
            ))),
        }
    }

    /// Returns the number of bytes waiting in the input buffer.
    #[pyo3(signature = ())]
    fn input_waiting(&mut self) -> PyResult<u32> {
        let port = match &self.inner {
            Some(port) => port,
            None => return Err(PyValueError::new_err(format!("Port not open"))),
        };

        match port.0.bytes_to_read() {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(PyValueError::new_err(format!(
                "Failed to get bytes in input buffer: {:?}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::thread::sleep;
    use std::time::Duration;

    use super::SerialBus;

    #[test]
    fn test_serial_read() {
        let (mut master, _slave, name) =
            openpty::openpty(None, None, None).expect("Creating pty failed");
        let mut bus =
            SerialBus::new(name, 9600, Some(8), Some("None".to_string()), Some(1)).unwrap();

        bus.open().expect("Opening bus failed");

        master
            .write_all("Hello, world!".as_bytes())
            .expect("Writing to bus failed");

        // Wait for the data to be written
        master.flush().expect("Flushing master failed");
        sleep(Duration::from_millis(50));

        let data = bus.read(13, 10.0).expect("Reading from bus failed");
        assert_eq!("Hello, world!".as_bytes(), data.as_slice());

        bus.close().expect("Closing bus failed");
    }

    #[test]
    fn test_serial_write() {
        let (mut master, _slave, name) =
            openpty::openpty(None, None, None).expect("Creating pty failed");
        let mut bus =
            SerialBus::new(name, 9600, Some(8), Some("None".to_string()), Some(1)).unwrap();

        bus.open().expect("Opening bus failed");

        let data = "Hello, world!".as_bytes().to_vec();
        bus.write(data).expect("Writing to bus failed");

        let mut buffer = [0u8; 13];
        master
            .read(buffer.as_mut())
            .expect("Reading from bus failed");

        assert_eq!("Hello, world!".as_bytes(), buffer);

        bus.close().expect("Closing bus failed");
    }

    #[test]
    fn test_serial_input_waiting() {
        let (mut master, _slave, name) =
            openpty::openpty(None, None, None).expect("Creating pty failed");
        let mut bus =
            SerialBus::new(name, 9600, Some(8), Some("None".to_string()), Some(1)).unwrap();

        bus.open().expect("Opening bus failed");

        master
            .write_all("Hello, world!".as_bytes())
            .expect("Writing to bus failed");

        // Wait for the data to be written
        master.flush().expect("Flushing master failed");
        sleep(Duration::from_millis(50));

        let input_buffer_len = bus.input_waiting().expect("Getting input waiting failed");
        assert_eq!(13, input_buffer_len);

        bus.close().expect("Closing bus failed");
    }
}
