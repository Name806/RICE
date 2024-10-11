mod common;
use common::Constants;
use common::BitBoard;
use common::SlidingAttackData;
use common::LeapingAttackData;
use common::AllMoveData;
use std::fs::File;
use std::io::{self, Read};
use common::{Color, Pieces};


impl Pieces {
    fn parse_ascii(a: char) -> Option<Self> {
        match a {
            'p' | 'P' => Some(Self::PAWN),
            'n' | 'N' => Some(Self::KNIGHT),
            'b' | 'B' => Some(Self::BISHOP),
            'r' | 'R' => Some(Self::ROOK),
            'q' | 'Q' => Some(Self::QUEEN),
            'k' | 'K' => Some(Self::KING),
            _ => None,
            }
    }
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

const BOTH_OCCUPANCIES: usize = 2;

struct Game {
    piece_positions: Vec<Vec<BitBoard>>,
    occupancies: Vec<BitBoard>,
    side: Color,
    en_passant: Option<u8>,
    castle_rights: u8,
    halfmove_timer: u8,
    fullmove_number: u16,
}

impl Game {
    fn new() -> Self {
        Self {
            piece_positions: vec![vec![BitBoard::new(); 6]; 2],
            occupancies: vec![BitBoard::new(); 3],
            side: Color::WHITE,
            en_passant: None,
            castle_rights: 0b1111,
            halfmove_timer: 0,
            fullmove_number: 0,
        }
    }

    fn new_fen(fen: String) -> Self {
        let mut game = Game::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if let [piece_positions, ply, castle_rights, en_passant_target, halfmove_timer, fullmove_number] = parts.as_slice() {
            let rows = piece_positions.split("/");
            let mut index = 0;
            for r in rows {
                let characters = r.chars();
                for c in characters {
                    let ascii_c = c as u8;
                    let color: Color;
                    let piece = Pieces::parse_ascii(c);
                    if ascii_c >= b'a' && ascii_c <= b'z' {
                        color = Color::BLACK;
                    }
                    else if ascii_c >= b'A' && ascii_c <= b'Z' {
                        color = Color::WHITE;
                    }
                    else if ascii_c >= b'0' && ascii_c <= b'9' {
                        index += ascii_c - b'0';
                        continue;
                    }
                    else { 
                        panic!("wrong character in piece positions string: {}", c); 
                    }
                    if let Some(p) = piece {
                        game.piece_positions[color as usize][p as usize].set_bit(index);
                        game.occupancies[color as usize].set_bit(index);
                        game.occupancies[BOTH_OCCUPANCIES].set_bit(index);
                        index += 1;
                    }
                }
            }
            
            let side = match *ply {
                "w" => Color::WHITE,
                "b" => Color::BLACK,
                &_ => panic!("ply does not match"),
            };
            
            game.side = side;
            
            for c in castle_rights.chars() {
                let cr = match c {
                    'K' => CastleRights::WHITE_KING.as_int(),
                    'Q' => CastleRights::WHITE_QUEEN.as_int(),
                    'k' => CastleRights::BLACK_KING.as_int(),
                    'q' => CastleRights::BLACK_QUEEN.as_int(),
                    _ => panic!("castle rights does not match"),
                };
                
                game.castle_rights |= cr;
            }
            
            if *en_passant_target != "-" {
                let file = en_passant_target.chars().nth(1).unwrap() as u8 - b'a';
                let rank = en_passant_target.chars().nth(2).unwrap() as u8 - b'0';
                let index = (rank * 8) + file;
                game.en_passant = Some(index);
            }
            
            game.halfmove_timer = halfmove_timer.parse::<u8>().unwrap();
            game.fullmove_number = fullmove_number.parse::<u16>().unwrap();
        }
        else {
            panic!("fen does not have the right number of space-separated parts");
        }
        game
    }

    fn get_attacked_squares(&self, move_data: &AllMoveData) -> BitBoard {
        let mut attacked = BitBoard::new();

        for piece in 0..6 {
            let mut piece_positions = self.piece_positions[self.side as usize][piece as usize];
            while piece_positions.not_zero() {
                let index = piece_positions.pop_ls1b().unwrap();
                let occupancy = self.occupancies[BOTH_OCCUPANCIES];
                let piece = Pieces::int_to_piece(piece);
                let attacks = move_data.get_attacks(index, piece, self.side, &occupancy);
                let attacks = attacks & !self.occupancies[self.side as usize];
                attacked |= attacks;
            }
        }

        attacked
    }
}

const starting_fen: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 0";

fn main() {
    let mut file = File::open(Constants::FILE_NAME).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let all_move_data: AllMoveData = serde_json::from_str(&contents).unwrap();
    println!("loaded move data");
    
    let starting_game = Game::new_fen(String::from(starting_fen));
    starting_game.piece_positions[Color::BLACK as usize][Pieces::PAWN as usize].display();
    println!("castle rights: {:0b}", starting_game.castle_rights);

    starting_game.get_attacked_squares(&all_move_data).display();
}
