use ld_game_engine::{
    Context,
    event::Event,
    event::Event::*,
    surface::SurfaceContextExt,
    ui::Button,
    v2,
};
use ld_game_engine::event::KeyMeta;
use TutorialState::*;

use crate::ChaosTheory;
use crate::data::StoredData;

const WELCOME_TEXT: &[&str] = &[
    "Welcome to Chaos Theory!",
    "This is a game about chaos",
    "More specifically, about mutli-pendulums.",
];

const PENDULUM_TEXT: &[&str] = &[
    "<-- this is a part of the pendulum, which ",
    "    was pre-setup like that in this level",
    "",
    "    press 'space' to start the simulation",
];

const FIRST_SIM_TEXT: &[&str] = &[
    "as you can see, nothing interesting is happening",
    "",
    "press 'shift+r' to hard-reset it",
];

const SETUP_TEXT: &[&str] = &[
    "<-- now, click at this point and drag",
    "    to add parts to the pendulum.",
    "    You can easily prove that the red zones",
    "    are where you can't do that",
    "",
    "    once you're done, press 'space'",
    "    to start the simulation again",
];

const TARGET_TEXT_WON: &[&str] = &[
    "Looks like you managed to win this level!",
    "Yes, the objective is to pass through all the blue circles",
    "You get bonus points if you pass the same target more than once",
    "",
    "press 'r' to soft reset and see if you get luckier next time",
];

const TARGET_TEXT_LOST: &[&str] = &[
    "<-- The objective of the game is to pass through all the blue circles",
    "    You get bonus points if you pass the same target more than once",
    "",
    "    press 'r' to soft reset and see if you get lucky next time",
];

const LAST_TEXT: &[&str] = &[
    "Well, that's basically it, you can press 'c' to clear",
    "out excess trails, and you can control sounds or skip levels",
    "by pressing the settings button in the corner",
    "press 'space' to unpause and enjoy the game!"
];

#[derive(Debug)]
pub struct Tutorial {
    state: TutorialState,

    skip: Button,
    next: Button,
}

#[derive(Debug)]
enum TutorialState {
    WelcomeText,
    PendulumDesc,
    PendulumDescRunning { timer: f64 },
    SetupDesc,
    TargetDesc { timer: f64 },
    LastText { timer: f64 },
    Done,
}

const TIMEOUT: f64 = 5.0;

impl Tutorial {
    pub fn new(game: &mut ChaosTheory) -> Self {
        Self {
            state: WelcomeText,
            skip: game.button("skip"),
            next: game.button("next"),
        }
    }

    fn finish(&mut self, context: &mut Context<ChaosTheory>) {
        self.state = Done;
        let data = context.storage().clone();
        context.set_storage(StoredData {
            passed_tutorial: true,
            ..data
        });
    }

    pub fn on_event(&mut self, event: &Event, context: &mut Context<ChaosTheory>) -> bool {
        match (&self.state, event) {
            (WelcomeText, _) => {
                self.skip.text.pos /= 0.666;
                self.next.text.pos /= 0.666;
                if self.skip.on_event(event, context) {
                    self.finish(context);
                } else if self.next.on_event(event, context) {
                    self.state = PendulumDesc;
                }
                false
            }
            (PendulumDesc, KeyUp { code: 32, .. }) => {
                self.state = PendulumDescRunning { timer: TIMEOUT };
                true
            }
            (PendulumDesc, _) => false,
            (PendulumDescRunning { timer }, KeyUp { code: 82, meta: KeyMeta { shift: true, .. }, .. }) if *timer <= 0.0 => {
                self.state = SetupDesc;
                true
            }
            (PendulumDescRunning { .. }, _) => false,
            (SetupDesc, KeyUp { code: 32, .. }) => {
                self.state = TargetDesc { timer: TIMEOUT };
                true
            }
            (SetupDesc, e) => e.is_mouse() || matches!(e, KeyUp { code: 82, meta: KeyMeta { shift: true, .. }, .. }),
            (TargetDesc { timer }, KeyUp { code: 82, meta: KeyMeta { shift: false, .. }, .. }) if *timer <= 0.0 => {
                self.state = LastText { timer: TIMEOUT };
                true
            }
            (TargetDesc { .. }, _) => false,
            (LastText { timer }, e) if *timer <= 0.0 && e.is_mouse() || matches!(e, KeyUp { code: 67, .. }) => true,
            (LastText { timer }, KeyUp { code: 32, .. }) if *timer <= 0.0 => {
                self.finish(context);
                true
            }
            (LastText { .. }, _) => false,
            (Done, _) => true
        }
    }

    pub fn on_update(&mut self, context: &mut Context<ChaosTheory>, won: bool) -> bool {
        let surface = context.surface().context();
        let size = context.surface().size();

        match self.state {
            WelcomeText => {
                surface.fill_color("white");
                surface.set_font("2rem monospace");

                let mut top = -size.y / 2.0 + context.rem_to_px(5.0);

                for &line in WELCOME_TEXT {
                    surface.fill_text(line, 0.0, top).unwrap();
                    top += context.rem_to_px(1.5);
                }

                let (skip_width, _) = self.skip.text.compute_size(context);
                let (next_width, _) = self.next.text.compute_size(context);

                self.skip.on_update(context, v2![-skip_width, top]);
                self.next.on_update(context, v2![next_width, top]);
            }
            PendulumDesc => {
                surface.fill_color("white");
                surface.set_font("1.2rem monospace");

                let mut top = -250.0 * 0.666;

                surface.set_text_align("left");
                for &line in PENDULUM_TEXT {
                    surface.fill_text(line, 50.0 * 0.666, top).unwrap();
                    top += context.rem_to_px(0.9);
                }
                surface.set_text_align("center");
            }
            PendulumDescRunning { timer } => {
                if timer <= 0.0 {
                    surface.fill_color("white");
                    surface.set_font("1.2rem monospace");

                    let mut top = -size.y / 2.0 + context.rem_to_px(5.0);

                    for &line in FIRST_SIM_TEXT {
                        surface.fill_text(line, 0.0, top).unwrap();
                        top += context.rem_to_px(0.9);
                    }

                    return true;
                } else {
                    self.state = PendulumDescRunning { timer: timer - context.delta_time() }
                }
            }
            SetupDesc => {
                surface.fill_color("white");
                surface.set_font("1.2rem monospace");

                let mut top = -300.0 * 0.666;
                surface.set_text_align("left");
                for &line in SETUP_TEXT {
                    surface.fill_text(line, 25.0 * 0.666, top).unwrap();
                    top += context.rem_to_px(0.9);
                }
                surface.set_text_align("center");
            }
            TargetDesc { timer } => {
                if timer <= 0.0 {
                    surface.fill_color("white");
                    surface.set_font("1.2rem monospace");

                    if won {
                        let mut top = -size.y / 2.0 + context.rem_to_px(5.0);

                        for &line in TARGET_TEXT_WON {
                            surface.fill_text(line, 0.0, top).unwrap();
                            top += context.rem_to_px(0.9);
                        }
                    } else {
                        let mut top = 0.0 * 0.666;
                        surface.set_text_align("left");
                        for &line in TARGET_TEXT_LOST {
                            surface.fill_text(line, 75.0 * 0.666, top).unwrap();
                            top += context.rem_to_px(0.9);
                        }
                        surface.set_text_align("center");
                    }
                    return true;
                } else {
                    self.state = TargetDesc { timer: timer - context.delta_time() }
                }
            }
            LastText { timer } => {
                if timer <= 0.0 {
                    surface.fill_color("white");
                    surface.set_font("1.2rem monospace");

                    let mut top = -size.y / 2.0 + context.rem_to_px(5.0);

                    for &line in LAST_TEXT {
                        surface.fill_text(line, 0.0, top).unwrap();
                        top += context.rem_to_px(0.9);
                    }
                    return true;
                } else {
                    self.state = LastText { timer: timer - context.delta_time() }
                }
            }
            _ => {}
        }
        false
    }
}
