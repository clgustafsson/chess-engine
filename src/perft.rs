use std::time::Instant;

use crate::{
    board::Position,
    mve::{move_to_algebraic, NULL_MOVE},
};

//counts the amount of leaf nodes at a certain depth
pub fn perft(pos: &mut Position, depth: u8) -> usize {
    let legal_moves = pos.legal_moves();
    if depth == 1 {
        return legal_moves.1;
    }
    let mut count = 0;

    for i in 0..legal_moves.1 {
        let mut pos_clone = pos.clone();
        pos_clone.make_move(legal_moves.0[i]);
        count += perft(&mut pos_clone, depth - 1);
    }
    count
}

//performs a perft seach with printed results and meausers the speed of the move generation
//the results from this can be used to compare correctness and speed with stockfish
pub fn bench(pos: &mut Position, depth: u8) {
    let start = Instant::now();
    let mut count = 0;
    let legal_moves = pos.legal_moves();
    if depth == 1 {
        count += legal_moves.1;
        for i in 0..legal_moves.1 {
            println!("{} ", move_to_algebraic(legal_moves.0[i]));
        }
    } else {
        for i in 0..legal_moves.1 {
            let mve = legal_moves.0[i];
            let mut pos_clone = pos.clone();
            pos_clone.make_move(mve);
            let res = perft(&mut pos_clone, depth - 1);
            count += res;
            println!("{}: {}", move_to_algebraic(mve), res)
        }
    }
    let end = Instant::now();
    let duration = end.duration_since(start);

    let nodes_per_second = count as u128 * 1_000_000 / duration.as_micros();

    println!();
    println!("Time Elapsed: {} ms", duration.as_millis());
    println!("Total Nodes: {}", count);
    println!("Nodes/Second: {}", nodes_per_second);
}

#[cfg(test)]
mod tests {
    use crate::init::initialize_engine;

    use super::*;
    //https://www.chessprogramming.org/Perft_Results

    //initialize_engine() is is not thread safe atm
    //so running all tests at once on multiple theads will cause false negatives
    //run the tests one by one or on one thread
    //cargo test --release -- --test-threads=1

    #[test]
    fn perft_startpos() {
        initialize_engine();
        let mut board = Position::new();
        board.parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        assert_eq!(20, perft(&mut board, 1));
        assert_eq!(400, perft(&mut board, 2));
        assert_eq!(8_902, perft(&mut board, 3));
        assert_eq!(197_281, perft(&mut board, 4));
        assert_eq!(4_865_609, perft(&mut board, 5));
        assert_eq!(119_060_324, perft(&mut board, 6));
    }

    #[test]
    fn perft_kiwipete() {
        initialize_engine();
        let mut board = Position::new();
        board.parse_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

        assert_eq!(48, perft(&mut board, 1));
        assert_eq!(2_039, perft(&mut board, 2));
        assert_eq!(97_862, perft(&mut board, 3));
        assert_eq!(4_085_603, perft(&mut board, 4));
        assert_eq!(193_690_690, perft(&mut board, 5));
    }

    #[test]
    fn perft_position3() {
        initialize_engine();
        let mut board = Position::new();
        board.parse_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");

        assert_eq!(14, perft(&mut board, 1));
        assert_eq!(191, perft(&mut board, 2));
        assert_eq!(2_812, perft(&mut board, 3));
        assert_eq!(43_238, perft(&mut board, 4));
        assert_eq!(674_624, perft(&mut board, 5));
        assert_eq!(11_030_083, perft(&mut board, 6));
        assert_eq!(178_633_661, perft(&mut board, 7));
    }

    #[test]
    fn perft_position4_white() {
        initialize_engine();
        let mut board = Position::new();
        board.parse_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");

        assert_eq!(6, perft(&mut board, 1));
        assert_eq!(264, perft(&mut board, 2));
        assert_eq!(9_467, perft(&mut board, 3));
        assert_eq!(422_333, perft(&mut board, 4));
        assert_eq!(15_833_292, perft(&mut board, 5));
    }

    #[test]
    fn perft_position4_black() {
        initialize_engine();
        let mut board = Position::new();
        board.parse_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1");

        assert_eq!(6, perft(&mut board, 1));
        assert_eq!(264, perft(&mut board, 2));
        assert_eq!(9_467, perft(&mut board, 3));
        assert_eq!(422_333, perft(&mut board, 4));
        assert_eq!(15_833_292, perft(&mut board, 5));
    }

    #[test]
    fn perft_position5() {
        initialize_engine();
        let mut board = Position::new();
        board.parse_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");

        assert_eq!(44, perft(&mut board, 1));
        assert_eq!(1_486, perft(&mut board, 2));
        assert_eq!(62_379, perft(&mut board, 3));
        assert_eq!(2_103_487, perft(&mut board, 4));
        assert_eq!(89_941_194, perft(&mut board, 5));
    }

    #[test]
    fn perft_position6() {
        initialize_engine();
        let mut board = Position::new();
        board.parse_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");

        assert_eq!(46, perft(&mut board, 1));
        assert_eq!(2_079, perft(&mut board, 2));
        assert_eq!(89_890, perft(&mut board, 3));
        assert_eq!(3_894_594, perft(&mut board, 4));
        assert_eq!(164_075_551, perft(&mut board, 5));
    }
}
