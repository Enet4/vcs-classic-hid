//! LED manipulation module
pub mod anims;
use crate::Device;

/// A behavioral construct for effects and animations on the controller's LEDs.
///
pub trait LedAnimation {
    /// Reset the effect's state. This generally means a rewind.
    /// Generally, the state of the animation
    /// should always be the same after this call.
    ///
    /// In stateless animations,
    /// this function serves no purpose and should be a no-op.
    #[allow(unused)]
    fn reset(&mut self, ticks: u64) {}

    /// Update the state of the animation,
    /// applying the intended effects on the given report.
    ///
    /// Returns `Ended` if the animation ends
    /// and no longer wishes to request for LED activations.
    fn update(&mut self, ticks: u64, report: &mut LedReport) -> AnimationEvent;
}

/// Identifier for a quadrant of the LED ring.
///
/// Quadrants are not fully disjoint:
/// they share an LED at the extremities.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum Quadrant {
    BottomLeft = 0,
    TopLeft = 1,
    TopRight = 2,
    BottomRight = 3,
}

impl Quadrant {
    pub fn from_u8(quadrant: u8) -> Option<Self> {
        match quadrant {
            0 => Some(Quadrant::BottomLeft),
            1 => Some(Quadrant::TopLeft),
            2 => Some(Quadrant::TopRight),
            3 => Some(Quadrant::BottomRight),
            _ => None,
        }
    }
}

/// A selection of leds in the ring.
#[derive(Debug, Default, Copy, Clone, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct LedSelection([bool; 24]);

impl LedSelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a single LED by index, from 0 to 23.
    pub fn single(index: u8) -> Self {
        let mut x = [false; 24];
        x[index as usize] = true;
        LedSelection(x)
    }

    /// Select a single LED by an arbitrary range of indices.
    pub fn range<R>(range: R) -> Self
    where
        R: IntoIterator<Item = u8>,
    {
        let mut x = [false; 24];
        for i in range {
            x[i as usize] = true;
        }
        LedSelection(x)
    }

    /// Select a diagonal quadrant of LEDs, from 0 to 3.
    pub fn quadrant(quadrant: u8) -> Self {
        assert!(quadrant < 4);
        let mut x = [false; 24];
        let base = quadrant as usize * 6;
        if quadrant == 3 {
            x[base..=base + 5].fill(true);
            x[0] = true;
        } else {
            x[base..=base + 6].fill(true);
        }
        LedSelection(x)
    }

    /// Select a span of LEDS comprising the center LED
    /// plus the adjacent LEDs at the given radius.
    pub fn span(center: u8, radius: u8) -> Self {
        let mut x = [false; 24];
        let center = center as usize;
        let radius = radius as usize;
        for i in 0..radius {
            if let Some(l) = x.get_mut((center + i).rem_euclid(24)) {
                *l = true;
            }
        }
        for i in 1..radius {
            if let Some(l) = x.get_mut((center as i32 - i as i32).rem_euclid(24) as usize) {
                *l = true;
            }
        }
        LedSelection(x)
    }

    /// Combine (union) with another selection.
    pub fn or(self, other: LedSelection) -> Self {
        let mut x = [false; 24];
        for (out, (v1, v2)) in x.iter_mut().zip(self.0.iter().zip(other.0.iter())) {
            *out = *v1 || *v2;
        }
        LedSelection(x)
    }

    /// Select all LEDs.
    pub const ALL: LedSelection = LedSelection([true; 24]);

    /// Select no LED.
    pub const NONE: LedSelection = LedSelection([false; 24]);
}

/// Structure representing a report for LED activation on the controller.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LedReport([u8; 28]);

/// By default, an LED report will turn off all LEDs.
impl Default for LedReport {
    #[inline]
    fn default() -> Self {
        let mut arr = [0; 28];
        arr[0] = 2;
        arr[1] = 25;
        LedReport(arr)
    }
}

impl LedReport {
    /// Create a new LED report.
    ///
    /// By default, an LED report will turn off all LEDs.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new LED report filled with the given intensity value.
    #[inline]
    pub fn filled(value: u8) -> Self {
        let mut x = Self::default();
        x.fill(value);
        x
    }

    /// Turn all ring of LEDs off.
    #[inline]
    pub fn clear(&mut self) {
        self.fill(0)
    }

    /// Set all LEDs in the ring to the given value.
    #[inline]
    pub fn fill(&mut self, value: u8) {
        (&mut self.0[3..]).fill(value);
    }

    /// Set the Fuji LED to a value.
    #[inline]
    pub fn set_fuji(&mut self, value: u8) {
        self.0[2] = value;
    }

    /// Set a LED in the ring to a value.
    #[inline]
    pub fn set(&mut self, led: u8, value: u8) {
        self.0[3 + led as usize] = value;
    }

    /// Set a selection of LEDs in the ring to a value.
    #[inline]
    pub fn set_selection(&mut self, selection: LedSelection, value: u8) {
        for (led, sel) in std::iter::Iterator::zip(self.0.iter_mut().skip(3), selection.0.iter()) {
            if *sel {
                *led = value;
            }
        }
    }

    /// Invert the value of the LED in the ring.
    #[inline]
    pub fn invert(&mut self, led: u8) {
        let led = led as usize;
        self.0[3 + led] = !self.0[3 + led];
    }

    /// Invert the values of a selection of the LED in the ring.
    #[inline]
    pub fn invert_selection(&mut self, selection: LedSelection) {
        for (led, sel) in std::iter::Iterator::zip(self.0.iter_mut().skip(3), selection.0.iter()) {
            if *sel {
                *led = !*led;
            }
        }
    }

    /// Add an intensity to a LED in the ring.
    ///
    /// Values are automatically clamped.
    #[inline]
    pub fn saturating_add(&mut self, led: u8, value_delta: i16) {
        let led = led as usize;
        let current = self.0[3 + led];
        let out = (i16::from(current) + value_delta).clamp(0, 255);
        self.0[3 + led] = out as u8;
    }

    /// Add an intensity to a selection of LEDs in the ring.
    #[inline]
    pub fn saturating_add_selection(&mut self, selection: LedSelection, value_delta: i16) {
        selection
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, sel)| if *sel { Some(i) } else { None })
            .for_each(|i| self.saturating_add(i as u8, value_delta))
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

impl AsRef<[u8]> for LedReport {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum AnimationEvent {
    /// the animation is running
    Running,
    /// the animation is ended and should receive no more update events
    Ended,
}

impl Default for AnimationEvent {
    fn default() -> Self {
        AnimationEvent::Running
    }
}
