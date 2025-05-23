mod transposition_table;
mod move_heuristics;
mod eval_data;

use move_heuristics::MoveSorter;
use eval_data::EvalData;
use transposition_table::{TranspositionTable, Bound};

use crate::common::{AllMoveData, Pieces, Constants, Color, ZobristHashes};
use crate::move_generation::{Game, EncodedMove, GameState};
use crate::score::Score;

use std::cmp::{max, min};

pub struct Engine {
    name: &'static str,
    author: &'static str,
    game: Game,
    move_data: AllMoveData,
    transposition_table: TranspositionTable,
}

impl Engine {
    pub fn new(move_data: &AllMoveData, hashes: &ZobristHashes) -> Self {
        let game = Game::new_starting(move_data, hashes);
        Self {
            name: "RICE",
            author: "Parker White",
            game,
            move_data: move_data.clone(),
            transposition_table: TranspositionTable::new(),
        }
    }

    pub fn get_move(&self) -> EncodedMove {
        let mut moves = Vec::new();
        self.game.generate_moves(&mut moves, false);
        
        moves[0]
    }

    pub fn set_game(&mut self, game: Game) {
        self.game = game;
    }

    pub fn game_string(&self) -> String {
        format!("{}", self.game)
    }

    pub fn get_id_info(&self) -> (&'static str, &'static str) { (self.name, self.author) }

    pub fn search_to_depth(&mut self, depth: u8) {
        self.search(depth, 0);
    }

    pub fn get_best_found_move(&self) -> EncodedMove {
        if let Some(game_entry) = self.transposition_table.lookup(self.game.hash, 0) {
            if let Some(m) = game_entry.best_move {
                return m;
            }
        }

        self.get_move()
    }

    pub fn no_search_best_eval(&mut self) -> EncodedMove {
        let mut moves = Vec::new();
        self.game.generate_moves(&mut moves, false);
        let mut best_move = moves[0];
        let mut max_eval = Score::Checkmate((false, 0));
        for m in moves {
            self.game.make_move(m);
            let eval = self.evaluate_position();
            self.game.unmake_move(m);
            if eval > max_eval {
                best_move = m;
                max_eval = eval;
            }
        }
        best_move
    }

    fn evaluate_position(&self) -> Score {
        Score::Playing(self.evaluate_side(Color::WHITE) - self.evaluate_side(Color::BLACK))
    }

    fn evaluate_side(&self, side: Color) -> i32 {
        let mut score = 0;
        for piece_index in 0..6 {
            let mut piece_positions = self.game.piece_positions[side as usize][piece_index];
            while let Some(piece_square) = piece_positions.pop_ls1b() {
                score += EvalData::MATERIAL_VALUE[piece_index];
                let piece_type = Pieces::int_to_piece(piece_index as u8);
                let num_controlled_squares = self.move_data.get_attacks(piece_square, &piece_type, side, &self.game.occupancies[Constants::BOTH_OCCUPANCIES]).count_bits();
                score += num_controlled_squares as i32 * EvalData::MOBILITY_VALUE[piece_index];
            }
        }

        score
    }

    fn search(&mut self, depth: u8, ply: u32) -> Score {
        // check transposition table
        if let Some(entry) = self.transposition_table.lookup(self.game.hash, depth) {
            if matches!(entry.bound, Bound::Exact) {
                return entry.score;
            }
        }

        if depth == 0 {
            return self.evaluate_position();
        }

        let mut moves = Vec::new();
        let game_state = self.game.generate_moves(&mut moves, false);
        match game_state {
            GameState::Checkmate => return Score::Checkmate((false, ply)),
            GameState::Draw => return Score::Draw,
            GameState::Normal => (),
        }

        let mut max_score = Score::NEG_INF;
        let mut best_move = moves[0];
        for m in moves {
            self.game.make_move(m);
            let score = -self.search(depth - 1, ply + 1);
            self.game.unmake_move(m);

            if score > max_score { 
                max_score = score; 
                best_move = m;
            }
        }

        self.transposition_table.store(self.game.hash, max_score, Bound::Exact, depth, Some(best_move));
        max_score
    }
}


