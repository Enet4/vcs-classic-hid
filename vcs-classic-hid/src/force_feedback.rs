//! Force feedback module
use crate::Device;

/// A force feedback report.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct FfReport([u8; 6]);

impl Default for FfReport {
    fn default() -> Self {
        FfReport::new()
    }
}

impl FfReport {
    /// Create a new force feedback report
    /// which disables any ongoing force feedback.
    pub const fn new() -> Self {
        FfReport([1, 0, 0, 0, 0, 0])
    }

    /// Create a new force feedback report
    /// with the given parameters.
    ///
    /// - `intensity`: how intense is the force feedback
    /// - `up_time`: the duration of each vibration
    /// - `down_time`: the time off between each vibration
    /// - `times`: the number of times to vibrate
    pub const fn new_with_params(intensity: u8, up_time: u8, down_time: u8, times: u8) -> Self {
        FfReport([
            1,
            intensity,
            up_time,
            down_time,
            times,
            0,
        ])
    }

    /// Send this report as an HID message to the given device.
    ///  
    /// **Safety:** although not memory unsafe, the operation must be done
    /// on a readily available device handle for the Atari Classic Controller.
    /// The effects on any other device are unknown and potentially dangerous.
    #[inline]
    pub fn send<D>(&self, mut device: D) -> Result<(), D::Error>
    where
        D: Device,
    {
        device.write(&self.0).map(|_| ())
    }
}

impl AsRef<[u8]> for FfReport {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
