use crate::common::AllMoveData;
use crate::move_generation::{Game, EncodedMove};

pub struct Engine {
    name: &'static str,
    author: &'static str,
    game: Game,
    move_data: AllMoveData,
}

impl Engine {
    pub fn new(move_data: &AllMoveData) -> Self {
        let game = Game::new();
        Self {
            name: "RICE",
            author: "Parker White",
            game,
            move_data: move_data.clone(),
        }
    }

    pub fn get_move(&self) -> EncodedMove {
        let mut moves = Vec::new();
        self.game.generate_moves(&mut moves, &self.move_data);
        
        moves[0]
    }

    pub fn set_game(&mut self, game: Game) {
        self.game = game;
    }

    pub fn get_id_info(&self) -> (&'static str, &'static str) { (self.name, self.author) }
}
