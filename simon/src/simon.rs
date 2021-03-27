//! A game of Simon says on the classic controller
use std::marker::PhantomData;

use vcs_classic_hid::{Device, force_feedback::FfReport, input::{process_input, StickPosition}, led::{
        anims::{Asr, Pulsate},
        AnimationEvent, LedAnimation, LedReport, LedSelection,
    }};

use rand::Rng;

/// A game of Simon Says for the classic controller.
#[derive(Debug)]
pub struct Simon<D> {
    phantom: PhantomData<D>,
    sequence: Vec<Choice>,
    state: GameState,
}

/// Sum type for most of the game's state.
///
/// The only property not stored here is `sequence`.
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Idle {
        base_tick: u64,
    },
    Preparing {
        base_tick: u64,
    },
    Showing {
        /// the current animation showing what the user needs to press
        anim: Asr,
        index: usize,
    },
    Playing {
        /// the index yet to be picked by the player (starts at 0)
        index: usize,
        /// the choice currently pushed on the stick (applied on release)
        pushed: Option<Choice>,
    },
    GameOver {
        /// the moment when it went game over, so we know when to stop
        base_tick: u64,

        anim: Pulsate,
    },
}

/// The event to communicate with the game loop.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum GameEvent {
    Running,
    Ended,
}

/// Enumeration of the possible choices that the user needs to guess.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum Choice {
    Up,
    Right,
    Down,
    Left,
}

impl Choice {
    fn from_u8(value: u8) -> Option<Choice> {
        match value {
            0 => Some(Choice::Up),
            1 => Some(Choice::Right),
            2 => Some(Choice::Down),
            3 => Some(Choice::Left),
            _ => None,
        }
    }
}

impl<D> Default for Simon<D> {
    fn default() -> Self {
        Simon {
            phantom: PhantomData,
            sequence: Vec::new(),
            state: GameState::Idle { base_tick: 0 },
        }
    }
}

impl<D> Simon<D>
where
    D: Device,
{
    pub fn new() -> Self {
        println!("Simon!");
        Self::default()
    }

    pub fn reset(&mut self, ticks: u64) {
        println!("Simon!");
        self.state = GameState::Idle { base_tick: ticks };
        self.sequence.clear();
    }

    pub fn update(&mut self, mut device: &mut D, ticks: u64) -> Result<GameEvent, D::Error> {
        match self.state {
            GameState::Idle { base_tick } => self.update_idle(device, ticks - base_tick),

            GameState::Preparing { base_tick } => {
                if ticks - base_tick > 30 {
                    self.start(ticks);
                }

                // reset LEDs
                let report = LedReport::new();
                device.write(report)?;
        
                Ok(GameEvent::Running)
            }

            GameState::Showing { mut anim, index } => {
                let mut report = LedReport::new();
                let e = anim.update(ticks, &mut report);
                if let AnimationEvent::Ended = e {
                    // done showing an item
                    if index < self.sequence.len() - 1 {
                        // move on to the next one
                        let index = index + 1;
                        let mut anim = Self::anim_simon(self.sequence[index]);
                        anim.reset(ticks);

                        self.state = GameState::Showing { anim, index };
                    } else {
                        // we're done showing items,
                        // move on to playing state
                        self.state = GameState::Playing {
                            index: 0,
                            pushed: None,
                        };
                    }
                }
                report.send(device)?;
                Ok(GameEvent::Running)
            }

            GameState::Playing { index, pushed } => {
                if let Some(state) = process_input(&mut device)? {
                    match pushed {
                        None => {
                            // check for user input
                            match state.stick_position {
                                StickPosition::Up => {
                                    self.state = GameState::Playing {
                                        index,
                                        pushed: Some(Choice::Up),
                                    };
                                }
                                StickPosition::Right => {
                                    self.state = GameState::Playing {
                                        index,
                                        pushed: Some(Choice::Right),
                                    };
                                }
                                StickPosition::Down => {
                                    self.state = GameState::Playing {
                                        index,
                                        pushed: Some(Choice::Down),
                                    };
                                }
                                StickPosition::Left => {
                                    self.state = GameState::Playing {
                                        index,
                                        pushed: Some(Choice::Left),
                                    };
                                }
                                _ => {
                                    // no-op
                                }
                            }
                        }
                        Some(c) => {
                            // if stick was centered, apply choice
                            if state.stick_position == StickPosition::Center {
                                if c != self.sequence[index] {
                                    self.game_over(device, ticks)?;
                                } else {
                                    // correct!
                                    let index = index + 1;
                                    if index == self.sequence.len() {
                                        // next level
                                        self.next_level(ticks);
                                    } else {
                                        // next element in sequence
                                        self.state = GameState::Playing {
                                            index,
                                            pushed: None,
                                        };

                                        // reset LEDs
                                        let report = LedReport::new();
                                        report.send(device)?;
                                    }
                                }
                            } else {
                                // LEDs showing decision
                                let mut report = LedReport::new();
                                report.set_selection(Self::led_select_direction(c), 0xFF);
                                report.send(device)?;
                            }
                        }
                    }
                }
                Ok(GameEvent::Running)
            }

            GameState::GameOver {
                base_tick,
                mut anim,
            } => {
                if ticks - base_tick > 160 {
                    // cancel any pending vibration
                    device.write(FfReport::new())?;
                    self.reset(ticks);
                }

                let mut report = LedReport::new();
                anim.update(ticks - base_tick, &mut report);
                report.send(device)?;
                Ok(GameEvent::Running)
            }
        }
    }

    fn led_select_direction(choice: Choice) -> LedSelection {
        match choice {
            Choice::Up => LedSelection::range(9..16),
            Choice::Right => LedSelection::range(15..22),
            Choice::Down => LedSelection::range((21..28).map(|x| x % 24)),
            Choice::Left => LedSelection::range(3..10),
        }
    }

    fn anim_simon(choice: Choice) -> Asr {
        let selection = Self::led_select_direction(choice);
        Asr::new_with_params(selection, 0xFF, 5, 30, 8)
    }

    fn update_idle(&mut self, mut device: &mut D, ticks: u64) -> Result<GameEvent, D::Error> {
        let state = process_input(&mut device)?;

        if let Some(state) = state {
            if state.button_menu || state.button_1 {
                self.prepare(ticks)?;
            }
            if state.button_fuji {
                return Ok(GameEvent::Ended);
            }
        }

        // do some silly animation, wait for menu button press
        let mut report = LedReport::new();

        let c = match (ticks / 50) % 4 {
            0 => Choice::Up,
            1 => Choice::Right,
            2 => Choice::Down,
            3 => Choice::Left,
            _ => unreachable!(),
        };

        Self::anim_simon(c).update(ticks % 50, &mut report);

        report.send(device)?;
        Ok(GameEvent::Running)
    }

    fn prepare(&mut self, ticks: u64) -> Result<(), D::Error> {
        println!("Get ready!");
        self.state = GameState::Preparing { base_tick: ticks };
        Ok(())
    }

    fn start(&mut self, ticks: u64) {
        println!("It begins! Watch carefully!");

        // pick the first two choices
        self.sequence = vec![Self::choose(), Self::choose()];

        let mut anim = Self::anim_simon(self.sequence[0]);
        anim.reset(ticks);
        self.state = GameState::Showing { anim, index: 0 };
    }

    fn next_level(&mut self, ticks: u64) {
        self.sequence.push(Self::choose());

        let mut anim = Self::anim_simon(self.sequence[0]);
        anim.reset(ticks);
        self.state = GameState::Showing { anim, index: 0 };
    }

    fn choose() -> Choice {
        let c = rand::thread_rng().gen_range(0_u8..=3);
        Choice::from_u8(c).unwrap()
    }

    fn game_over(&mut self, device: &mut D, ticks: u64) -> Result<(), D::Error> {
        self.state = GameState::GameOver {
            base_tick: ticks,
            anim: Pulsate::new_with_params(LedSelection::ALL, 18, 0x25, 0x7F),
        };
        println!("Game Over\nScore: {}", self.sequence.len());

        // force feedback for a few moments
        device.write(FfReport::new_with_params(0xCC, 0xBB, 0, 1))?;

        Ok(())
    }
}
