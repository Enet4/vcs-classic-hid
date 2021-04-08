//! VCS Classic Joystick HID library.
//!
//! This crate uses `hidapi`
//! for finding and using Atari VCS classic joysticks.
//!
//! With this crate, an assortment of facilities are provided
//! for reading the current state of the device and,
//! more importantly, send force feedback and LED manipulation messages.
//!
//! ## Finding a device
//!
//! The functions [open], [open_serial], and [open_all]
//! are helper functions for opening devices for access to a classic controller
//! via [`hidapi`](hidapi).
//! The result provides an [`HidDevice`](hidapi::HidDevice).
//!
//! ```no_run
//! # fn main() -> Result<(), hidapi::HidError> {
//! let device = vcs_classic_hid::open()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using the device
//!
//! Raw access to the HID device can still be done
//! through the interface provided,
//! but a set of facilities are provided through
//! the implemented [`Device`] trait
//! and additional functions and data types.
//!
//! ### Receiving input state
//!
//! A [State] value represents a possible input state
//! of the classic controller,
//! and can be created from a device report via
//! [`State::from_report`](crate::State::from_report).
//! However, this is not very convenient, and may return stale input
//!
//! The function [process_input] handles all input state events in queue
//! and returns a `State` instance.
//!
//! ```no_run
//! # fn main() -> Result<(), hidapi::HidError> {
//! let mut device = vcs_classic_hid::open()?;
//! match vcs_classic_hid::process_input(&mut device)? {
//!     Some(state) => {
//!         // use state
//!         let _fuji_button_down = state.button_fuji;
//!     }
//!     None => {
//!         // no user input
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! ### Changing LED state
//!
//! Both the light on the Fuji button and the ring of 24 LEDs
//! can be manipulated programmatically.
//! To set an exact state of the LEDs,
//! create an [`LedReport`] and send it to the device.
//!
//! For example, to light up all LEDs in the controller's ring to the maximum:
//!
//! ```no_run
//! use vcs_classic_hid::Device;
//! 
//! # fn main() -> Result<(), hidapi::HidError> {
//! # let mut device = vcs_classic_hid::open()?;
//! let led = vcs_classic_hid::LedReport::filled(0xFF);
//! Device::write(&mut device, led)?;
//! # Ok(())
//! # }
//! ```
//!
//! To cancel LED manipulation and let the controller itself manipulate them
//! based on user input
//! (which is the default behavior), use [`reset_leds`](Device::reset_leds):
//!
//! ```no_run
//! use vcs_classic_hid::Device;
//! 
//! # fn main() -> Result<(), hidapi::HidError> {
//! # let mut device = vcs_classic_hid::open()?;
//! device.reset_leds()?;
//! # Ok(())
//! # }
//! ```
//!
use std::ffi::CStr;

pub use hidapi;
use hidapi::{HidApi, HidDevice};

pub mod force_feedback;
pub mod led;
pub mod input;

pub use force_feedback::FfReport;
pub use led::LedReport;
pub use input::{State, StickPosition, process_input};

/// Generic interface for human interaction devices.
pub trait Device {
    /// The type used for errors
    type Error;

    /// Set or unset blocking mode
    fn set_blocking(&mut self, blocking: bool) -> Result<(), Self::Error>;

    /// Read a report into the given array,
    /// returns the number of bytes read.
    fn read(&mut self, out: &mut [u8]) -> Result<usize, Self::Error>;

    /// Write a report to the device,
    /// returns the number of bytes effectively written.
    ///
    /// **Safety:** the operation is not memory unsafe,
    /// but can still cause catastrophic problems to the device
    /// depending on the data passed.
    fn write<T>(&mut self, data: T) -> Result<usize, Self::Error>
    where
        T: AsRef<[u8]>;

    /// Write a report which disables LED manipulation
    /// in the VCS classic controller.
    fn reset_leds(&mut self) -> Result<(), Self::Error> {
        self.write(&[2, 0, 0, 0]).map(|_| ())
    }
}

impl<D> Device for &mut D
where
    D: Device
{
    type Error = D::Error;

    fn set_blocking(&mut self, blocking: bool) -> Result<(), Self::Error> {
        (**self).set_blocking(blocking)
    }

    fn read(&mut self, out: &mut [u8]) -> Result<usize, Self::Error> {
        (**self).read(out)
    }

    fn write<T>(&mut self, data: T) -> Result<usize, Self::Error>
    where
        T: AsRef<[u8]>,
    {
        (**self).write(data)
    }
}

impl Device for HidDevice {
    type Error = hidapi::HidError;

    fn set_blocking(&mut self, blocking: bool) -> Result<(), Self::Error> {
        HidDevice::set_blocking_mode(self, blocking)
    }

    fn read(&mut self, out: &mut [u8]) -> Result<usize, Self::Error> {
        HidDevice::read(self, out)
    }

    fn write<T>(&mut self, data: T) -> Result<usize, Self::Error>
    where
        T: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let k = HidDevice::write(self, data)?;
        if k != data.len() {
            eprintln!("Expected to write {} bytes, but wrote {}", data.len(), k);
        }
        Ok(k)
    }
}

const VENDOR_ID: u16 = 0x3250;
const PRODUCT_ID: u16 = 0x1001;

/// Inspect the list of devices available
/// and open the first VCS classic controller device found.
pub fn open() -> Result<hidapi::HidDevice, hidapi::HidError> {
    let api = HidApi::new()?;
    api.open(VENDOR_ID, PRODUCT_ID)
}

/// Inspect the list of devices available
/// and open a classic controller device by path.
///
/// **Safety:** The function does not check whether the device
/// behind the given path is actually the classic controller.
pub fn open_path(device_path: &CStr) -> Result<hidapi::HidDevice, hidapi::HidError> {
    let api = HidApi::new()?;
    api.open_path(device_path)
}

//// Inspect the list of devices available
/// and open a classic controller device by path.
///
/// Open a classic controller device by serial number.
pub fn open_serial(sn: &str) -> Result<hidapi::HidDevice, hidapi::HidError> {
    let api = HidApi::new()?;
    api.open_serial(VENDOR_ID, PRODUCT_ID, sn)
}

/// Find and open all classic controller devices available into a list.
pub fn open_all() -> Result<Vec<hidapi::HidDevice>, hidapi::HidError> {
    let api = HidApi::new()?;
    api.device_list()
        .filter(|d| d.vendor_id() == VENDOR_ID && d.product_id() == PRODUCT_ID)
        .map(|d| d.open_device(&api))
        .collect()
}
