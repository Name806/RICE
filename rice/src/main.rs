mod common;
use common::Constants;
use common::BitBoard;
use common::SlidingAttackData;
use common::LeapingAttackData;
use common::AllMoveData;
use std::fs::File;
use std::io::{self, Read};

enum Color {
    WHITE,
    BLACK,
}

enum Pieces {
    PAWN,
    KNIGHT,
    BISHOP,
    ROOK,
    QUEEN,
    KING,
}

enum CastleRights {
    WHITE_KING,
    WHITE_QUEEN,
    BLACK_KING,
    BLACK_QUEEN,
}

impl CastleRights {
    fn as_int(&self) -> u8 {
        match self {
            Self::WHITE_KING =>     0b1,
            Self::WHITE_QUEEN =>   0b10,
            Self::BLACK_KING =>   0b100,
            Self::BLACK_QUEEN => 0b1000,
        }
    }
}

struct Game {
    piece_positions: Vec<BitBoard>,
    occupancies: Vec<BitBoard>,
    side: Color,
    en_passant: bool,
    castle_rights: u8,
}

impl Game {
    fn new() -> Self {
        Self {
            piece_positions: vec![BitBoard::new(); 12],
            occupancies: vec![BitBoard::new(); 3],
            side: Color::WHITE,
            en_passant: false,
            castle_rights: 0b1111,
        }
    }

    fn new_fen() -> Self {

    }
}

fn main() {
    let mut file = File::open(Constants::FILE_NAME).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let all_move_data: AllMoveData = serde_json::from_str(&contents).unwrap();
    println!("loaded move data");
}
