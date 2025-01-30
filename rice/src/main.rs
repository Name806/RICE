mod common;
mod move_generation;

use move_generation::{Game, EncodedMove, GameState};
use common::{Constants, AllMoveData, Pieces};

use std::fs::File;
use std::io::Read;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::io;

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

fn perftree(args: Vec<String>, all_move_data: &AllMoveData) {
    let depth: u32 = args[2].parse().expect("depth should be a number");
    let fen = &args[3];

    let moves = if args.len() > 4 {
        args[4..].join(" ").split_whitespace().map(String::from).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let mut game = Game::new_fen(fen.clone());
    for m in moves {
        let mut chars = m.trim().chars();
        let file = chars.next().unwrap() as u8 - b'a';
        let rank = 7 - (chars.next().unwrap() as u8 - b'1');
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
        game.generate_moves(&mut move_options, all_move_data);
        for move_option in move_options {
            if move_option.source_square() == from_index && move_option.target_square() == to_index && move_option.promoted_piece() == promotion_piece {
                game.make_move(&move_option);
                break;
            }
        }
    }

    let mut move_options = Vec::new();
    game.generate_moves(&mut move_options, all_move_data);
    let mut total_nodes = 0;

    let mut log_file = OpenOptions::new().create(true).append(true).open("perftree_output.log").expect("failed to open log file");
    let mut output = String::new();

    for game_move in move_options {
        let mut move_nodes = 0;
        if depth > 0 {
            game.make_move(&game_move);
            move_nodes = count_nodes(depth - 1, &mut game, all_move_data);
            game.unmake_move();
        }
        total_nodes += move_nodes;
        output.push_str(&format!("{} {}\n", game_move, move_nodes));
    }

    output.push('\n');
    output.push_str(&format!("{}\n", total_nodes));
    output = output.trim().to_string();
    write!(io::stdout(), "{}", output).expect("failed to write to stdout");
    write!(log_file, "{}", output).expect("failed to write to log");
}

const _STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn handle_uci() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut buffer = String::new();
    loop {
        buffer.clear();
        if stdin.read_line(&mut buffer).is_err() { continue; }

        let command = buffer.trim();
        if command.is_empty() { continue; }

        match command {
            "uci" => println!("id name NAME"),
            "isready" => println!("readyok"),
            "quit" => return,
            _ => (),
        }

        stdout.flush().unwrap();
    }
}

fn main() {
    let mut file = File::open(Constants::FILE_NAME).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let all_move_data: AllMoveData = serde_json::from_str(&contents).unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "perftree" {
        perftree(args, &all_move_data);
        return;
    }

    handle_uci();
}

fn count_nodes(depth: u32, game: &mut Game, move_data: &AllMoveData) -> u32 {
    if depth == 0 { return 1; }

    let mut move_list : Vec<EncodedMove> = Vec::new();
    let game_state = game.generate_moves(&mut move_list, move_data);
    if matches!(game_state, GameState::Draw | GameState::Checkmate) { return 0; }
    let mut node_count = 0;
    for new_move in move_list.iter() {
        game.make_move(new_move);
        node_count += count_nodes(depth - 1, game, move_data);
        game.unmake_move();
    }
    node_count
}
