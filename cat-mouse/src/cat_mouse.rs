//! A game where you're a mouse and you try to catch glowing cheese
//! while avoiding the cat.
//!
use std::marker::PhantomData;

use rand::{distributions::Uniform, Rng};
use vcs_classic_hid::{
    force_feedback::FfReport,
    input::process_input,
    led::{anims::Pulsate, AnimationEvent, LedAnimation, LedReport, LedSelection},
    Device,
};

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum GameEvent {
    Running,
    Ended,
}
impl Default for GameEvent {
    fn default() -> Self {
        GameEvent::Running
    }
}

#[derive(Debug)]
enum GameState {
    Idle {
        base_ticks: u64,
    },
    Ready {},
    Playing {
        mouse_position: i16,
        cheese_position: i16,
        cat_position: i16,
        cat_speed: i16,
    },
    GameOver {
        base_ticks: u64,
        animation: BiteAnimation,
    },
}

#[derive(Debug)]
pub struct CatMouse<D> {
    phantom: PhantomData<D>,
    score: u16,
    state: GameState,
}

impl<D> Default for CatMouse<D>
where
    D: Device,
{
    fn default() -> Self {
        CatMouse::new()
    }
}

impl<D> CatMouse<D>
where
    D: Device,
{
    pub fn new() -> Self {
        CatMouse {
            phantom: PhantomData,
            score: 0,
            state: GameState::Idle { base_ticks: 0 },
        }
    }

    pub fn update(&mut self, mut device: &mut D, ticks: u64) -> Result<GameEvent, D::Error> {
        match self.state {
            GameState::Idle { base_ticks } => {
                // process input
                if let Some(state) = process_input(&mut device)? {
                    if state.button_menu || state.button_1 {
                        self.ready(device)?;
                    } else if state.button_fuji {
                        return Ok(GameEvent::Ended);
                    }
                }

                // show some animations
                let mut led = LedReport::new();
                let mouse = ((ticks - base_ticks) / 10 % 24) as u8;
                let mut animation = Pulsate::new_with_params(
                    LedSelection::range(
                        [0, 3, 6, 9, 12, 15, 18, 21]
                            .iter()
                            .copied()
                            .filter(|x| *x > mouse),
                    ),
                    8,
                    0x1C,
                    0x60,
                );
                animation.update(ticks, &mut led);
                led.set(mouse, 0xFF);
                device.write(led)?;
            }
            GameState::Ready {} => {
                // process input
                if let Some(state) = process_input(&mut device)? {
                    if state.button_fuji {
                        return Ok(GameEvent::Ended);
                    }
                    self.start(state.roll);
                }
            }
            GameState::Playing {
                mut cheese_position,
                mut mouse_position,
                mut cat_position,
                mut cat_speed,
            } => {
                // process input
                if let Some(state) = process_input(&mut device)? {
                    if state.button_fuji {
                        return Ok(GameEvent::Ended);
                    }
                    mouse_position = state.roll as i16;
                }

                // update cat position

                // determine the closest route to the mouse
                let rel_pos = (mouse_position - cat_position).rem_euclid(1024) - 512;

                if rel_pos > 0 {
                    cat_position = (cat_position - cat_speed).rem_euclid(1024);
                } else {
                    cat_position = (cat_position + cat_speed) % 1024;
                }

                // check collision with cat
                if (mouse_position - cat_position).abs() <= 16 {
                    device.write(FfReport::new_with_params(0xF8, 28, 26, 4))?;

                    // game over
                    self.game_over(ticks);
                    return Ok(GameEvent::Running);
                }

                // check collision with cheese
                if (mouse_position - cheese_position).abs() <= 16 {
                    // increase score
                    self.score += 1;
                    // new position for the cheese
                    Self::ff_munch_cheese(&mut device)?;
                    cheese_position = spawn_cheese(mouse_position);
                    // make cat faster
                    match self.score {
                        10 => cat_speed = 2,
                        20 => cat_speed = 3,
                        30 => cat_speed = 4,
                        50 => cat_speed = 5,
                        70 => cat_speed = 6,
                        100 => cat_speed = 7,
                        _ => {
                            // do nothing
                        }
                    }
                }

                self.state = GameState::Playing {
                    cheese_position,
                    mouse_position,
                    cat_position,
                    cat_speed,
                };

                // update LEDs
                let mut led = LedReport::new();

                // mouse: high intensity pulsating LED
                let mut mouse = Pulsate::new_with_params(
                    LedSelection::single(position_to_led(mouse_position)),
                    12,
                    0xCF,
                    0xFF,
                );
                mouse.update(ticks, &mut led);

                // cheese: pulsating LED
                let mut cheese = Pulsate::new_with_params(
                    LedSelection::single(position_to_led(cheese_position)),
                    7,
                    0x1C,
                    0x66,
                );
                cheese.update(ticks, &mut led);

                // cat: low intensity LED
                led.set(position_to_led(cat_position), 0x46);

                led.send(device)?;
            }
            GameState::GameOver {
                base_ticks,
                mut animation,
            } => {
                if ticks - base_ticks > 180 {
                    self.reset(ticks);
                    return Ok(GameEvent::Running);
                }

                let mut led = LedReport::new();
                animation.update(ticks, &mut led);
                self.state = GameState::GameOver {
                    base_ticks,
                    animation,
                };
                device.write(led)?;
            }
        }
        Ok(GameEvent::Running)
    }

    pub fn reset(&mut self, ticks: u64) {
        self.score = 0;
        self.state = GameState::Idle { base_ticks: ticks };
    }

    fn ready(&mut self, device: &mut D) -> Result<(), D::Error> {
        device.write(LedReport::new())?;
        self.state = GameState::Ready {};
        Ok(())
    }

    fn start(&mut self, roll: u16) {
        let mouse_position = roll as i16;
        let cheese_position = spawn_cheese(mouse_position);
        let cat_position = spawn_cat(mouse_position);

        self.state = GameState::Playing {
            mouse_position,
            cheese_position,
            cat_position,
            cat_speed: 1,
        };
    }

    fn game_over(&mut self, ticks: u64) {
        self.state = GameState::GameOver {
            base_ticks: ticks,
            animation: BiteAnimation::new(ticks),
        };
        println!("Game Over\nScore: {}", self.score);
    }

    fn ff_munch_cheese(device: &mut D) -> Result<(), D::Error> {
        device
            .write(FfReport::new_with_params(0xA4, 9, 12, 3))
            .map(|_| ())
    }
}

/// Choose a position for the cheese
fn spawn_cheese(mouse_position: i16) -> i16 {
    let mut x: i16 = 0;
    for _ in 0..20 {
        x = rand::thread_rng().sample(Uniform::new(0, 1024));
        if (x - mouse_position).abs() > 100 {
            break;
        }
    }
    x
}

/// Choose a position for the cat
fn spawn_cat(mouse_position: i16) -> i16 {
    let mut x: i16 = 0;
    for _ in 0..20 {
        x = rand::thread_rng().sample(Uniform::new(0, 1024));
        if (x - mouse_position).abs() > 400 {
            break;
        }
    }
    x
}

#[inline]
fn position_to_led(position: i16) -> u8 {
    let x = position;
    debug_assert!(x >= 0 && x < 1024, "bad position input {}", position);
    let out = ((x as u16).wrapping_mul(3) / 128) as u8;
    debug_assert!(
        out < 24,
        "calculated invalid LED {} for position {}",
        out,
        position
    );
    out
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct BiteAnimation {
    base_ticks: u64,

    bites_left: u32,
}

impl BiteAnimation {
    pub fn new(ticks: u64) -> Self {
        BiteAnimation {
            base_ticks: ticks,
            bites_left: 4,
        }
    }
}

impl LedAnimation for BiteAnimation {
    fn reset(&mut self, ticks: u64) {
        self.base_ticks = ticks;
    }

    fn update(&mut self, ticks: u64, report: &mut LedReport) -> AnimationEvent {
        if self.bites_left == 0 {
            return AnimationEvent::Ended;
        }

        const BITE_PERIOD: u64 = 23;
        const BITE_CLOSING: u64 = 8;
        const BITE_CRUSHING: u64 = 11;
        const BITE_OPENING: u64 = 21;

        let dur = (ticks - self.base_ticks) % BITE_PERIOD;
        match dur {
            dur if dur < BITE_CLOSING => {
                // TODO
                let amount = dur * 7 / BITE_CLOSING;
                report.set_selection(LedSelection::span(0, amount as u8), 0x66);
                report.set_selection(LedSelection::span(12, amount as u8), 0x66);
            }
            dur if dur < BITE_CRUSHING => {
                report.fill(0x66);
            }
            dur if dur < BITE_OPENING => {
                let amount = (BITE_OPENING - dur) * 7 / (BITE_OPENING - BITE_CRUSHING);
                report.set_selection(LedSelection::span(0, amount as u8), 0x66);
                report.set_selection(LedSelection::span(12, amount as u8), 0x66);
            }
            _ => {
                // do nothing
                if dur == BITE_PERIOD - 1 {
                    self.bites_left -= 1;
                }
            }
        }

        AnimationEvent::Running
    }
}
