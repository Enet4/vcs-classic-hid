//! Controller input handling module

use crate::Device;

/// Identifier for the position of the controller's stick.
///
/// They can be used 
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum StickPosition {
    Center = 0,
    Up = 1,
    UpRight = 2,
    Right = 3,
    DownRight = 4,
    Down = 5,
    DownLeft = 6,
    Left = 7,
    UpLeft = 8,
}

impl Default for StickPosition {
    fn default() -> Self {
        StickPosition::Center
    }
}

impl StickPosition {
    pub fn new() -> Self {
        StickPosition::default()
    }
    
    pub fn from_u8(position: u8) -> Option<Self> {
        match position {
            0 => Some(StickPosition::Center),
            1 => Some(StickPosition::Up),
            2 => Some(StickPosition::UpRight),
            3 => Some(StickPosition::Right),
            4 => Some(StickPosition::DownRight),
            5 => Some(StickPosition::Down),
            6 => Some(StickPosition::DownLeft),
            7 => Some(StickPosition::Left),
            8 => Some(StickPosition::UpLeft),
            _ => None,
        }
    }
}

/// A friendly representation of a game controller input state.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct State {
    /// The position of the stick
    pub stick_position: StickPosition,
    /// Whether the main button is down
    pub button_1: bool,
    /// Whether the secondary trigger is down
    pub button_2: bool,
    /// Whether the back button is down
    pub button_back: bool,
    /// Whether the menu/context button is down
    pub button_menu: bool,
    /// Whether the Atari button is down
    pub button_fuji: bool,
    /// The absolute position of the rotational paddle,
    /// as a number between 0 and 1023
    pub roll: u16,
}

impl State {

    /// Obtain the controller's state 
    /// by reading the next controller state report.
    ///
    /// This is not fully recommended because
    /// if many more events are on queue,
    /// the obtained state may be stale.
    ///
    /// This function may panic if the device is
    /// not an Atari VCS classic controller.
    pub fn from_device<D>(mut device: D) -> Result<Self, D::Error>
    where
        D: Device,
    {
        let mut buf = [0; 6];
        buf[0] = 1;
        device.read(&mut buf)?;

        Ok(Self::from_report(&buf))
    }

    /// Obtain the controller's state from the full report packet.
    ///
    /// May panic if the data cannot represent an input report.
    pub fn from_report(data: &[u8]) -> Self {
        assert!(data.len() >= 6);
        assert_eq!(data[0], 1);

        msg_to_state(&data[1..])
    }
}

fn msg_to_state(msg: &[u8]) -> State {
    assert_eq!(msg.len(), 5);
    State {
        stick_position: StickPosition::from_u8(msg[2] >> 4).unwrap_or_default(),
        button_1: (msg[1] & 1) == 1,
        button_2: ((msg[1] >> 1) & 1) == 1,
        button_back: (msg[2] & 1) == 1,
        button_menu: ((msg[2] >> 1) & 1) == 1,
        button_fuji: ((msg[2] >> 2) & 1) == 1,
        roll: u16::from(msg[3]) + (u16::from(msg[4]) << 8),
    }
}


/// Process input reports in queue from the device
/// and return its current state.
///
/// This function does not block.
/// Might return `None` if no input report was received.
/// When this happens, game loops should preferably assume
/// no changes occurred to the controller's input state.
pub fn process_input<D>(mut device: D) -> Result<Option<State>, D::Error>
where
    D: Device,
{
    let mut buf = [0; 6];
    buf.fill(0);

    let mut has_msg = false;
    let mut last_amount = 0;
    device.set_blocking(false)?;
    let msg = loop {
    
        let amount = device.read(&mut buf)?;

        if amount == 0 && !has_msg {
            // queue empty, continue without message
            break &buf[0..0];
            
        } else if amount != 0 {
            has_msg = true;
            last_amount = amount;
            // consume more events while it doesn't block
            continue;
        }

        let msg = &buf[..last_amount];
        break msg;
    };

    if !msg.is_empty() {
        if msg.len() != 5 {
            eprintln!(
                "Special report #{:02X}: {:?}",
                buf[0],
                msg,
            );
            Ok(None)
        } else {
            Ok(Some(msg_to_state(msg)))
        }
    } else {
        Ok(None)
    }
}
