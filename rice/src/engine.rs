mod transposition_table;

use transposition_table::{TranspositionTable, Bound, TranspositionEntry};
use crate::common::{AllMoveData, EvalData, Pieces, Constants, Color, ZobristHashes};
use crate::move_generation::{Game, EncodedMove, GameState};
use crate::score::Score;
use std::cmp::{max, min};

pub struct Engine {
    name: &'static str,
    author: &'static str,
    game: Game,
    move_data: AllMoveData,
    eval_data: EvalData,
    transposition_table: TranspositionTable,
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
            transposition_table: TranspositionTable::new(),
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

    // call with -inf, inf for full search
    fn negamax(&mut self, depth: u8, ply: u32, mut alpha: Score, mut beta: Score) -> Score {
        let original_alpha = alpha;

        if let Some(entry) = self.transposition_table.lookup(self.game.hash, depth) {
            match entry.bound {
                EXACT => return entry.score,
                LOWER => alpha = max(alpha, entry.score),
                UPPER => beta = min(beta, entry.score),
            }
        }

        if depth == 0 {
            return self.quiescence(ply, alpha, beta);
        }

        let mut moves = Vec::new();
        let game_state = self.game.generate_moves(&mut moves);
        match game_state {
            GameState::Checkmate => return Score::Checkmate((false, ply)),
            GameState::Draw => return Score::Draw,
            GameState::Normal => (),
        }

        // todo: sort moves
        let mut value = Score::Draw;
        for m in moves {
            self.game.make_move(&m);
            let new_value = -self.negamax(depth - 1, ply + 1, -beta, -alpha);
            self.game.unmake_move();
            if new_value > value { value = new_value; }
            if value > alpha { alpha = value; }
            if alpha > beta { break; }
        }

        let bound = if value <= original_alpha {
            Bound::Upper
        } else if value >= beta {
            Bound::Lower
        }
        else {
            Bound::Exact
        };

        self.transposition_table.store(self.game.hash, value, bound, depth);

        value
    }

    fn quiescence(&mut self, ply: u32, mut alpha: Score, beta: Score) -> Score {
        let stand_pat = self.evaluate_position();
        let mut best_value = stand_pat;
        if stand_pat >= beta { return stand_pat; }
        if alpha < stand_pat { alpha = stand_pat; }

        let mut moves = Vec::new();
        let game_state = self.game.generate_moves(&mut moves);
        match game_state {
            GameState::Checkmate => return Score::Checkmate((false, ply)),
            GameState::Draw => return Score::Draw,
            GameState::Normal => (),
        }

        // todo: sort moves

        for m in moves {
            if !(m.capture().is_some() || m.promoted_piece().is_some() || m.castle()) {
                continue;
            }

            self.game.make_move(&m);
            let score = -self.quiescence(ply + 1, -beta, -alpha);
            self.game.unmake_move();

            if score >= beta { return score; }
            if score > best_value { best_value = score; }
            if score > alpha { alpha = score; }
        }

        best_value
    }
}


