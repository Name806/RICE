use crate::common::AllMoveData;
use crate::move_generation::{Game, EncodedMove};

use std::cmp::Ordering;

#[derive(Eq)]
pub enum Score {
    Checkmate((bool, u32)),
    Draw,
    Playing(u32),
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
