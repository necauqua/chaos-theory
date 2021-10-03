use nalgebra::Vector2;
use wasm_bindgen::prelude::*;
use ld_game_engine::{Game, GameRun, GameState, Resources, util::setup_panic_hook};
use crate::states::main_game::{Level, MainGame};

pub mod states;
pub mod rope;

pub type Pos = Vector2<f64>;

#[derive(Debug)]
pub struct ChaosTheory {

}

impl Game for ChaosTheory {
    type Storage = ();

    fn load(_resources: Resources) -> (Self, Box<dyn GameState<Self>>) {
        let global = ChaosTheory {};
        let initial_state = Box::new(MainGame::new(Level::test()));
        (global, initial_state)
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    setup_panic_hook();

    ChaosTheory::run();
}
