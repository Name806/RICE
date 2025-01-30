mod engine;
mod common;
mod move_generation;

use move_generation::{Game, EncodedMove, GameState};
use common::{Constants, AllMoveData, Pieces};
use engine::Engine;

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


    let mut game = Game::new_fen(fen.clone());
    
    game.parse_moves(args, all_move_data);

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

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn handle_uci(move_data: &AllMoveData) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut buffer = String::new();
    let mut engine = Engine::new(move_data);
    loop {
        buffer.clear();
        if stdin.read_line(&mut buffer).is_err() { continue; }

        let command = buffer.trim();
        if command.is_empty() { continue; }

        match command {
            "uci" => {
                let (name, author) = engine.get_id_info();
                println!("id name {} author {}", name, author);
                println!("uciok");
            },
            "isready" => println!("readyok"),
            cmd if cmd.starts_with("position") => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                let mut game = Game::new();
                if parts[1] == "startpos" {
                    game = Game::new_fen(String::from(STARTING_FEN));
                } 
                else if parts[1] == "fen" {
                    game = Game::new_fen(parts[2..].join(" "));
                }

                if let Some(moves_index) = parts.iter().position(|&x| x == "moves") {
                    let moves: Vec<String> = parts[moves_index + 1..].iter().map(|s| s.to_string()).collect();
                    game.parse_moves(moves, move_data);
                }
                engine.set_game(game);
            }
            cmd if cmd.starts_with("go") => {
                let m = engine.get_move();
                println!("bestmove {}", m);
            }
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

    handle_uci(&all_move_data);
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
