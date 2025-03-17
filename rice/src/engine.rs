use crate::common::{AllMoveData, EvalData, Pieces, Constants, Color, ZobristHashes};
use crate::move_generation::{Game, EncodedMove};
use crate::score::Score;

pub struct Engine {
    name: &'static str,
    author: &'static str,
    game: Game,
    move_data: AllMoveData,
    eval_data: EvalData,
}

impl Engine {
    pub fn new(move_data: &AllMoveData, hashes: &ZobristHashes) -> Self {
        let game = Game::new(move_data, hashes);
        Self {
            name: "RICE",
            author: "Parker White",
            game,
            move_data: move_data.clone(),
            eval_data: EvalData::new(),
        }
    }

    pub fn get_move(&self) -> EncodedMove {
        let mut moves = Vec::new();
        self.game.generate_moves(&mut moves);
        
        moves[0]
    }

    pub fn set_game(&mut self, game: Game) {
        self.game = game;
    }

    pub fn get_id_info(&self) -> (&'static str, &'static str) { (self.name, self.author) }

    fn evaluate_position(&self) -> Score {
        Score::Playing(self.evaluate_side(self.game.side) - self.evaluate_side(!self.game.side))
    }

    fn evaluate_side(&self, side: Color) -> i32 {
        let mut score = 0;
        for piece_index in 0..6 {
            let mut piece_positions = self.game.piece_positions[side as usize][piece_index];
            while let Some(piece_square) = piece_positions.pop_ls1b() {
                score += self.eval_data.material_values[piece_index];
                let piece_type = Pieces::int_to_piece(piece_index as u8);
                let num_controlled_squares = self.move_data.get_attacks(piece_square, &piece_type, side, &self.game.occupancies[Constants::BOTH_OCCUPANCIES]).count_bits();
                score += num_controlled_squares as i32 * self.eval_data.mobility_values[piece_index];
            }
        }

        score
    }
}

pub enum Bound {
    Exact,
    Lower,
    Upper,

}

pub struct TranspositionEntry {
    hash_num: u64,
    depth: u8,
    score: Score,
    bound: Bound,
}

pub struct TranspositionTable(Vec<TranspositionEntry>);

