use crate::common::AllMoveData;
use crate::move_generation::{Game, EncodedMove};

use std::cmp::Ordering;
use std::ops::Neg;

#[derive(Eq)]
pub enum Score {
    Checkmate((bool, u32)),
    Draw,
    Playing(i32),
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Score::Playing(0), Score::Draw) | (Score::Draw, Score::Playing(0)) => true,
            (Score::Playing(a), Score::Playing(b)) => a == b,
            (Score::Checkmate((b1, n1)), Score::Checkmate((b2, n2))) => b1 == b2 && n1 == n2,
            (Score::Draw, Score::Draw) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other { return Some(Ordering::Equal) }
        match (self, other) {
            (Score::Checkmate((b1, n1)), Score::Checkmate((b2, n2))) => {
                if b1 == b2 {
                    if *b1 { Some(n1.cmp(n2).reverse()) } else { Some(n1.cmp(n2)) }
                }
                else if *b1 {
                    Some(Ordering::Greater)
                }
                else {
                    Some(Ordering::Less)
                }
            },

            (Score::Checkmate((true, _)), _) => Some(Ordering::Greater),
            (_, Score::Checkmate((true, _))) => Some(Ordering::Less),

            (Score::Checkmate((false, _)), _) => Some(Ordering::Less),
            (_, Score::Checkmate((false, _))) => Some(Ordering::Greater),

            (Score::Playing(a), Score::Playing(b)) => Some(a.cmp(b)),

            (Score::Draw, Score::Playing(x)) => Some(x.cmp(&0).reverse()),
            (Score::Playing(x), Score::Draw) => Some(0.cmp(x).reverse()),

            _ => None,
        }
    }
}

impl Neg for Score {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Score::Checkmate((b, n)) => Score::Checkmate((!b, n)),
            Score::Playing(n) => Score::Playing(-n),
            Score::Draw => Score::Draw,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score() {
        let self_checkmate_far = Score::Checkmate((true, 5));
        let self_checkmate_close = Score::Checkmate((true, 2));
        let self_checkmate = Score::Checkmate((true, 0));

        let other_checkmate_far = Score::Checkmate((false, 5));
        let other_checkmate_close = Score::Checkmate((false, 2));
        let other_checkmate = Score::Checkmate((false, 0));

        let positive_playing = Score::Playing(100);
        let negative_playing = Score::Playing(-100);

        let draw = Score::Draw;

        assert!(other_checkmate_close < other_checkmate_far);
        assert!(self_checkmate_close > self_checkmate_far);

        assert!(self_checkmate_far < self_checkmate_close);
        assert!(self_checkmate_close > self_checkmate_far);

        assert!(self_checkmate_far > other_checkmate_far);
        assert!(other_checkmate_far < self_checkmate_far);

        assert!(self_checkmate > other_checkmate);
        assert!(other_checkmate < self_checkmate);

        assert!(positive_playing > negative_playing);
        assert!(negative_playing < positive_playing);

        assert!(self_checkmate_far > positive_playing);
        assert!(self_checkmate_far > negative_playing);
        assert!(self_checkmate_far > draw);

        assert!(other_checkmate_far < positive_playing);
        assert!(other_checkmate_far < negative_playing);
        assert!(other_checkmate_far < draw);

        assert!(draw == Score::Playing(0));
        assert!(Score::Playing(0) == draw);

        assert!(self_checkmate_far == self_checkmate_far);
    }
}
