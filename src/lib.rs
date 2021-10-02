
use wasm_bindgen::prelude::*;
use ld_game_engine::{Game, GameRun, GameState, Resources, util::setup_panic_hook};
use crate::states::main_game::MainGame;

pub mod states;
pub mod rope;

#[derive(Debug)]
pub struct ChaosTheory {

}

impl Game for ChaosTheory {
    type Storage = ();

    fn load(_resources: Resources) -> (Self, Box<dyn GameState<Self>>) {
        let global = ChaosTheory {};
        let initial_state = Box::new(MainGame::new());
        (global, initial_state)
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    setup_panic_hook();

    ChaosTheory::run();
}
