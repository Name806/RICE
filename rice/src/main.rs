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

struct Move {
    source_square: u8,
    target_square: u8,
    piece_moved: Pieces,
    promoted_piece: Option<Pieces>,
    capture: Option<Pieces>,
    double_push: bool,
    en_passant: bool,
    castle: bool,
}


impl Move {
    const NONE: u32 = 1111;
    const BIT_COUNTS: [u8; 7] = [6, 12, 10, 14, 18, 19, 20];
    fn encode(self) -> EncodedMove {
        let mut move_code = 0_u32;

        move_code |= self.source_square as u32;

        move_code |= (self.target_square as u32) << Move::BIT_COUNTS[0];

        move_code |= (self.piece_moved as u32) << Move::BIT_COUNTS[1];

        if let Some(p) = self.promoted_piece {
            move_code |= (p as u32) << Move::BIT_COUNTS[2];
        }
        else {
            move_code |= Move::NONE << Move::BIT_COUNTS[2];
        }

        if let Some(c) = self.capture {
            move_code |= (c as u32) << Move::BIT_COUNTS[3];
        }
        else {
            move_code |= Move::NONE << Move::BIT_COUNTS[3];
        }

        move_code |= (self.double_push as u32) << Move::BIT_COUNTS[4];
        move_code |= (self.en_passant as u32) << Move::BIT_COUNTS[5];
        move_code |= (self.castle as u32) << Move::BIT_COUNTS[6];

        EncodedMove(move_code)
    }
}

struct EncodedMove(u32);

impl EncodedMove {
    fn decode(self) -> Move {
        let source_square =   self.0 & 0b00000000000000000000000000111111;
        let source_square = source_square as u8;
        let target_square =  (self.0 & 0b00000000000000000000111111000000) >> Move::BIT_COUNTS[0];
        let target_square = target_square as u8;
        let piece_moved =    (self.0 & 0b00000000000000001111000000000000) >> Move::BIT_COUNTS[1];
        let piece_moved = Pieces::int_to_piece(piece_moved as u8);
        let promoted_piece = (self.0 & 0b00000000000011110000000000000000) >> Move::BIT_COUNTS[2];
        let promoted_piece = match promoted_piece {
            Move::NONE => None,
            _ => Some(Pieces::int_to_piece(promoted_piece as u8)),
        };
        let capture =        (self.0 & 0b00000000111100000000000000000000) >> Move::BIT_COUNTS[3];
        let capture = match capture {
            Move::NONE => None,
            _ => Some(Pieces::int_to_piece(capture as u8)),
        };
        let double_push =    (self.0 & 0b00000001000000000000000000000000) >> Move::BIT_COUNTS[4] != 0;
        let en_passant =     (self.0 & 0b00000010000000000000000000000000) >> Move::BIT_COUNTS[5] != 0;
        let castle =         (self.0 & 0b00000100000000000000000000000000) >> Move::BIT_COUNTS[6] != 0;
        Move {
            source_square,
            target_square,
            piece_moved,
            promoted_piece,
            capture,
            double_push,
            en_passant,
            castle,
        }
    }
}

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

    fn get_attacked_squares(&self, side: Color, move_data: &AllMoveData, occupancy: &BitBoard) -> BitBoard {
        let mut attacked = BitBoard::new();

        for piece in 0..6 {
            let mut piece_positions = self.piece_positions[self.side as usize][piece as usize];
            while piece_positions.not_zero() {
                let index = piece_positions.pop_ls1b().unwrap();
                let piece = Pieces::int_to_piece(piece);
                let attacks = move_data.get_attacks(index, &piece, self.side, &occupancy);
                attacked |= attacks;
            }
        }

        attacked
    }

    fn get_moves(&self, move_data: &AllMoveData) -> Vec<EncodedMove> {
        let mut moves = Vec::new();

        let side_to_move = self.side as usize;
		let both_occupancy = self.occupancies[BOTH_OCCUPANCIES];
		let king_position = self.piece_positions[side_to_move][Pieces::KING as usize];

        let king_square = king_position.ls1b_index().expect("King not found!");
		let mut king_attacks = move_data.get_attacks(king_square, &Pieces::KING, self.side, &both_occupancy);
		king_attacks &= !self.occupancies[side_to_move];
		let without_king_occupancy = both_occupancy & !king_position;
		let king_danger_squares = self.get_attacked_squares(!self.side, move_data, &without_king_occupancy);
		king_attacks |= !king_danger_squares;
		
		let mut checking_pieces = BitBoard::new();
		for piece in 0..6 {
            if piece == Pieces::KING as u8 { continue; }
			let attacks_from_king = move_data.get_attacks(king_square, &Pieces::int_to_piece(piece), !self.side, &both_occupancy);
			checking_pieces |= self.piece_positions[!self.side as usize][piece as usize] & attacks_from_king;
		}
		
		let num_checking = checking_pieces.count_bits();

        moves.extend(self.board_to_moves(king_square, &king_attacks, Pieces::KING, false, false, false, &move_data));
		if num_checking > 1 {
			return moves;
		}
        let mut capture_mask = BitBoard(0xFFFFFFFFFFFFFFFF);
        let mut block_mask = BitBoard(0xFFFFFFFFFFFFFFFF);
        if num_checking == 1 {
            capture_mask = checking_pieces;
            let checker_square = checking_pieces.ls1b_index().expect("checking_pieces should not be empty");
            if (checking_pieces & self.piece_positions[!self.side as usize][Pieces::QUEEN as usize]).not_zero() || (checking_pieces & self.piece_positions[!self.side as usize][Pieces::BISHOP as usize]).not_zero() || (checking_pieces & self.piece_positions[!self.side as usize][Pieces::ROOK as usize]).not_zero() {
                // push mask = squares between king and attacker
                block_mask = move_data.squares_between(king_square, checker_square);
            }
            else {
                block_mask = BitBoard::new();
            }
        }

        let queen_attacks_from_king = move_data.get_attacks(king_square, &Pieces::QUEEN, self.side, &both_occupancy);

        let mut bishop_positions = self.piece_positions[!self.side as usize][Pieces::BISHOP as usize];
        let mut rook_positions = self.piece_positions[!self.side as usize][Pieces::ROOK as usize];
        let mut queen_positions = self.piece_positions[!self.side as usize][Pieces::QUEEN as usize];

        let mut bishop_pins = BitBoard::new();
        let mut rook_pins = BitBoard::new();
        while bishop_positions.not_zero() || queen_positions.not_zero() || rook_positions.not_zero() {
            if let Some(bishop_square) = bishop_positions.pop_ls1b() {
                let bishop_attacks = move_data.get_attacks(bishop_square, &Pieces::BISHOP, !self.side, &both_occupancy);
                bishop_pins |= bishop_attacks & queen_attacks_from_king;
                // loop through pieces, if piece is pinned, calculate its attacks and remove it
            }
            if let Some(queen_square) = queen_positions.pop_ls1b() {
                let queen_bishop_attacks = move_data.get_attacks(queen_square, &Pieces::BISHOP, !self.side, &both_occupancy);
                let queen_rook_attacks = move_data.get_attacks(queen_square, &Pieces::ROOK, !self.side, &both_occupancy);
                bishop_pins |= queen_bishop_attacks & queen_attacks_from_king;
                rook_pins |= queen_rook_attacks & queen_attacks_from_king;
            }
            if let Some(rook_square) = rook_positions.pop_ls1b() {
                let rook_attacks = move_data.get_attacks(rook_square, &Pieces::ROOK, !self.side, &both_occupancy);
                rook_pins |= rook_attacks & queen_attacks_from_king;
            }

        }
        moves
    }

    fn calculate_pinned_moves(&self, pinning_positions: &BitBoard, piece_type: Pieces) -> Vec<EncodedMove> {

    }
	
	fn board_to_moves(&self, source_square: u8, target_squares: &BitBoard, piece_moved: Pieces, en_passant: bool, double_push: bool, castle: bool, move_data: &AllMoveData) -> Vec<EncodedMove> {
		let mut moves: Vec<EncodedMove> = Vec::new();
		let mut target_squares = target_squares.clone();
		while target_squares.not_zero() {
			let target_square = target_squares.pop_ls1b().expect("target_squares not zero");
			let mut target_position = BitBoard::new();
			target_position.set_bit(target_square);
			let mut capture: Option<Pieces> = None;
			if en_passant {
				capture = Some(Pieces::PAWN);
			}
			for piece in 0..6 {
				if (self.piece_positions[!self.side as usize][piece] & target_position).not_zero() {
					capture = Some(Pieces::int_to_piece(piece as u8));
					break;
				}
			}
			if piece_moved.clone() == Pieces::PAWN {
				if (target_position & move_data.get_promotion_ranks(self.side)).not_zero() {
					const promotion_options: [Pieces; 4] = [Pieces::QUEEN, Pieces::BISHOP, Pieces::ROOK, Pieces::KNIGHT];
					for promoted_piece in promotion_options {
						moves.push(Move {
							source_square,
							target_square,
							piece_moved: piece_moved.clone(),
							capture,
							promoted_piece: Some(promoted_piece),
							en_passant,
							double_push,
							castle,
						}.encode());
					}
					return moves;
				}
			}
			moves.push(Move {
				source_square,
				target_square,
				piece_moved,
				capture,
				promoted_piece: None,
				en_passant,
				double_push,
				castle,
			}.encode());
		}
		moves
	}
}

const starting_fen: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";

fn main() {
    let mut file = File::open(Constants::FILE_NAME).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let all_move_data: AllMoveData = serde_json::from_str(&contents).unwrap();
    println!("loaded move data");
    
    let starting_game = Game::new_fen(String::from(starting_fen));
    starting_game.piece_positions[Color::BLACK as usize][Pieces::PAWN as usize].display();
    println!("castle rights: {:0b}", starting_game.castle_rights);

}
