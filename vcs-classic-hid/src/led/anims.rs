//! Animation abstraction for LEDs
//!
//! This module contains implementations for animations
//! that can be applied to the LED.
//!
//! For any of these to work, a steady event loop is required.
//!
//! ## Example
//!
//! ```no_run
//! # fn main() -> Result<(), hidapi::HidError> {
//! use std::time::Duration;
//! use vcs_classic_hid::{Device, LedReport, open};
//! use vcs_classic_hid::led::LedAnimation;
//!
//! let mut device = open()?;
//! let mut ticks = 0;
//! let mut animation = vcs_classic_hid::led::anims::RotatingLed;
//! loop {
//!     let mut report = LedReport::new();
//!     animation.update(ticks, &mut report);
//!     Device::write(&mut device, report)?;
//!
//!     ticks += 1;
//!     std::thread::sleep(Duration::from_millis(25)); // ~ 50 FPS
//! }
//! # Ok(())
//! # }
//! ```
use super::{AnimationEvent, LedAnimation, LedReport, LedSelection};

#[derive(Debug)]
pub struct RotatingLed;

impl LedAnimation for RotatingLed {
    fn reset(&mut self, _ticks: u64) {}

    fn update(&mut self, ticks: u64, report: &mut LedReport) -> AnimationEvent {
        report.set(((ticks / 20) % 24) as u8, 0xFF);
        AnimationEvent::Running
    }
}

/// Incrementally pulsating LEDs
#[derive(Debug)]
pub struct OneWayPulsate {
    selection: LedSelection,
    value_min: u8,
    value_max: u8,
    tick_period: u64,
}

impl OneWayPulsate {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_selection(selection: LedSelection) -> Self {
        OneWayPulsate {
            selection,
            ..OneWayPulsate::default()
        }
    }

    pub fn new_with_params(selection: LedSelection, tick_period: u64, value_min: u8, value_max: u8) -> Self {
        OneWayPulsate {
            selection,
            tick_period,
            value_min,
            value_max
        }
    }
}

impl Default for OneWayPulsate {
    fn default() -> Self {
        OneWayPulsate {
            selection: LedSelection::ALL,
            value_min: 0,
            value_max: 0xFF,
            tick_period: 128,
        }
    }
}

impl LedAnimation for OneWayPulsate {
    fn update(&mut self, ticks: u64, report: &mut LedReport) -> AnimationEvent {
        let phase = ticks % self.tick_period;
        let value = self.value_min + (phase * (self.value_max - self.value_min) as u64 / self.tick_period) as u8;
        report.set_selection(self.selection, value);
        AnimationEvent::Running
    }
}

/// Pulsating LEDs.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pulsate {
    selection: LedSelection,
    value_min: u8,
    value_max: u8,
    tick_period: u64,
}

impl Pulsate {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_selection(selection: LedSelection) -> Self {
        Pulsate {
            selection,
            ..Pulsate::default()
        }
    }

    pub fn new_with_params(selection: LedSelection, tick_period: u64, value_min: u8, value_max: u8) -> Self {
        Pulsate {
            selection,
            tick_period,
            value_min,
            value_max
        }
    }
}

impl Default for Pulsate {
    fn default() -> Self {
        Pulsate {
            selection: LedSelection::ALL,
            value_min: 0,
            value_max: 0xFF,
            tick_period: 128,
        }
    }
}

impl LedAnimation for Pulsate {
    fn update(&mut self, ticks: u64, report: &mut LedReport) -> AnimationEvent {
        let down = ticks / self.tick_period % 2 == 1;
        let phase = ticks % self.tick_period;
        
        let value = if down {
            self.value_min + (phase * (self.value_max - self.value_min) as u64 / self.tick_period) as u8
        } else {
            self.value_max - (phase * (self.value_max - self.value_min) as u64 / self.tick_period) as u8
        };
        report.set_selection(self.selection, value);
        AnimationEvent::Running
    }
}
/// An attack-sustain-release pulse.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Asr {
    selection: LedSelection,
    base_tick: u64,
    ticks_attack: u64,
    ticks_sustain: u64,
    ticks_release: u64,
    value: u8,
}

impl Asr {
    pub fn new_with_selection(
        selection: LedSelection,
        value: u8,
    ) -> Self {
        Asr {
            selection,
            value,
            .. Asr::default()
        }
    }

    pub fn new_with_params(
        selection: LedSelection,
        value: u8,
        ticks_attack: u64,
        ticks_sustain: u64,
        ticks_release: u64,
    ) -> Self {
        Asr {
            selection,
            base_tick: 0,
            ticks_attack,
            ticks_sustain,
            ticks_release,
            value,
        }
    }
}

impl Default for Asr {
    fn default() -> Self {
        Asr {
            selection: LedSelection::ALL,
            base_tick: 0,
            ticks_attack: 20,
            ticks_sustain: 60,
            ticks_release: 20,
            value: 0xFF,
        }
    }
}

impl LedAnimation for Asr {
    fn reset(&mut self, ticks: u64) {
        self.base_tick = ticks;
    }

    fn update(&mut self, ticks: u64, report: &mut LedReport) -> AnimationEvent {
        let dur = ticks - self.base_tick;

        match dur {
            dur if dur < self.ticks_attack => {
                let val = (dur * 255 / self.ticks_attack) as u8;
                report.set_selection(self.selection, val);

                AnimationEvent::Running
            }
            dur if dur < self.ticks_attack + self.ticks_sustain => {
                report.set_selection(self.selection, 0xFF);
                AnimationEvent::Running
            }
            dur if dur < self.ticks_attack + self.ticks_sustain + self.ticks_release => {
                let dur = dur - self.ticks_attack - self.ticks_sustain;
                let val = !((dur * 255 / self.ticks_release) as u8);
                report.set_selection(self.selection, val);
                AnimationEvent::Running
            }
            _ => {
                report.set_selection(self.selection, 0);
                AnimationEvent::Ended
            }
        }
    }
}
