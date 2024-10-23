mod common;
use common::Constants;
use common::BitBoard;
use common::SlidingAttackData;
use common::LeapingAttackData;
use common::AllMoveData;
use common::CastleRights;
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

enum GameState {
    Draw,
    Checkmate,
    Normal,
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

    fn deep_clone(old_game: &Self) -> Self {
        let mut new_game = Self::new();
        for i in 0..old_game.piece_positions.len() {
            for j in 0..old_game.piece_positions[i].len() {
                new_game.piece_positions[i][j] = BitBoard(old_game.piece_positions[j][i].0.clone());
            }
        }

        for i in 0..old_game.occupancies.len() {
            new_game.occupancies[i] = BitBoard(old_game.occupancies[i].0);
        }

        new_game.side = match old_game.side {
            Color::WHITE => Color::WHITE,
            Color::BLACK => Color::BLACK,
        };

        new_game.en_passant = match old_game.en_passant {
            Some(index) => Some(index.clone()),
            None => None,
        };

        new_game.castle_rights = old_game.castle_rights.clone();
        new_game.halfmove_timer = old_game.halfmove_timer.clone();
        new_game.fullmove_number = old_game.fullmove_number.clone();

        new_game
    }

    fn make_move(&self, encoded_move: &EncodedMove) -> Self {
        let chess_move = encoded_move.decode();
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

    fn generate_moves(&self, moves: &mut Vec<EncodedMove>, move_data: &AllMoveData) -> GameState {
        if self.halfmove_timer > 100 {
            return GameState::Draw;
        }
        let side_to_move = self.side as usize;
        let both_occupancy = self.occupancies[BOTH_OCCUPANCIES];
        let king_position = self.piece_positions[side_to_move][Pieces::KING as usize];

        let king_square = king_position.ls1b_index().expect("King not found!");
        let mut king_attacks = move_data.get_attacks(king_square, &Pieces::KING, self.side, &both_occupancy);
        king_attacks &= !self.occupancies[side_to_move];
        let without_king_occupancy = both_occupancy & !king_position;
        let king_danger_squares = self.get_attacked_squares(!self.side, move_data, &without_king_occupancy);
        king_attacks &= !king_danger_squares;
        
        let mut checking_pieces = BitBoard::new();
        for piece in 0..6 {
            if piece == Pieces::KING as u8 { continue; }
            let attacks_from_king = move_data.get_attacks(king_square, &Pieces::int_to_piece(piece), !self.side, &both_occupancy);
            checking_pieces |= self.piece_positions[!self.side as usize][piece as usize] & attacks_from_king;
        }
        
        let num_checking = checking_pieces.count_bits();

        self.add_moves(moves, king_square, &king_attacks, Pieces::KING, false, &move_data);
        if num_checking > 1 {
            if moves.len() == 0 {
                return GameState::Checkmate;
            }
            return GameState::Normal;
        }
        let mut capture_mask = BitBoard(0xFFFFFFFFFFFFFFFF);
        let mut block_mask = BitBoard(0xFFFFFFFFFFFFFFFF);
        let mut castle_attacks = BitBoard::new();
        if num_checking == 1 {
            capture_mask = checking_pieces;
            let checker_square = checking_pieces.ls1b_index().expect("checking_pieces should not be empty");
            block_mask = move_data.squares_between(king_square, checker_square);
        }
        else {
            for i in 0..4 {
                let flag = 1 << i;
                let castle_rights = CastleRights::int_to_castle_rights(flag);
                if self.castle_rights & flag == 0 { continue; }
                if let Some((target_square, move_squares)) = move_data.get_castle_info(castle_rights, self.side) {
                    if (move_squares & king_danger_squares).not_zero() { continue; }
                    castle_attacks |= BitBoard::new_set(target_square);
                }
            }
        }

        self.add_moves(moves, king_square, &castle_attacks, Pieces::KING, true, &move_data);

        let queen_attacks_from_king = move_data.get_attacks(king_square, &Pieces::QUEEN, self.side, &both_occupancy);

        let mut pieces_to_ignore = BitBoard::new();
        const SLIDING_PIECES: [Pieces; 3] = [Pieces::BISHOP, Pieces::ROOK, Pieces::QUEEN];
        for sliding_piece in SLIDING_PIECES {
            let mut opponent_positions = self.piece_positions[!self.side as usize][sliding_piece as usize];
            while let Some(opponent_square) = opponent_positions.pop_ls1b() {
                let opponent_attacks = move_data.get_attacks(opponent_square, &sliding_piece, !self.side, &both_occupancy);
                let pinned_pieces = opponent_attacks & queen_attacks_from_king;
                self.calculate_pinned_moves(moves, &mut pieces_to_ignore, opponent_square, &pinned_pieces, king_square, &both_occupancy, &capture_mask, &block_mask, move_data);
            }
        }

        for piece in 0..6 {
            let piece_type = Pieces::int_to_piece(piece);
            if piece_type == Pieces::KING { continue; }
            let mut piece_position = self.piece_positions[self.side as usize][piece as usize] & !pieces_to_ignore;
            while let Some(piece_square) = piece_position.pop_ls1b() {
                let piece_attacks = self.get_legal_attacks(piece_square, piece_type, &both_occupancy, &block_mask, &capture_mask, king_square, move_data);
                self.add_moves(moves, piece_square, &piece_attacks, piece_type, false, move_data);
            }
        }
        if moves.len() == 0 {
            if num_checking != 0 {
                return GameState::Checkmate;
            }
            else {
                return GameState::Draw;
            }
        }
        return GameState::Normal;
    }

    fn calculate_pinned_moves(&self, moves: &mut Vec<EncodedMove>, pieces_to_ignore: &mut BitBoard, opponent_square: u8, pinned_pieces: &BitBoard, king_square: u8, both_occupancy: &BitBoard, block_mask: &BitBoard, capture_mask: &BitBoard, move_data: &AllMoveData) {
        for piece in 0..6 {
            let piece_type = Pieces::int_to_piece(piece);
            if piece_type == Pieces::KING { continue; }
            let mut pinned_pieces_of_type = self.get_piece_positions(self.side, piece_type) & *pinned_pieces;
            while let Some(pinned_square) = pinned_pieces_of_type.pop_ls1b() {
                let mut pinned_position = BitBoard::new();
                pinned_position.set_bit(pinned_square);
                let opponent_position = BitBoard::new_set(opponent_square);

                let pinned_movement = move_data.squares_between(opponent_square, king_square) | opponent_position;
                let piece_attacks = self.get_legal_attacks(pinned_square, piece_type, &both_occupancy, block_mask, capture_mask, king_square, &move_data);
                let pinned_attacks = pinned_movement & piece_attacks;
                self.add_moves(moves, pinned_square, &pinned_attacks, piece_type, false, move_data);
                *pieces_to_ignore |= BitBoard::new_set(pinned_square);
            }
        }
    }

    fn check_en_passant_special_case(&self, king_square: u8, both_occupancy: &BitBoard, piece_attacks: &BitBoard, passant_position: &BitBoard, move_data: &AllMoveData) -> bool {
        if !(*piece_attacks & *passant_position).not_zero() { return false; }
        let passant_rank = move_data.get_pawn_double_push_ranks(!self.side);
        if !(passant_rank & BitBoard::new_set(king_square)).not_zero() { return false; }
        const STRAIGHT_SLIDING_PIECES: [Pieces; 2] = [Pieces::ROOK, Pieces::QUEEN];
        for piece in STRAIGHT_SLIDING_PIECES {
            let mut enemy_positions = self.piece_positions[!self.side as usize][piece as usize] & passant_rank;
            while let Some(enemy_square) = enemy_positions.pop_ls1b() {
                if !(BitBoard::new_set(enemy_square) & passant_rank).not_zero() { continue; }
                if (move_data.squares_between(king_square, enemy_square) & *both_occupancy).count_bits() != 2 {
                    return false;
                }
            }
        }
        return true;
    }

    fn get_legal_attacks(&self, square: u8, piece_type: Pieces, both_occupancy: &BitBoard, block_mask: &BitBoard, capture_mask: &BitBoard, king_square: u8, move_data: &AllMoveData) -> BitBoard {
        let mut piece_attacks = move_data.get_attacks(square, &piece_type, self.side, both_occupancy);
        if piece_type == Pieces::PAWN {
            let mut pawn_attack_mask = self.occupancies[!self.side as usize] & *capture_mask;
            if let Some(en_passant_square) = self.en_passant {
                let passant_position = BitBoard::new_set(en_passant_square);
                pawn_attack_mask |= passant_position;
                if self.check_en_passant_special_case(king_square, both_occupancy, &piece_attacks, &passant_position, move_data) {
                    pawn_attack_mask = BitBoard::new();
                }
            }
            let pawn_attacks = piece_attacks & pawn_attack_mask;
            let mut pawn_movement = move_data.get_pawn_moves(square, self.side);
            if pawn_movement.count_bits() == 2 && (*both_occupancy & (move_data.get_pawn_single_push_ranks(self.side) & pawn_movement)).not_zero() {
                pawn_movement = BitBoard::new();
            }
            pawn_movement &= *block_mask;
            piece_attacks = pawn_attacks | pawn_movement;
        }
        else {
            piece_attacks &= *block_mask | *capture_mask;
        }
        piece_attacks &= !self.occupancies[self.side as usize];
        piece_attacks
    }

    fn google_en_passant() {
        println!("holy hell!");
        // i actually think I can add this to add_moves, if the piece that is moving is a pawn and
        // it is capturing en_passant_target, then it is en_passant. also, I need to make sure when
        // I generate pawn attacks that I let them attack en passant squares.
    }

    fn get_piece_positions(&self, side: Color, piece: Pieces) -> BitBoard{
        self.piece_positions[side as usize][piece as usize]
    }

    
    fn add_moves(&self, all_moves: &mut Vec<EncodedMove>, source_square: u8, target_squares: &BitBoard, piece_moved: Pieces, castle: bool, move_data: &AllMoveData) {
        let mut moves: Vec<EncodedMove> = Vec::new();
        let mut target_squares = target_squares.clone();
        while target_squares.not_zero() {
            let target_square = target_squares.pop_ls1b().expect("target_squares not zero");
            let mut target_position = BitBoard::new();
            target_position.set_bit(target_square);
            let mut capture: Option<Pieces> = None;
            
            for piece in 0..6 {
                if (self.piece_positions[!self.side as usize][piece] & target_position).not_zero() {
                    capture = Some(Pieces::int_to_piece(piece as u8));
                    break;
                }
            }
            let mut en_passant = false;
            let mut double_push = false;
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
                    return;
                }

                if let Some(en_passant_square) = self.en_passant {
                    if (target_position & BitBoard::new_set(en_passant_square)).not_zero() {
                        en_passant = true;
                    }
                }
                if (target_position & move_data.get_pawn_double_push_ranks(self.side)).not_zero() {
                    double_push = true;
                }
            }
            if en_passant {
                capture = Some(Pieces::PAWN);
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
        all_moves.extend(moves);
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
