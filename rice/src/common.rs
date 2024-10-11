use std::{ops, cmp};
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, cmp::PartialEq, Serialize, Deserialize)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn new() -> Self {
        return Self(0);
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
}

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
}
