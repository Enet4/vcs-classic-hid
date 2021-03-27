//! Simulated classic device implementation.
//!
//! Just create a [`SimulatedDevice`](crate::SimulatedDevice).
//! Writes and reads can be performed as if it were the real device.

use vcs_classic_hid::{input::StickPosition, Device};

#[derive(Debug, Default)]
pub struct SimulatedDevice {
    stick_position: u8,
    stick_roll: u16,
    empty_queue: bool,
    led_state: [u8; 24],
    button_1: bool,
    button_2: bool,
    button_back: bool,
    button_menu: bool,
    button_fuji: bool,
}

impl SimulatedDevice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn leds(&self) -> &[u8; 24] {
        &self.led_state
    }

    pub fn move_stick(&mut self, position: u8) {
        self.stick_position = position;
        // has new info
        self.enqueue();
    }

    pub fn set_roll(&mut self, roll: u16) {
        self.stick_roll = roll & 0x3FF;
        // has new info
        self.enqueue();
    }

    pub fn set_button_1(&mut self, down: bool) {
        self.button_1 = down;
        // has new info
        self.enqueue();
    }

    pub fn set_button_2(&mut self, down: bool) {
        self.button_2 = down;
        // has new info
        self.enqueue();
    }

    pub fn set_button_back(&mut self, down: bool) {
        self.button_back = down;
        // has new info
        self.enqueue();
    }

    pub fn set_button_menu(&mut self, down: bool) {
        self.button_menu = down;
        // has new info
        self.enqueue();
    }

    pub fn set_button_fuji(&mut self, down: bool) {
        self.button_fuji = down;
        // has new info
        self.enqueue();
    }

    pub fn is_button_1_down(&self) -> bool {
        self.button_1
    }

    pub fn is_button_2_down(&self) -> bool {
        self.button_2
    }

    pub fn is_button_back_down(&self) -> bool {
        self.button_back
    }

    pub fn is_button_menu_down(&self) -> bool {
        self.button_menu
    }

    pub fn is_button_fuji_down(&self) -> bool {
        self.button_fuji
    }

    pub fn stick_position(&self) -> StickPosition {
        StickPosition::from_u8(self.stick_position).unwrap()
    }

    #[inline]
    fn enqueue(&mut self) {
        self.empty_queue = false;
    }
}

impl Device for SimulatedDevice {
    type Error = &'static str;

    fn set_blocking(&mut self, _blocking: bool) -> Result<(), Self::Error> {
        // no op
        Ok(())
    }

    fn read(&mut self, out: &mut [u8]) -> Result<usize, Self::Error> {
        if self.empty_queue {
            // no new input to consume
            return Ok(0);
        }

        if out.len() < 6 {
            eprintln!("Out buffer too short");
            return Ok(0);
        }

        // produce controller input report
        out[0] = 1;
        out[1] = self.button_1 as u8 | (self.button_2 as u8) << 1;
        out[2] = self.button_back as u8
            | ((self.button_fuji as u8) << 2)
            | self.stick_position << 4
            | ((self.button_menu as u8) << 1);
        out[3] = self.stick_roll as u8;
        out[4] = (self.stick_roll >> 8) as u8;
        out[5] = 0;

        // force the queue to be empty
        // so that the next request for data suggests that the queue is empty
        self.empty_queue = true;
        Ok(5)
    }

    fn write<T>(&mut self, data: T) -> Result<usize, Self::Error>
    where
        T: AsRef<[u8]>,
    {
        let data = data.as_ref();
        match data.get(0) {
            None => {
                eprintln!("No report was sent");
                Ok(0)
            }
            Some(2) => {
                // LED report
                data.get(1).map(|l| {
                    let l = *l as usize;
                    // note, data[2] is ignored for now
                    for (led, d) in
                        std::iter::Iterator::zip(self.led_state[..].iter_mut(), data[3..].iter())
                            .take(l)
                    {
                        *led = *d;
                    }
                });

                Ok(data.len())
            }
            Some(b) => {
                eprintln!("Unsupported report type #{}", b);
                Ok(data.len())
            }
        }
    }
}
