use crate::common::{Pieces, Color, BitBoard, Constants, CastleRights, AllMoveData, ZobristHashes};
use std::fmt;
    
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
    const NONE: u32 = 0b1111;
    const BIT_COUNTS: [u8; 7] = [6, 12, 16, 20, 24, 25, 26];
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

#[derive(Copy, Clone)]
pub struct EncodedMove(u32);

impl EncodedMove {
    pub fn source_square(&self) -> u8 { (self.0 & 0x000000000000003F) as u8 }
    pub fn target_square(&self) -> u8 { ((self.0 & 0x0000000000000FC0) >> Move::BIT_COUNTS[0]) as u8 }
    pub fn piece_moved(&self) -> Pieces { Pieces::int_to_piece(((self.0 & 0x000000000000F000) >> Move::BIT_COUNTS[1]) as u8) }
    pub fn promoted_piece(&self) -> Option<Pieces> { 
        let promoted = (self.0 & 0x00000000000F0000) >> Move::BIT_COUNTS[2];
        if promoted == Move::NONE { None } else { Some(Pieces::int_to_piece(promoted as u8)) }
    }
    pub fn capture(&self) -> Option<Pieces> { 
        let capture = (self.0 & 0x0000000000F00000) >> Move::BIT_COUNTS[3];
        if capture == Move::NONE { None } else { Some(Pieces::int_to_piece(capture as u8)) }
    }
    pub fn double_push(&self) -> bool { (self.0 & 0x0000000001000000) >> Move::BIT_COUNTS[4] != 0 }
    pub fn en_passant(&self) -> bool { (self.0 &  0x0000000002000000) >> Move::BIT_COUNTS[5] != 0 }
    pub fn castle(&self) -> bool { (self.0 & 0x0000000004000000) >> Move::BIT_COUNTS[6] != 0 }
}

impl fmt::Display for EncodedMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start_file = self.source_square() % 8;
        let start_rank = 7 - (self.source_square() / 8);
        let end_file = self.target_square() % 8;
        let end_rank = 7 - (self.target_square() / 8);
        let mut movement_string = format!("{}{}{}{}", (b'a' + start_file) as char, (b'1' + start_rank) as char, (b'a' + end_file) as char, (b'1' + end_rank) as char);
        if let Some(promotion_piece) = self.promoted_piece() {
            match promotion_piece {
                Pieces::QUEEN => movement_string.push('q'),
                Pieces::KNIGHT => movement_string.push('n'),
                Pieces::ROOK => movement_string.push('r'),
                Pieces::BISHOP => movement_string.push('b'),
                _ => panic!("promotion piece wrong"),
            }
        }
        write!(f, "{}", movement_string)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::from("\n");
        result.push_str(&format!("white occupancy: {} \nblack occupancy: {} \nboth_occupancy: {}", self.occupancies[Color::WHITE as usize], self.occupancies[Color::BLACK as usize], self.occupancies[Constants::BOTH_OCCUPANCIES]));
        'square: for square in 0..64 {
            if square % 8 == 0 {
                result.push('\n');
            }
            for side in 0..2 {
                for piece in 0..6 {
                    if self.piece_positions[side as usize][piece as usize].get_bit(square) {
                        result.push_str(match (piece, side) {
                            (0, 0) => "K",
                            (1, 0) => "P",
                            (2, 0) => "N",
                            (3, 0) => "B",
                            (4, 0) => "R",
                            (5, 0) => "Q",
                            (0, 1) => "k",
                            (1, 1) => "p",
                            (2, 1) => "n",
                            (3, 1) => "b",
                            (4, 1) => "r",
                            (5, 1) => "q",
                            _ => panic!("out of bounds: side: {}, piece: {}", side, piece)
                        });
                        continue 'square;
                    }
                }
            }
            result.push_str("-");
        }

        result.push_str("\n");
        let side_string = match self.side {
            Color::WHITE => String::from("w"),
            Color::BLACK => String::from("b"),
        };
        result.push_str(&format!("side: {}, castle rights: {:b}", side_string, self.castle_rights));

        write!(f, "{}", result)
    }
}

struct HistoryEntry {
    move_made: EncodedMove,
    en_passant: Option<u8>,
    castle_rights: u8,
    halfmove_timer: u8,
    fullmove_number: u16,
    capture_square: u8,
    rook_movement: (u8, u8),
    hash: u64,
}

pub struct Game {
    pub piece_positions: Vec<Vec<BitBoard>>,
    pub occupancies: Vec<BitBoard>,
    pub side: Color,
    en_passant: Option<u8>,
    castle_rights: u8,
    halfmove_timer: u8,
    fullmove_number: u16,
    history: Vec<HistoryEntry>,
    move_data: AllMoveData,
    hashes: ZobristHashes,
    hash: u64,
}

pub enum GameState {
    Draw,
    Checkmate,
    Normal,
}

impl Game {
    pub fn new(move_data: &AllMoveData, hashes: &ZobristHashes) -> Self {
        Self {
            piece_positions: vec![vec![BitBoard::new(); 6]; 2],
            occupancies: vec![BitBoard::new(); 3],
            side: Color::WHITE,
            en_passant: None,
            castle_rights: 0b1111,
            halfmove_timer: 0,
            fullmove_number: 0,
            history: Vec::new(),
            move_data: move_data.clone(),
            hashes: hashes.clone(),
            hash: 0,
        }
    }

    pub fn unmake_move(&mut self) {
        let history = self.history.pop().expect("unmake move called on game with no history");

        let piece_moved = history.move_made.piece_moved();
        let source_square = history.move_made.source_square();
        let target_square = history.move_made.target_square();

        // update by reversing move

        self.side = !self.side;
        self.piece_positions[self.side as usize][piece_moved as usize].move_bit(target_square, source_square);
        self.occupancies[self.side as usize].move_bit(target_square, source_square);
        self.occupancies[Constants::BOTH_OCCUPANCIES].move_bit(target_square, source_square);

        if let Some(captured_piece) = history.move_made.capture() {
            self.piece_positions[!self.side as usize][captured_piece as usize].set_bit(history.capture_square);
            self.occupancies[!self.side as usize].set_bit(history.capture_square);
        self.occupancies[Constants::BOTH_OCCUPANCIES].set_bit(history.capture_square);
        }
        
        if history.move_made.castle() {
            let (rook_source, rook_target) = history.rook_movement;
            self.piece_positions[self.side as usize][Pieces::ROOK as usize].move_bit(rook_target, rook_source);
            self.occupancies[self.side as usize].move_bit(rook_target, rook_source);
            self.occupancies[Constants::BOTH_OCCUPANCIES].move_bit(rook_target, rook_source);
        }

        if let Some(promoted_piece) = history.move_made.promoted_piece() {
            self.piece_positions[self.side as usize][promoted_piece as usize].pop_bit(target_square);
            self.piece_positions[self.side as usize][Pieces::PAWN as usize].set_bit(target_square);
        }

        // copy values

        self.castle_rights = history.castle_rights;
        self.en_passant = history.en_passant;
        self.fullmove_number = history.fullmove_number;
        self.halfmove_timer = history.halfmove_timer;
        self.hash = history.hash;
    }

    pub fn make_move(&mut self, move_made: &EncodedMove) {
        self.hash ^= self.hashes.castle_rights[self.castle_rights as usize];
        let moved_piece = move_made.piece_moved();
        let move_source = move_made.source_square();
        let move_target = move_made.target_square();

        // variables to keep track of game history
        let mut capture_square = move_target;
        let mut rook_movement = (0, 0);

        // update piece position
        self.piece_positions[self.side as usize][moved_piece as usize].move_bit(move_source, move_target);
        self.toggle_piece_hash(self.side, moved_piece, move_source);
        self.toggle_piece_hash(self.side, moved_piece, move_target);

        // update occupancy for moved piece
        self.occupancies[self.side as usize].move_bit(move_source, move_target);
        self.occupancies[Constants::BOTH_OCCUPANCIES].move_bit(move_source, move_target);

        // update if capture
        let move_capture = move_made.capture();
        if let Some(piece_captured) = move_capture {
            self.piece_positions[!self.side as usize][piece_captured as usize].pop_bit(move_target);
            self.occupancies[!self.side as usize].pop_bit(move_target);
            self.toggle_piece_hash(!self.side, piece_captured, move_target);

            if piece_captured == Pieces::ROOK {
                if let Some(used_rights) = match move_target {
                    0 => Some(CastleRights::BlackQueen),
                    7 => Some(CastleRights::BlackKing),
                    56 => Some(CastleRights::WhiteQueen),
                    63 => Some(CastleRights::WhiteKing),
                    _ => None,
                } {
                    let new_castle_rights = self.castle_rights & !used_rights.as_int();
                    self.castle_rights = new_castle_rights;
                }
            }
        }

        // promotion
        if let Some(promoted_piece) = move_made.promoted_piece() {
            self.piece_positions[self.side as usize][Pieces::PAWN as usize].pop_bit(move_target);
            self.toggle_piece_hash(self.side, Pieces::PAWN, move_target);
            self.piece_positions[self.side as usize][promoted_piece as usize].set_bit(move_target);
            self.toggle_piece_hash(self.side, promoted_piece, move_target);
        }

        // clear old en passant hash
        if let Some(en_passant_square) = self.en_passant {
            self.hash ^= self.hashes.en_passant_file[(en_passant_square % 8) as usize];
        }

        // update en passant target
        if move_made.double_push() {
            let mut target_position = move_target;
            if self.side == Color::WHITE {
                target_position += 8;
            }
            else {
                target_position -= 8;
            }
            self.en_passant = Some(target_position);
            self.hash ^= self.hashes.en_passant_file[(target_position % 8) as usize];
        }
        else {
            self.en_passant = None;
        }

        if move_made.en_passant() {
            let mut enemy_pawn_square = move_target;
            if self.side == Color::WHITE {
                enemy_pawn_square += 8;
            }
            else {
                enemy_pawn_square -= 8;
            }
            self.piece_positions[!self.side as usize][Pieces::PAWN as usize].pop_bit(enemy_pawn_square);
            self.occupancies[!self.side as usize].pop_bit(enemy_pawn_square);
            self.occupancies[Constants::BOTH_OCCUPANCIES].pop_bit(enemy_pawn_square);
            self.toggle_piece_hash(!self.side, Pieces::PAWN, enemy_pawn_square);
            capture_square = enemy_pawn_square;
        }

        if moved_piece == Pieces::KING {
            let used_rights = match self.side {
                Color::WHITE => 0b0011,
                Color::BLACK => 0b1100,
            };
            let new_castle_rights = self.castle_rights & !used_rights;
            self.castle_rights = new_castle_rights;
        }
        else if moved_piece == Pieces::ROOK {
            if let Some(used_rights) = match move_source {
                0 => Some(CastleRights::BlackQueen),
                7 => Some(CastleRights::BlackKing),
                56 => Some(CastleRights::WhiteQueen),
                63 => Some(CastleRights::WhiteKing),
                _ => None,
            } {
                let new_castle_rights = self.castle_rights & !used_rights.as_int();
                self.castle_rights = new_castle_rights;
            }
        }
        
        if move_made.castle() {
            let (rook_source_square, rook_target_square) = Self::rook_movement(move_target);
            self.piece_positions[self.side as usize][Pieces::ROOK as usize].move_bit(rook_source_square, rook_target_square);
            self.occupancies[self.side as usize].move_bit(rook_source_square, rook_target_square);
            self.occupancies[Constants::BOTH_OCCUPANCIES].move_bit(rook_source_square, rook_target_square);
            rook_movement = (rook_source_square, rook_target_square);
            self.toggle_piece_hash(self.side, Pieces::ROOK, rook_source_square);
            self.toggle_piece_hash(self.side, Pieces::ROOK, rook_target_square);
        }

        if moved_piece == Pieces::PAWN || move_capture.is_some() {
            self.halfmove_timer = 0;
        }
        else {
            self.halfmove_timer += 1;
        }
        if self.side == Color::BLACK {
            self.fullmove_number += 1;
        }
        self.side = !self.side;
        self.hash ^= self.hashes.side;

        self.hash ^= self.hashes.castle_rights[self.castle_rights as usize];

        // save game to history
        self.history.push(HistoryEntry {
            move_made: move_made.clone(),
            castle_rights: self.castle_rights.clone(),
            en_passant: self.en_passant.clone(),
            fullmove_number: self.fullmove_number.clone(),
            halfmove_timer: self.halfmove_timer.clone(),
            capture_square,
            rook_movement,
            hash: self.hash,
        });
    }

    fn rook_movement(king_target_square: u8) -> (u8, u8) {
        match king_target_square {
            62 => (63, 61),
            58 => (56, 59),
            6 => (7, 5),
            2 => (0, 3),
            _ => panic!("king tried to castle from invalid position"),
        }
    }

    fn toggle_piece_hash(&mut self, color: Color, piece: Pieces, square: u8) {
        self.hash ^= self.hashes.pieces[color as usize * 6 + piece as usize][square as usize];
    }

    pub fn new_fen(fen: String, move_data: &AllMoveData, hashes: &ZobristHashes) -> Self {
        let mut game = Game::new(move_data, hashes);
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
                        game.toggle_piece_hash(color, p, index);
                        index += 1;
                    }
                }
            }
            
            let side = match *ply {
                "w" => Color::WHITE,
                "b" => {
                    game.hash ^= hashes.side;
                    Color::BLACK
                },
                &_ => panic!("ply does not match"),
            };
            
            game.side = side;
            
            game.castle_rights = 0;
            for c in castle_rights.chars() {
                let cr = match c {
                    'K' => CastleRights::WhiteKing.as_int(),
                    'Q' => CastleRights::WhiteQueen.as_int(),
                    'k' => CastleRights::BlackKing.as_int(),
                    'q' => CastleRights::BlackQueen.as_int(),
                    '-' => break,
                    _ => panic!("castle rights does not match"),
                };
                
                game.castle_rights |= cr;
            }

            game.hash ^= hashes.castle_rights[game.castle_rights as usize];
            
            if *en_passant_target != "-" {
                let file = en_passant_target.chars().nth(0).unwrap() as u8 - b'a';
                let rank = 7 - (en_passant_target.chars().nth(1).unwrap() as u8 - b'1');
                let index = (rank * 8) + file;
                game.hash ^= hashes.en_passant_file[file as usize];
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

    pub fn parse_moves(&mut self, moves: Vec<String>) -> Result<(), String> {
        for m in moves.clone() {
            // this code fixes the crashing in perftree, but it shouldn't be needed.
            if m.len() < 4 || m.len() > 5 {
                return Err(format!("move is wrong size: {}", m));
            }
            let mut chars = m.trim().chars();
            let file_char = chars.next().unwrap() as u8;
            let rank_char = chars.next().unwrap() as u8;
            if file_char < b'a'  || rank_char < b'1' || rank_char - b'1' > 7 {
                continue;
            }
            let file = file_char - b'a';
            let rank = 7 - (rank_char - b'1');
            let from_index = (rank * 8) + file;
            let file = chars.next().unwrap() as u8 - b'a';
            let rank = 7 - (chars.next().unwrap() as u8 - b'1');
            let to_index = (rank * 8) + file;
            let mut promotion_piece: Option<Pieces> = None;
            if let Some(promotion_char) = chars.next() {
                let promotion_char = promotion_char as u8;
                if promotion_char != 32 {
                    promotion_piece = match promotion_char {
                        b'q' | b'Q' => Some(Pieces::QUEEN),
                        b'n' | b'N' => Some(Pieces::KNIGHT),
                        b'r' | b'R' => Some(Pieces::ROOK),
                        b'b' | b'B' => Some(Pieces::BISHOP),
                        _ => panic!("promotion char is not valid: {}", promotion_char),
                    };
                }
            }
            let mut move_options = Vec::new();
            self.generate_moves(&mut move_options);
            for move_option in move_options {
                if move_option.source_square() == from_index && move_option.target_square() == to_index && move_option.promoted_piece() == promotion_piece {
                    self.make_move(&move_option);
                    break;
                }
            }
        }
        Ok(())
    }

    fn get_attacked_squares(&self, side: Color, occupancy: &BitBoard) -> BitBoard {
        let mut attacked = BitBoard::new();

        for piece in 0..6 {
            let mut piece_positions = self.piece_positions[side as usize][piece as usize];
            while piece_positions.not_zero() {
                let index = piece_positions.pop_ls1b().unwrap();
                let piece = Pieces::int_to_piece(piece);
                let attacks = self.move_data.get_attacks(index, &piece, side, &occupancy);
                attacked |= attacks;
            }
        }

        attacked
    }

    pub fn generate_moves(&self, moves: &mut Vec<EncodedMove>) -> GameState {
        if self.halfmove_timer > 100 {
            return GameState::Draw;
        }
        let side_to_move = self.side as usize;
        let both_occupancy = self.occupancies[BOTH_OCCUPANCIES];
        let king_position = self.piece_positions[side_to_move][Pieces::KING as usize];

        let king_square = king_position.ls1b_index().expect("King not found!");
        let mut king_attacks = self.move_data.get_attacks(king_square, &Pieces::KING, self.side, &both_occupancy);
        king_attacks &= !self.occupancies[side_to_move];
        let without_king_occupancy = both_occupancy & !king_position;
        let king_danger_squares = self.get_attacked_squares(!self.side, &without_king_occupancy);
        king_attacks &= !king_danger_squares;
        
        let mut checking_pieces = BitBoard::new();
        for piece in 0..6 {
            if piece == Pieces::KING as u8 { continue; }
            let attacks_from_king = self.move_data.get_attacks(king_square, &Pieces::int_to_piece(piece), self.side, &both_occupancy);
            checking_pieces |= self.piece_positions[!self.side as usize][piece as usize] & attacks_from_king;
        }
        
        let num_checking = checking_pieces.count_bits();

        self.add_moves(moves, king_square, &king_attacks, Pieces::KING, false);
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
            block_mask = self.move_data.squares_between(king_square, checker_square);
        }
        else {
            for i in 0..4 {
                let flag = 1 << i;
                let castle_rights = CastleRights::int_to_castle_rights(flag);
                if self.castle_rights & flag == 0 { continue; }
                if let Some((target_square, move_squares)) = self.move_data.get_castle_info(castle_rights, self.side) {
                    if (move_squares & king_danger_squares).not_zero() { continue; }
                    let (source_square, _) = Self::rook_movement(target_square);
                    if (self.move_data.squares_between(source_square, king_square) & self.occupancies[Constants::BOTH_OCCUPANCIES]).not_zero() { continue; }
                    castle_attacks |= BitBoard::new_set(target_square);
                }
            }
        }

        self.add_moves(moves, king_square, &castle_attacks, Pieces::KING, true);

        let queen_attacks_from_king = self.move_data.get_attacks(king_square, &Pieces::QUEEN, self.side, &both_occupancy);

        let mut pieces_to_ignore = BitBoard::new();
        const SLIDING_PIECES: [Pieces; 3] = [Pieces::BISHOP, Pieces::ROOK, Pieces::QUEEN];
        for sliding_piece in SLIDING_PIECES {
            let mut opponent_positions = self.piece_positions[!self.side as usize][sliding_piece as usize];
            while let Some(opponent_square) = opponent_positions.pop_ls1b() {
                let opponent_attacks = self.move_data.get_attacks(opponent_square, &sliding_piece, !self.side, &both_occupancy);
                if !(self.move_data.squares_between(opponent_square, king_square)).not_zero() { continue; }
                let pinned_pieces = opponent_attacks & queen_attacks_from_king;
                self.calculate_pinned_moves(moves, &mut pieces_to_ignore, opponent_square, &pinned_pieces, king_square, &both_occupancy, &capture_mask, &block_mask);
            }
        }

        for piece in 0..6 {
            let piece_type = Pieces::int_to_piece(piece);
            if piece_type == Pieces::KING { continue; }
            let mut piece_position = self.piece_positions[self.side as usize][piece as usize] & !pieces_to_ignore;
            while let Some(piece_square) = piece_position.pop_ls1b() {
                let piece_attacks = self.get_legal_attacks(piece_square, piece_type, &both_occupancy, &block_mask, &capture_mask, king_square);
                self.add_moves(moves, piece_square, &piece_attacks, piece_type, false);
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

    fn calculate_pinned_moves(&self, moves: &mut Vec<EncodedMove>, pieces_to_ignore: &mut BitBoard, opponent_square: u8, pinned_pieces: &BitBoard, king_square: u8, both_occupancy: &BitBoard, block_mask: &BitBoard, capture_mask: &BitBoard) {
        let between_king_and_opponent = self.move_data.squares_between(opponent_square, king_square);
        if !between_king_and_opponent.not_zero() { return; }
        for piece in 0..6 {
            let piece_type = Pieces::int_to_piece(piece);
            if piece_type == Pieces::KING { continue; }
            let mut pinned_pieces_of_type = self.get_piece_positions(self.side, piece_type) & *pinned_pieces & between_king_and_opponent;
            while let Some(pinned_square) = pinned_pieces_of_type.pop_ls1b() {
                let mut pinned_position = BitBoard::new();
                pinned_position.set_bit(pinned_square);
                let opponent_position = BitBoard::new_set(opponent_square);

                let pinned_movement = self.move_data.squares_between(opponent_square, king_square) | opponent_position;
                let piece_attacks = self.get_legal_attacks(pinned_square, piece_type, &both_occupancy, block_mask, capture_mask, king_square);
                let pinned_attacks = pinned_movement & piece_attacks;
                self.add_moves(moves, pinned_square, &pinned_attacks, piece_type, false);
                *pieces_to_ignore |= BitBoard::new_set(pinned_square);
            }
        }
    }

    fn check_en_passant_special_case(&self, king_square: u8, both_occupancy: &BitBoard, piece_attacks: &BitBoard, en_passant_square: u8) -> bool {
        if !(*piece_attacks & BitBoard::new_set(en_passant_square)).not_zero() { return false; }
        let passant_rank = self.move_data.get_pawn_double_push_ranks(!self.side);
        if !(passant_rank & BitBoard::new_set(king_square)).not_zero() { return false; }
        const STRAIGHT_SLIDING_PIECES: [Pieces; 2] = [Pieces::ROOK, Pieces::QUEEN];
        for piece in STRAIGHT_SLIDING_PIECES {
            let mut enemy_positions = self.piece_positions[!self.side as usize][piece as usize] & passant_rank;
            while let Some(enemy_square) = enemy_positions.pop_ls1b() {
                if !(BitBoard::new_set(enemy_square) & passant_rank).not_zero() { continue; }
                if (self.move_data.squares_between(king_square, enemy_square) & *both_occupancy).count_bits() == 2 {
                    return true;
                }
            }
        }
        return false;
    }

    fn get_legal_attacks(&self, square: u8, piece_type: Pieces, both_occupancy: &BitBoard, block_mask: &BitBoard, capture_mask: &BitBoard, king_square: u8) -> BitBoard {
        let mut piece_attacks = self.move_data.get_attacks(square, &piece_type, self.side, both_occupancy);
        if piece_type == Pieces::PAWN {
            let mut pawn_attack_mask = self.occupancies[!self.side as usize] & *capture_mask;
            if let Some(en_passant_square) = self.en_passant {
                let double_pushed_piece_square = match !self.side {
                    Color::WHITE => en_passant_square - 8,
                    Color::BLACK => en_passant_square + 8,
                };
                if (BitBoard::new_set(double_pushed_piece_square) & *capture_mask).not_zero() {
                    pawn_attack_mask.set_bit(en_passant_square);
                }
                if self.check_en_passant_special_case(king_square, both_occupancy, &piece_attacks, en_passant_square) {
                    pawn_attack_mask.pop_bit(en_passant_square);
                }
            }
            let pawn_attacks = piece_attacks & pawn_attack_mask;
            let mut pawn_movement = self.move_data.get_pawn_moves(square, self.side);
            if pawn_movement.count_bits() == 2 && (*both_occupancy & (self.move_data.get_pawn_single_push_ranks(self.side) & pawn_movement)).not_zero() {
                pawn_movement = BitBoard::new();
            }
            pawn_movement &= *block_mask;
            pawn_movement &= !self.occupancies[!self.side as usize];
            piece_attacks = pawn_attacks | pawn_movement;
        }
        else {
            piece_attacks &= *block_mask | *capture_mask;
        }
        piece_attacks &= !self.occupancies[self.side as usize];
        piece_attacks
    }

    fn _google_en_passant() {
        println!("holy hell!");
    }

    fn get_piece_positions(&self, side: Color, piece: Pieces) -> BitBoard{
        self.piece_positions[side as usize][piece as usize]
    }

    
    fn add_moves(&self, all_moves: &mut Vec<EncodedMove>, source_square: u8, target_squares: &BitBoard, piece_moved: Pieces, castle: bool) {
        let mut target_squares = target_squares.clone();
        'target_square: while let Some(target_square) = target_squares.pop_ls1b() {
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
            if piece_moved == Pieces::PAWN {
                if (target_position & self.move_data.get_promotion_ranks(self.side)).not_zero() {
                    let mut moves: Vec<EncodedMove> = Vec::new();
                    const PROMOTION_OPTIONS: [Pieces; 4] = [Pieces::QUEEN, Pieces::BISHOP, Pieces::ROOK, Pieces::KNIGHT];
                    for promoted_piece in PROMOTION_OPTIONS {
                        moves.push(Move {
                            source_square,
                            target_square,
                            piece_moved,
                            capture,
                            promoted_piece: Some(promoted_piece),
                            en_passant,
                            double_push,
                            castle,
                        }.encode());
                    }
                    all_moves.extend(moves.clone());
                    continue 'target_square;
                }

                if let Some(en_passant_square) = self.en_passant {
                    if (target_position & BitBoard::new_set(en_passant_square)).not_zero() {
                        en_passant = true;
                    }
                }
                if (target_position & self.move_data.get_pawn_double_push_ranks(self.side)).not_zero() && ((8..16).contains(&source_square) || (48..56).contains(&source_square)) {
                        double_push = true;
                }
            }
            if en_passant {
                capture = Some(Pieces::PAWN);
            }
            let calculated_move = Move {
                source_square,
                target_square,
                piece_moved,
                capture,
                promoted_piece: None,
                en_passant,
                double_push,
                castle,
            }.encode();
            all_moves.push(calculated_move);
        }
    }
}
