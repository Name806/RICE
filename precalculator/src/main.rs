mod common;
use common::BitBoard;
use common::Constants;
use common::SlidingAttackData;
use common::LeapingAttackData;
use common::AllMoveData;
use std::{fs::File, io::Write};
use common::Color;
use common::ZobristHashes;

use rand::Rng;

struct NotFiles {
    a: BitBoard,
    h: BitBoard,
    ab: BitBoard,
    gh: BitBoard,
}

fn main() {
    // create bitboards to detect files
    let mut a_file = BitBoard::new();
    let mut b_file = BitBoard::new();
    let mut g_file = BitBoard::new();
    let mut h_file = BitBoard::new();
    for i in 0..8 {
        let rank = i * 8;
        a_file.set_bit(rank);
        b_file.set_bit(rank + 1);

        g_file.set_bit(rank + 8 - 2);
        h_file.set_bit(rank + 8 - 1);
    }
    let not_files = NotFiles {
        a: !a_file,
        h: !h_file,
        ab: !(a_file | b_file),
        gh: !(g_file | h_file),
    };

    // mask leaper attacks and sliding occupancies
    let mut pawn_attacks = vec![vec![BitBoard::new(); 64]; 2];
    let mut knight_attacks = vec![BitBoard::new(); 64];
    let mut king_attacks = vec![BitBoard::new(); 64];
    let mut pawn_moves = vec![vec![BitBoard::new(); 64]; 2];
    
    let mut white_pawn_starting_file = BitBoard::new();
    let mut black_pawn_starting_file = BitBoard::new();
    
    for i in 0..8 {
        black_pawn_starting_file.set_bit(8 + i);
        white_pawn_starting_file.set_bit(48 + i);
    }

    for i in 0..64 {
        let index = i as usize;
        pawn_attacks[Color::WHITE as usize][index] = mask_pawn_attacks(Color::WHITE, i, &not_files);
        pawn_attacks[Color::BLACK as usize][index] = mask_pawn_attacks(Color::BLACK, i, &not_files);
        pawn_moves[Color::WHITE as usize][index] = mask_pawn_moves(Color::WHITE, i, white_pawn_starting_file, black_pawn_starting_file);
        pawn_moves[Color::BLACK as usize][index] = mask_pawn_moves(Color::BLACK, i, white_pawn_starting_file, black_pawn_starting_file);

        knight_attacks[index] = mask_knight_attacks(i, &not_files);
        king_attacks[index] = mask_king_attacks(i, &not_files);
    }

    // relevant bits tables

    let bishop_relevant_bits = vec![ 6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6 ];

        let rook_relevant_bits = vec![12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12 ];

    let mut random_state = 1804289383;

    // initailize magic numbers

    let mut bishop_magic_numbers = vec![BitBoard::new(); 64];
    let mut rook_magic_numbers = vec![BitBoard::new(); 64];

    println!("\nCalculating Rook Magic Numbers (0/64)");
    for square in 0..64 {
        let magic_number = find_magic_number(square, rook_relevant_bits[square as usize], Constants::_ROOK, &mut random_state);
        rook_magic_numbers[square as usize] = magic_number;
        println!("Calculating Rook Magic Numbers ({}/64)", square + 1);
    }

    println!("\nCalculating Bishop Magic Numbers (0/64)");
    for square in 0..64 {
        let magic_number = find_magic_number(square, bishop_relevant_bits[square as usize], Constants::_BISHOP, &mut random_state);
        bishop_magic_numbers[square as usize] = magic_number;
        println!("Calculating Bishop Magic Numbers ({}/64)", square + 1);
    }
    let mut bishop_masks = vec![BitBoard::new(); 64];
    let mut rook_masks = vec![BitBoard::new(); 64];
    let mut bishop_attacks = vec![vec![BitBoard::new(); 512]; 64];
    let mut rook_attacks = vec![vec![BitBoard::new(); 4096]; 64];
    init_sliders_attacks(Constants::_BISHOP, &mut bishop_masks, &mut rook_masks, &mut bishop_attacks, &mut rook_attacks, &bishop_magic_numbers, &bishop_relevant_bits);
    init_sliders_attacks(Constants::_ROOK, &mut bishop_masks, &mut rook_masks, &mut bishop_attacks, &mut rook_attacks, &rook_magic_numbers, &rook_relevant_bits);

    let bishop_attack_data = SlidingAttackData::_new(bishop_attacks, bishop_magic_numbers, bishop_masks, bishop_relevant_bits);
    let rook_attack_data = SlidingAttackData::_new(rook_attacks, rook_magic_numbers, rook_masks, rook_relevant_bits);
    let leaping_attack_data = LeapingAttackData {
        pawn_attacks,
        pawn_moves,
        knight: knight_attacks,
        king: king_attacks,
    };
    let mut promotion_ranks = vec![BitBoard::new(); 2];
    let mut pawn_double_push_ranks = vec![BitBoard::new(); 2];
    let mut pawn_single_push_ranks = vec![BitBoard::new(); 2];
    for i in 0..8 {
        promotion_ranks[Color::WHITE as usize].set_bit(i);
        pawn_single_push_ranks[Color::WHITE as usize].set_bit(i + 40);
        pawn_double_push_ranks[Color::WHITE as usize].set_bit(i + 32);
        promotion_ranks[Color::BLACK as usize].set_bit(i + 56);
        pawn_single_push_ranks[Color::BLACK as usize].set_bit(i + 16);
        pawn_double_push_ranks[Color::BLACK as usize].set_bit(i + 24);
    }

    let mut directions = vec![vec![BitBoard::new(); 64]; 8];
    for i in 0..8 {
        let (file_dir, rank_dir) = common::_index_to_direction(i);
        for square in 0..64 {
            directions[i][square as usize] = squares_in_direction(square, file_dir, rank_dir);
        }
    }

    let all_move_data = AllMoveData::_new(bishop_attack_data, rook_attack_data, leaping_attack_data, promotion_ranks, pawn_single_push_ranks, pawn_double_push_ranks, directions);
    let hashes = generate_hashes();

    //save data for the ai to use later
    println!("Saving Results");
    let json_data = serde_json::to_string_pretty(&all_move_data).unwrap();
    let mut file = File::create(Constants::MOVE_DATA_FILE_NAME).unwrap();
    file.write_all(json_data.as_bytes()).unwrap();

    let json_data = serde_json::to_string_pretty(&hashes).unwrap();
    let mut file = File::create(Constants::HASHES_FILE_NAME).unwrap();
    file.write_all(json_data.as_bytes()).unwrap();
    println!("Results Saved to: {}", Constants::HASHES_FILE_NAME);
}

fn generate_hashes() -> ZobristHashes {
    let mut rng = rand::thread_rng();
    let mut hashes = ZobristHashes::new();
    for piece_index in 0..hashes.pieces.len() {
        for square_index in 0..hashes.pieces[0].len() {
            hashes.pieces[piece_index][square_index] = rng.gen();
        }
    }

    hashes.side = rng.gen();

    for i in 0..hashes.en_passant_file.len() {
        hashes.en_passant_file[i] = rng.gen();
    }

    for i in 0..hashes.castle_rights.len() {
        hashes.castle_rights[i] = rng.gen();
    }

    hashes
}

fn squares_in_direction(square: u8, file_dir: i8, rank_dir: i8) -> BitBoard {
    let mut result = BitBoard::new();
    let mut file = (square as i8 % 8) + file_dir;
    let mut rank = (square as i8 / 8) + rank_dir;
    while file >= 0 && file < 8 && rank >= 0 && rank < 8 {
        let index = (rank * 8) + file;
        result.set_bit(index as u8);
        file += file_dir;
        rank += rank_dir;
    }
    result
}

fn mask_pawn_moves(side: Color, square: u8, white_pawn_starting_file: BitBoard, black_pawn_starting_file: BitBoard) -> BitBoard {
    let mut moves = BitBoard::new();
    if side == Color::WHITE {
        if square < 8 { return moves; } 
        moves.set_bit(square - 8);
        if white_pawn_starting_file.get_bit(square) {
            moves.set_bit(square - 16)
        }
    }
    else if side == Color::BLACK {
        if square >= 56 { return moves; }
        moves.set_bit(square + 8);
        if black_pawn_starting_file.get_bit(square) {
            moves.set_bit(square + 16);
        }
    }
    if (square >= 48 && square < 56) || (square >= 8 && square < 16) {
        println!("square: {}, side: {} \nmoves: {}", square, side as u8, moves);
    }
    moves
}

fn init_sliders_attacks(piece: u8, bishop_masks: &mut Vec<BitBoard>, rook_masks: &mut Vec<BitBoard>, bishop_attacks: &mut Vec<Vec<BitBoard>>, rook_attacks: &mut Vec<Vec<BitBoard>>, magic_numbers: &Vec<BitBoard>, relevant_bits: &Vec<u8>) {
    for square in 0..64 {
        bishop_masks[square] = mask_sliding_occupancy(square as u8, Constants::_BISHOP);
        rook_masks[square] = mask_sliding_occupancy(square as u8, Constants::_ROOK);

        let attack_mask = if piece == Constants::_BISHOP { bishop_masks[square] } else { rook_masks[square] };
        let relevant_bits_count = attack_mask.count_bits();
        let occupancy_indicies = 1 << relevant_bits_count;
        for index in 0..occupancy_indicies {
            let occupancy = set_occupancy(index, relevant_bits_count, &attack_mask);
            if piece == Constants::_BISHOP {
                let magic_index: usize = ((occupancy * magic_numbers[square]) >> (64 - relevant_bits[square])).0 as usize;
                bishop_attacks[square][magic_index] = mask_sliding_attacks(square as u8, piece, &occupancy);
            }
            if piece == Constants::_ROOK {
                let magic_index: usize = ((occupancy * magic_numbers[square]) >> (64 - relevant_bits[square])).0 as usize;
                rook_attacks[square][magic_index] = mask_sliding_attacks(square as u8, piece, &occupancy);
            }
        }
    }
}

fn get_random_number(state: &mut u32) -> u32 {
    let mut number = *state;
    number ^= number << 13;
    number ^= number >> 17;
    number ^= number << 5;
    *state = number;
    number
}

fn get_magic_candidate_part(state: &mut u32) -> u64 {
    let n1 = (get_random_number(state) & 0xFFFF) as u64;
    let n2 = (get_random_number(state) & 0xFFFF) as u64;
    let n3 = (get_random_number(state) & 0xFFFF) as u64;
    let n4 = (get_random_number(state) & 0xFFFF) as u64;

    n1 | (n2 << 16) | (n3 << 32) | (n4 << 48)
}

fn generate_magic_candidate(state: &mut u32) -> BitBoard {
    BitBoard(get_magic_candidate_part(state) & get_magic_candidate_part(state) & get_magic_candidate_part(state))
}

fn find_magic_number(square: u8, relevant_bits: u8, piece: u8, state: &mut u32) -> BitBoard {
    let mut occupancies = vec![BitBoard::new(); 4096];
    let mut attacks = vec![BitBoard::new(); 4096];
    let attack_mask = mask_sliding_occupancy(square, piece);
    let occupancy_indicies = BitBoard(1 << relevant_bits);

    for i in 0..occupancy_indicies.0 {
        let index = i as usize;
        occupancies[index] = set_occupancy(i, relevant_bits, &attack_mask);
        attacks[index] = mask_sliding_attacks(square, piece, &occupancies[index]);
    }

    for _ in 0..100000000 {
        let magic_number = generate_magic_candidate(state);
        if ((attack_mask * magic_number) & BitBoard(0xFF00000000000000)).count_bits() < 6 { continue; }

        let mut used_attacks = vec![BitBoard::new(); 4096];
        let mut fail = false;
        for index in 0..occupancy_indicies.0 {
            let magic_index = ((occupancies[index as usize] * magic_number) >> (64 - relevant_bits)).0 as usize;
            if !used_attacks[magic_index].not_zero() {
                used_attacks[magic_index] = attacks[index as usize];
            }
            else if used_attacks[magic_index].0 != attacks[index as usize].0 {
                fail = true;
                break;
            }
        }
        if !fail {
            return magic_number;
        }
    }

    panic!("magic number failed!: {}", square);
}

fn mask_pawn_attacks(side: Color, index: u8, not_files: &NotFiles) -> BitBoard {
    let mut attacks = BitBoard::new();
    let mut piece_position = BitBoard::new();
    piece_position.set_bit(index);

    if side == Color::WHITE {
        if ((piece_position >> 7) & not_files.a).not_zero() { attacks |= piece_position >> 7; }; 
        if ((piece_position >> 9) & not_files.h).not_zero() { attacks |= piece_position >> 9; };
    }
    else {

        if ((piece_position << 7) & not_files.h).not_zero() { attacks |= piece_position << 7; }; 
        if ((piece_position << 9) & not_files.a).not_zero() { attacks |= piece_position << 9; };
    }

    attacks
}

fn mask_knight_attacks(index: u8, not_files: &NotFiles) -> BitBoard {
    let mut attacks = BitBoard::new();
    let mut piece_position = BitBoard::new();
    piece_position.set_bit(index);

    if ((piece_position >> 17) & not_files.h ).not_zero() { attacks |= piece_position >> 17; };
    if ((piece_position >> 15) & not_files.a ).not_zero() { attacks |= piece_position >> 15; };
    if ((piece_position >> 10) & not_files.gh).not_zero() { attacks |= piece_position >> 10; };
    if ((piece_position >> 6 ) & not_files.ab).not_zero() { attacks |= piece_position >> 6 ; };
    if ((piece_position << 17) & not_files.a ).not_zero() { attacks |= piece_position << 17; };
    if ((piece_position << 15) & not_files.h ).not_zero() { attacks |= piece_position << 15; };
    if ((piece_position << 10) & not_files.ab).not_zero() { attacks |= piece_position << 10; };
    if ((piece_position << 6 ) & not_files.gh).not_zero() { attacks |= piece_position << 6 ; };

    attacks
}

fn mask_king_attacks(index: u8, not_files: &NotFiles) -> BitBoard {
    let mut attacks = BitBoard::new();
    let mut piece_position = BitBoard::new();
    piece_position.set_bit(index);

    if (piece_position >> 8).not_zero() { attacks |= piece_position >> 8 };
    if ((piece_position >> 9) & not_files.h).not_zero() { attacks |= piece_position >> 9 };
    if ((piece_position >> 7) & not_files.a).not_zero() { attacks |= piece_position >> 7 };
    if ((piece_position >> 1) & not_files.h).not_zero() { attacks |= piece_position >> 1 };
    if (piece_position << 8).not_zero() { attacks |= piece_position << 8 };
    if ((piece_position << 9) & not_files.a).not_zero() { attacks |= piece_position << 9 };
    if ((piece_position << 7) & not_files.h).not_zero() { attacks |= piece_position << 7 };
    if ((piece_position << 1) & not_files.a).not_zero() { attacks |= piece_position << 1 };

    attacks
}

fn mask_sliding_occupancy(index: u8, piece: u8) -> BitBoard {
    let mut occupancy = BitBoard::new();
    let start_file = index as i8 % 8;
    let start_rank = index as i8 / 8;

    let (file_dir, rank_dir) = 
    if piece == Constants::_BISHOP {
        (1, 1)
    }
    else if piece == Constants::_ROOK {
        (1, 0)
    }
    else {
        panic!("invalid piece passed to fn mask_sliding_occupancy");
    };
    
    
    occupancy |= mask_occupancy_in_direction(start_file, start_rank, file_dir, rank_dir);
    occupancy |= mask_occupancy_in_direction(start_file, start_rank, -file_dir, -rank_dir);
    occupancy |= mask_occupancy_in_direction(start_file, start_rank, -rank_dir, file_dir);
    occupancy |= mask_occupancy_in_direction(start_file, start_rank, rank_dir, -file_dir);
    occupancy
}

fn mask_occupancy_in_direction(start_file: i8, start_rank: i8, file_dir: i8, rank_dir: i8) -> BitBoard {
    let mut occupancy = BitBoard::new();

    let mut file = start_file + file_dir;
    let mut rank = start_rank + rank_dir;
    while if file_dir == 0 { true } else { file >= 1 && file <= 6 } && if rank_dir == 0 { true } else { rank >= 1 && rank <= 6 } {
        let index = (rank as u8 * 8) + file as u8;
        occupancy.set_bit(index);
        file += file_dir;
        rank += rank_dir;
    }

    occupancy
}

fn mask_sliding_attacks(index: u8, piece: u8, blocking: &BitBoard) -> BitBoard {
    let mut attacks = BitBoard::new();
    let start_file = index as i8 % 8;
    let start_rank = index as i8 / 8;

    let (file_dir, rank_dir) = 
    if piece == Constants::_BISHOP {
        (1, 1)
    }
    else if piece == Constants::_ROOK {
        (1, 0)
    }
    else {
        panic!("invalid piece passed to fn mask_sliding_occupancy");
    };

    attacks |= mask_attacks_in_direction(start_file, start_rank, file_dir, rank_dir, blocking);
    attacks |= mask_attacks_in_direction(start_file, start_rank, -file_dir, -rank_dir, blocking);
    attacks |= mask_attacks_in_direction(start_file, start_rank, -rank_dir, file_dir, blocking);
    attacks |= mask_attacks_in_direction(start_file, start_rank, rank_dir, -file_dir, blocking);

    attacks
}

fn mask_attacks_in_direction(start_file: i8, start_rank: i8, file_dir: i8, rank_dir: i8, blocking: &BitBoard) -> BitBoard {
    let mut attacks = BitBoard::new();

    let mut file = start_file + file_dir;
    let mut rank = start_rank + rank_dir;
    while file >= 0 && rank >= 0 && file <= 7 && rank <= 7 {
        let index = (rank as u8 * 8) + file as u8;
        let mut attack_position = BitBoard::new();
        attack_position.set_bit(index);
        attacks |= attack_position;
        if (attack_position & *blocking).not_zero() { break; }
        file += file_dir;
        rank += rank_dir;
    }

    attacks
}

fn set_occupancy(index: u64, bits_in_mask: u8, attack_mask: &BitBoard) -> BitBoard {
    let mut attack_mask = BitBoard(attack_mask.0);
    let mut occupancy = BitBoard::new();
    for count in 0..bits_in_mask {
        let square = attack_mask.ls1b_index();

        let square = match square {
            None => panic!("no lsb1 index"),
            Some(s) => s,
        };
        
        attack_mask.pop_bit(square);
        if (BitBoard(index as u64 & (1 << count as u64))).not_zero() {
            occupancy |= BitBoard(1 << square as u64);
        }
    }
    occupancy
}
