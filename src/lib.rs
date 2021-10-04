use std::borrow::Cow;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use data::StoredData;
use ld_game_engine::{
    Game,
    GameRun,
    GameState,
    Resources,
    sound::Sound,
    ui::Button,
    util::setup_panic_hook,
};
use ld_game_engine::util::Bitmap;

use crate::main_game::{Level, MainGame};

pub mod rope;
pub mod main_game;
pub mod data;
pub mod tutorial;

#[derive(Debug)]
pub struct ChaosTheory {
    background: Sound,
    click: Rc<Sound>,
    hover: Rc<Sound>,
    target_hit: Sound,
    win: Sound,
}

pub const BUTTON_COLOR: &str = "#661ebd";
pub const HOVER_COLOR: &str = "#8024f0";

impl ChaosTheory {
    pub fn button(&mut self, text: impl Into<Cow<'static, str>>) -> Button {
        Button::new(text.into(), BUTTON_COLOR)
            .with_size(1.5)
            .with_hover_color(HOVER_COLOR)
            .with_click_sound(self.click.clone())
            .with_hover_sound(self.hover.clone())
    }
}

impl Game for ChaosTheory {
    type Storage = StoredData;

    fn load(resources: Resources) -> (Self, Box<dyn GameState<Self>>) {
        let mut global = ChaosTheory {
            background: resources.load_sound("assets/background.mp3")
                .with_volume(0.05)
                .with_layers(Bitmap::empty().with_on(1))
                .looped(),
            click: Rc::new(resources.load_sound("assets/click.wav").with_volume(0.2)),
            hover: Rc::new(resources.load_sound("assets/hover.wav").with_volume(0.2)),
            target_hit: resources.load_sound("assets/target_hit2.wav").with_volume(0.2),
            win: resources.load_sound("assets/win.wav").with_volume(0.2),
        };
        let initial_state = Box::new(MainGame::new(Level::tutorial_level(), &mut global));
        (global, initial_state)
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    setup_panic_hook();

    ChaosTheory::run();
}
