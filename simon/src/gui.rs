//! Module for showing the simulated controller via macroquad
use crate::simulator::SimulatedDevice;
use macroquad::prelude::*;
use vcs_classic_hid::input::StickPosition;
use std::time::Duration;

/// Draw the simulated classic controller on the current window.
pub fn draw_device(device: &SimulatedDevice) {
    const STICK_RADIUS: f32 = 26.;

    let center_x = screen_width() / 2.;
    let center_y = screen_height() / 2.;

    let amp = 100.;
    let amp2 = 80.;
    let (stick_x, stick_y) = match device.stick_position() {
        StickPosition::Up => (center_x, center_y - amp),
        StickPosition::UpRight => (center_x + amp2, center_y - amp2),
        StickPosition::Right => (center_x + amp, center_y),
        StickPosition::DownRight => (center_x + amp2, center_y + amp2),
        StickPosition::Down => (center_x, center_y + amp),
        StickPosition::DownLeft => (center_x - amp2, center_y + amp2),
        StickPosition::Left => (center_x - amp, center_y),
        StickPosition::UpLeft => (center_x - amp2, center_y - amp2),
        StickPosition::Center => (center_x, center_y),
    };

    draw_circle_lines(stick_x, stick_y, STICK_RADIUS, 2., BLACK);

    draw_circle(
        stick_x,
        stick_y,
        STICK_RADIUS,
        Color {
            r: 0.24,
            g: 0.24,
            b: 0.24,
            a: 1.,
        },
    );

    draw_led_ring(200.0, device.leds());
}

fn draw_led_ring(size: f32, led_state: &[u8]) {
    let radius = size;

    let center_x = screen_width() / 2.;
    let center_y = screen_height() / 2.;

    for (i, led) in led_state.iter().copied().enumerate() {
        let angle = (i as f32) * std::f32::consts::TAU / led_state.len() as f32;

        let pos_x = center_x - radius * angle.sin();
        let pos_y = center_y + radius * angle.cos();
        draw_circle(
            pos_x,
            pos_y,
            16.,
            Color {
                r: led as f32 / 255.,
                g: led as f32 / 1000.,
                b: 0.,
                a: 1.,
            },
        );

        draw_text(&format!("{}", i), pos_x - 6., pos_y + 4., 18., BLACK);
        std::thread::sleep(Duration::new(0, 450_000));
    }
}
