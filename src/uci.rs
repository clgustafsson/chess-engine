use crate::{
    board::{Position, Result},
    init::initialize_engine,
    perft::bench,
    search::stop_search_immediately,
    ucigo::{search_position_from_time_info, ucigo},
};
use std::io::{stdin, BufRead, Write};

//handles all uci communication, the thread will block during all non thead safe operations and
//all thread safe operations will be handled in a diffent thread
//this ensures thread safety and allows the user to stop searches in a different thread as searches wont
//block input handling
pub fn uci() {
    let mut initialized = false;

    let mut pos = Position::startpos();

    for line_res in stdin().lock().lines() {
        let line = line_res.unwrap();

        if line == "uci" {
            println!("id name chessengine\nid author linusg\nuciok");
        } else if line == "isready" {
            if !initialized {
                initialize_engine();
                initialized = true;
            }
            println!("readyok");
        } else if line == "position startpos" {
            pos = Position::startpos();
        } else if let Some(stripped) = line.strip_prefix("position startpos moves") {
            pos = Position::from_move_string(stripped);
        } else if let Some(stripped) = line.strip_prefix("position fen") {
            if let Some(i) = line.find("moves") {
                let fen = &line["position fen".len()..i].trim();
                let moves = &line[i + "moves".len()..].trim();
                pos = Position::from_fen_move_string(fen, moves);
            } else {
                pos = Position::from_fen(stripped.trim());
            }
        } else if let Some(stripped) = line.strip_prefix("go") {
            if !initialized {
                println!("Engine must be initialized before starting a search with \"isready\"");
            } else {
                ucigo(&pos, stripped);
            }
        } else if let Some(stripped) = line.strip_prefix("bench") {
            if !initialized {
                println!("Engine must be initialized before starting a bench with \"isready\"");
            } else if let Ok(depth) = stripped.trim().parse() {
                bench(&mut pos, depth);
            } else {
                println!("unexpected format use bench <depth>");
            }
        } else if line == "stop" {
            stop_search_immediately()
        } else if line == "board" {
            pos.print();
        } else if line == "fen" {
            println!("{}", pos.fen());
        } else if line == "state" {
            if !initialized {
                println!("Engine must be initialized before calculating state with \"isready\"");
            } else {
                //legal moves must be calculated to update result
                pos.legal_moves();
                match pos.result {
                    Result::None => println!("ongoing"),
                    Result::Checkmate => println!("checkmate"),
                    Result::Draw => println!("draw"),
                }
            }
        } else if line == "legal moves" {
            if !initialized {
                println!("Engine must be initialized before generating moves with \"isready\"");
            } else {
                pos.print_legal_moves();
            }
        } else if line == "quit" {
            break;
        } else {
            println!("unknown command");
        }
    }
}
