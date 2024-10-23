use std::{ops, cmp};
use serde::{Serialize, Deserialize};

pub enum CastleRights {
    WHITE_KING,
    WHITE_QUEEN,
    BLACK_KING,
    BLACK_QUEEN,
}

impl CastleRights {
    pub fn as_int(&self) -> u8 {
        match self {
            Self::WHITE_KING =>     0b1,
            Self::WHITE_QUEEN =>   0b10,
            Self::BLACK_KING =>   0b100,
            Self::BLACK_QUEEN => 0b1000,
        }
    }
    pub fn int_to_castle_rights(i: u8) -> Self {
        match i {
            0b1 => Self::WHITE_KING,
            0b10 => Self::WHITE_QUEEN,
            0b100 => Self::BLACK_KING,
            0b1000 => Self::BLACK_QUEEN,
            _ => panic!("wrong flag passed to int_to_castle_rights"),
        }
    }
}



pub fn direction_to_index(file: i8, rank: i8) -> usize {
    match (file, rank) {
        (1, 0) => 0, // right
        (1, 1) => 1, // right-down
        (0, 1) => 2, // down
        (-1, 1) => 3, // left-down
        (-1, 0) => 4, // left
        (-1, -1) => 5, // left-up
        (0, -1) => 6, // up
        (1, -1) => 7, // right-up
        _ => panic!("invalid direction"),
    }
}

pub fn index_to_direction(index: usize) -> (i8, i8) {
    match index {
        0 => (1, 0),
        1 => (1, 1),
        2 => (0, 1),
        3 => (-1, 1),
        4 => (-1, 0),
        5 => (-1, -1),
        6 => (0, -1),
        7 => (1, -1),
        _ => panic!("index out of range: {}", index),
    }
}

#[derive(Copy, Clone, cmp::PartialEq, Serialize, Deserialize)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn new() -> Self {
        return Self(0);
    }
    pub fn new_set(i: u8) -> Self {
        Self(1 << i)
    }
    pub fn set_bit(&mut self, index: u8) { self.0 |= 1 << index as u64 }
    pub fn get_bit(&self, index: u8) -> bool { (self.0 & (1 << index as u64)) != 0 }
    pub fn pop_bit(&mut self, index: u8) -> bool {
        let result = self.get_bit(index);
        self.0 &= !(1 << index as u64);
        result
    }
    pub fn count_bits(&self) -> u8 {
        let mut count = 0;
        let mut value = self.0;
        while value != 0 {
            count += 1;
            value &= value - 1;
        }
        count
    }
    pub fn ls1b_index(&self) -> Option<u8> {
        if !self.not_zero() { return None }
        let trailing_bits = (*self & -*self) - 1;
        Some(trailing_bits.count_bits())
    }
    pub fn pop_ls1b(&mut self) -> Option<u8> {
        let index = self.ls1b_index()?;
        self.pop_bit(index);
        Some(index)
    }

    pub const SIZE: u8 = 64;
    pub const WIDTH: u8 = 8;
    pub fn display(&self) {
        println!();
        for i in 0..Self::SIZE {
            if i % Self::WIDTH == 0 {
                println!();
            }
            let display_string = if self.get_bit(i as u8) { "X" } else { "-" };
            print!("{}", display_string);
        }
        println!("\n\n0b{:b}", self.0);
        println!("0x{:x}", self.0);
        println!("{}", self.0);
    }

    pub fn not_zero(&self) -> bool { self.0 != 0 }
}

impl ops::Not for BitBoard {
    type Output = Self;
    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl ops::BitOr for BitBoard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl ops::BitAnd for BitBoard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl ops::Shl<u8> for BitBoard {
    type Output = Self;
    fn shl(self, rhs: u8) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl ops::Shr<u8> for BitBoard {
    type Output = Self;
    fn shr(self, rhs: u8) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}

impl ops::BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl ops::Neg for BitBoard {
    type Output = Self;
    fn neg(self) -> Self::Output {
        BitBoard((!self.0).wrapping_add(1))
    }
}

impl ops::Sub<u8> for BitBoard {
    type Output = Self;
    fn sub(self, rhs: u8) -> Self::Output {
        BitBoard(self.0.wrapping_sub(rhs as u64))
    }
}

impl ops::Mul<BitBoard> for BitBoard {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        BitBoard(self.0.wrapping_mul(rhs.0))
    }
}

impl ops::BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl ops::MulAssign<BitBoard> for BitBoard {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_mul(rhs.0);
    }
}

impl ops::ShrAssign<u8> for BitBoard {
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs;
    }
}

pub struct Constants;

impl Constants {
    pub const WHITE: u8 = 0;
    pub const BLACK: u8 = 1;
    pub const BISHOP: u8 = 0;
    pub const ROOK: u8 = 1;
    pub const FILE_NAME: &'static str = "move_data.json";
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SlidingAttackData {
    attacks: Vec<Vec<BitBoard>>,
    magic_numbers: Vec<BitBoard>,
    masks: Vec<BitBoard>,
    relevant_bits: Vec<u8>,
}

impl SlidingAttackData {
    pub fn new(attacks: Vec<Vec<BitBoard>>, magic_numbers: Vec<BitBoard>, masks: Vec<BitBoard>, relevant_bits: Vec<u8>) -> Self {
        Self {
            attacks,
            magic_numbers,
            masks,
            relevant_bits,
        }
    }
    pub fn get_attack(&self, square: u8, occupancy: &BitBoard) -> BitBoard {
        let mut o = BitBoard(occupancy.0);
        o &= self.masks[square as usize];
        o *= self.magic_numbers[square as usize];
        o >>= (64 - self.relevant_bits[square as usize]) as u8;
        self.attacks[square as usize][o.0 as usize]
    }
}

#[derive(Serialize, Deserialize)]
pub struct LeapingAttackData {
    pub pawn_attacks: Vec<Vec<BitBoard>>,
    pub knight: Vec<BitBoard>,
    pub king: Vec<BitBoard>,
    pub pawn_moves: Vec<Vec<BitBoard>>,
}

#[derive(Serialize, Deserialize)]
pub struct AllMoveData {
    bishop_attack_data: SlidingAttackData,
    rook_attack_data: SlidingAttackData,
    leaping_attack_data: LeapingAttackData,
    pawn_single_push_ranks: Vec<BitBoard>,
    pawn_double_push_ranks: Vec<BitBoard>,
    promotion_ranks: Vec<BitBoard>,
    directions: Vec<Vec<BitBoard>>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Pieces {
    KING,
    PAWN,
    KNIGHT,
    BISHOP,
    ROOK,
    QUEEN,
}

impl Pieces {
    pub fn int_to_piece(i: u8) -> Self {
        match i {
            0 => Pieces::PAWN,
            1 => Pieces::KNIGHT,
            2 => Pieces::BISHOP,
            3 => Pieces::ROOK,
            4 => Pieces::QUEEN,
            5 => Pieces::KING,
            _ => panic!("cannot parse int to piece"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Color {
    WHITE,
    BLACK,
}

impl ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Color::WHITE => Color::BLACK,
            Color::BLACK => Color::WHITE,
        }
    }
}

impl AllMoveData {
    pub fn get_attacks(&self, square: u8, piece: &Pieces, side: Color, occupancy: &BitBoard) -> BitBoard {
        match piece {
            Pieces::PAWN => self.leaping_attack_data.pawn_attacks[side as usize][square as usize],
            Pieces::KNIGHT => self.leaping_attack_data.knight[square as usize],
            Pieces::BISHOP => self.bishop_attack_data.get_attack(square, occupancy),
            Pieces::ROOK => self.rook_attack_data.get_attack(square, occupancy),
            Pieces::KING => self.leaping_attack_data.king[square as usize],
            Pieces::QUEEN => {
                let bishop_attack = self.bishop_attack_data.get_attack(square, occupancy);
                let rook_attack =  self.rook_attack_data.get_attack(square, occupancy);
                bishop_attack | rook_attack
            },
        }
    }

    pub fn get_pawn_moves(&self, square: u8, side: Color) -> BitBoard {
        self.leaping_attack_data.pawn_moves[side as usize][square as usize]
    }
    
    pub fn get_promotion_ranks(&self, side: Color) -> BitBoard {
        self.promotion_ranks[side as usize]
    }

    pub fn get_pawn_single_push_ranks(&self, side: Color) -> BitBoard {
        self.pawn_single_push_ranks[side as usize]
    }

    pub fn get_pawn_double_push_ranks(&self, side: Color) -> BitBoard {
        self.pawn_double_push_ranks[side as usize]
    }

    pub fn get_direction(&self, square: u8, file_dir: i8, rank_dir: i8) -> BitBoard {
        self.directions[direction_to_index(file_dir, rank_dir)][square as usize]
    }

    pub fn get_castle_info(&self, rights: CastleRights, side: Color) -> Option<(u8, BitBoard)> {
        match (rights, side) {
            (CastleRights::WHITE_KING, Color::WHITE) => Some((62, BitBoard((1 << 62) | (1 << 61)))),
            (CastleRights::WHITE_QUEEN, Color::WHITE) => Some((58, BitBoard((1 << 59) | (1 << 58)))),
            (CastleRights::BLACK_KING, Color::BLACK) => Some((6, BitBoard((1 << 5) | (1 << 6)))),
            (CastleRights::BLACK_QUEEN, Color::BLACK) => Some((2, BitBoard((1 << 2) | (1 << 3)))),
            _ => None,
        }
    }

    pub fn squares_between(&self, s1: u8, s2: u8) -> BitBoard {
        let file_dir;
        let rank_dir;
        let file1 = s1 % 8;
        let rank1 = s1 / 8;
        let file2 = s2 % 8;
        let rank2 = s2 / 8;

        if file1 > file2 { file_dir = -1; }
        else if file1 < file2 { file_dir = 1; }
        else { file_dir = 0; }

        if rank1 > rank2 { rank_dir = -1; }
        else if rank1 < rank2 { rank_dir = 1; }
        else { rank_dir = 0; }

        self.get_direction(s1, file_dir, rank_dir) & self.get_direction(s2, -file_dir, -rank_dir)
    }
    
    pub fn new(bishop_attack_data: SlidingAttackData, rook_attack_data: SlidingAttackData, leaping_attack_data: LeapingAttackData, promotion_ranks: Vec<BitBoard>, pawn_single_push_ranks: Vec<BitBoard>, pawn_double_push_ranks: Vec<BitBoard>, directions: Vec<Vec<BitBoard>>) -> Self {
        Self {
            bishop_attack_data,
            rook_attack_data,
            leaping_attack_data,
            promotion_ranks,
            pawn_single_push_ranks,
            pawn_double_push_ranks,
            directions,
        }
    }
}
