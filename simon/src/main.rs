use std::time::Duration;

#[cfg(feature = "simulator")]
use macroquad::prelude::*;
#[cfg(feature = "simulator")]
use vcs_classic_hid_simulator as simulator;
#[cfg(feature = "simulator")]
use simulator::SimulatedDevice;
use simon::{GameEvent, Simon};

mod simon;

#[cfg(feature = "simulator")]
mod gui;

#[cfg(not(feature = "simulator"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device = vcs_classic_hid::open()?;
    let mut f = 0;
    let mut game = Simon::new();
    loop {
        let a = game.update(&mut device, f)?;

        if a == GameEvent::Ended {
            break;
        }
        
        f += 1;
        std::thread::sleep(Duration::from_millis(25));
    }

    // reset LEDs
    device.write(&[2, 0, 0, 0])?;
    std::thread::sleep(Duration::from_millis(30));
    Ok(())
}

// -- GUI version --

#[cfg(feature = "simulator")]
#[macroquad::main("VCS Classic Controller Simulator")]
async fn main() {
    let mut device = SimulatedDevice::new();

    let mut f: u64 = 0;

    let mut game = Simon::new();

    loop {
        // -- input processing from keyboard to simulated controller state --
        let key_left = is_key_down(KeyCode::Left);
        let key_right = is_key_down(KeyCode::Right);
        let key_up = is_key_down(KeyCode::Up);
        let key_down = is_key_down(KeyCode::Down);

        let stick_position = match (key_up, key_right, key_down, key_left) {
            (true, false, false, false) => 1,
            (true, true, false, false) => 2,
            (false, true, false, false) => 3,
            (false, true, true, false) => 4,
            (false, false, true, false) => 5,
            (false, false, true, true) => 6,
            (false, false, false, true) => 7,
            (true, false, false, true) => 8,
            _ => 0,
        };

        device.move_stick(stick_position);

        let enter = is_key_down(KeyCode::Enter);
        device.set_button_1(enter);
        let shift = is_key_down(KeyCode::RightShift);
        device.set_button_menu(shift);
        let backspace = is_key_down(KeyCode::Backspace);
        device.set_button_back(backspace);

        // -- game logic --
        {
            game.update(&mut device, f).unwrap();
        }

        // -- rendering --

        clear_background(Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.,
        });

        let header = format!(
            "Frame: {:6} ({:3} FPS)  time: {:6.2}",
            f,
            get_fps(),
            get_time()
        );

        draw_text(&header, 10., 20., 22., WHITE);

        gui::draw_device(&device);

        f += 1;

        next_frame().await
    }
}
