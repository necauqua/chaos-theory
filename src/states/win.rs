use crate::states::main_game::MainGame;
use crate::ChaosTheory;
use ld_game_engine::GameState;

#[derive(Debug)]
pub struct Win {}

impl Win {
    pub fn new(main_game: MainGame) -> Self {
        Self {}
    }
}

impl GameState<ChaosTheory> for Win {}
