use std::borrow::Cow;

use wasm_bindgen::prelude::*;

use ld_game_engine::{Game, GameRun, GameState, Resources, util::setup_panic_hook};
use ld_game_engine::ui::Button;

use crate::states::main_game::{Level, MainGame};

pub mod rope;
pub mod states;

#[derive(Debug)]
pub struct ChaosTheory {}

impl ChaosTheory {
    pub fn button(&mut self, text: impl Into<Cow<'static, str>>) -> Button {
        Button::new(text.into(), "#661ebd")
            .with_size(1.5)
            .with_hover_color("#8024f0")
    }
}

impl Game for ChaosTheory {
    type Storage = ();

    fn load(_resources: Resources) -> (Self, Box<dyn GameState<Self>>) {
        let mut global = ChaosTheory {};
        let initial_state = Box::new(MainGame::new(Level::test(), &mut global));
        (global, initial_state)
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    setup_panic_hook();

    ChaosTheory::run();
}
