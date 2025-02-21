use std::{fmt::format, u8};

use crate::{
    mve::{algebraic_to_move, Move},
    piece::{BISHOP, KING, KNIGHT, NONE, PAWN, QUEEN, ROOK},
};

pub type Bitboard = u64;
pub type Square = u8;
pub type Color = bool;

#[derive(Clone, PartialEq)]
pub enum Result {
    Checkmate,
    Draw,
    None,
}

pub const WHITE: Color = true;
pub const BLACK: Color = false;

pub const EMPTY: Bitboard = 0;

#[derive(Clone)]
pub struct Position {
    pub w_piece_board: [Bitboard; 6],
    pub b_piece_board: [Bitboard; 6],
    pub w_board: Bitboard,
    pub b_board: Bitboard,
    pub pieces: [u8; 64],
    pub blocker_board: Bitboard,
    pub checked_squares: Bitboard,
    pub orthogonal_pin: Bitboard,
    pub diagonal_pin: Bitboard,
    pub en_passant_target_square: Bitboard,
    pub castling_rights: [bool; 4],
    pub color_to_move: Color,
    pub half_move_clock: u32,
    pub full_moves: u32,
    pub result: Result,
}

#[inline]
pub fn square_to_bitboard(square: Square) -> Bitboard {
    0b1u64 << square
}

#[inline]
pub fn bitboard_to_square(bit_square: Bitboard) -> Square {
    bit_square.trailing_zeros() as Square
}

pub const FILE: [Bitboard; 8] = [
    0b1000000010000000100000001000000010000000100000001000000010000000u64,
    0b0100000001000000010000000100000001000000010000000100000001000000u64,
    0b0010000000100000001000000010000000100000001000000010000000100000u64,
    0b0001000000010000000100000001000000010000000100000001000000010000u64,
    0b0000100000001000000010000000100000001000000010000000100000001000u64,
    0b0000010000000100000001000000010000000100000001000000010000000100u64,
    0b0000001000000010000000100000001000000010000000100000001000000010u64,
    0b0000000100000001000000010000000100000001000000010000000100000001u64,
];

pub const RANK: [Bitboard; 8] = [
    0b11111111u64,
    0b11111111u64 << 8,
    0b11111111u64 << 16,
    0b11111111u64 << 24,
    0b11111111u64 << 32,
    0b11111111u64 << 40,
    0b11111111u64 << 48,
    0b11111111u64 << 56,
];

pub const NOT_ON_H_FILE: Bitboard =
    0b1111111011111110111111101111111011111110111111101111111011111110u64;
pub const NOT_ON_A_FILE: Bitboard =
    0b0111111101111111011111110111111101111111011111110111111101111111u64;

pub const NOT_ON_GH_FILE: Bitboard =
    0b1111110011111100111111001111110011111100111111001111110011111100u64;
pub const NOT_ON_AB_FILE: Bitboard =
    0b0011111100111111001111110011111100111111001111110011111100111111u64;

impl Position {
    pub fn new() -> Position {
        Position {
            w_piece_board: [EMPTY; 6],
            b_piece_board: [EMPTY; 6],
            w_board: EMPTY,
            b_board: EMPTY,
            pieces: [NONE; 64],
            blocker_board: EMPTY,
            checked_squares: EMPTY,
            orthogonal_pin: EMPTY,
            diagonal_pin: EMPTY,
            en_passant_target_square: EMPTY,
            castling_rights: [false; 4],
            color_to_move: WHITE,
            half_move_clock: 0,
            full_moves: 0,
            result: Result::None,
        }
    }

    pub fn startpos() -> Position {
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn from_move_string(move_string: &str) -> Position {
        let mut position = Position::startpos();
        let mut moves = move_string.split_whitespace();

        for algebraic_mve in moves {
            let mve = algebraic_to_move(&position, algebraic_mve);
            position.make_move(mve);
        }

        position
    }

    pub fn from_fen_move_string(fen: &str, move_string: &str) -> Position {
        let mut position = Position::from_fen(fen);
        let mut moves = move_string.split_whitespace();

        for algebraic_mve in moves {
            let mve = algebraic_to_move(&position, algebraic_mve);
            position.make_move(mve);
        }
        position
    }

    pub fn from_fen(fen: &str) -> Position {
        let mut position = Position::new();
        position.parse_fen(fen);
        position
    }

    //this function is only safe for correct FEN strings
    pub fn parse_fen(&mut self, fen: &str) {
        let mut ptr = 1 << 63;
        let mut fen_iter = fen.as_bytes().iter();

        //parsing the board
        loop {
            let byte = fen_iter.next().unwrap();

            if byte == &b' ' {
                break;
            } else if (&b'1'..=&b'9').contains(&byte) {
                ptr >>= byte - b'0';
                continue;
            }
            match byte {
                b'r' => {
                    self.b_piece_board[ROOK] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = ROOK as u8;
                }
                b'b' => {
                    self.b_piece_board[BISHOP] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = BISHOP as u8;
                }
                b'n' => {
                    self.b_piece_board[KNIGHT] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = KNIGHT as u8;
                }
                b'q' => {
                    self.b_piece_board[QUEEN] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = QUEEN as u8;
                }
                b'k' => {
                    self.b_piece_board[KING] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = KING as u8;
                }
                b'p' => {
                    self.b_piece_board[PAWN] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = PAWN as u8;
                }
                b'R' => {
                    self.w_piece_board[ROOK] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = ROOK as u8;
                }
                b'B' => {
                    self.w_piece_board[BISHOP] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = BISHOP as u8;
                }
                b'N' => {
                    self.w_piece_board[KNIGHT] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = KNIGHT as u8;
                }
                b'Q' => {
                    self.w_piece_board[QUEEN] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = QUEEN as u8;
                }
                b'K' => {
                    self.w_piece_board[KING] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = KING as u8;
                }
                b'P' => {
                    self.w_piece_board[PAWN] |= ptr;
                    self.pieces[bitboard_to_square(ptr) as usize] = PAWN as u8;
                }
                _ => {
                    continue;
                }
            }
            ptr >>= 1;
        }
        self.w_board = self.w_piece_board[ROOK]
            | self.w_piece_board[BISHOP]
            | self.w_piece_board[KNIGHT]
            | self.w_piece_board[QUEEN]
            | self.w_piece_board[KING]
            | self.w_piece_board[PAWN];
        self.b_board = self.b_piece_board[ROOK]
            | self.b_piece_board[BISHOP]
            | self.b_piece_board[KNIGHT]
            | self.b_piece_board[QUEEN]
            | self.b_piece_board[KING]
            | self.b_piece_board[PAWN];
        self.blocker_board = self.w_board | self.b_board;

        //parse color to move

        self.color_to_move = match fen_iter.next().unwrap() {
            b'w' => WHITE,
            _ => BLACK,
        };
        fen_iter.next();

        //parsing castling rights
        loop {
            let byte = fen_iter.next().unwrap();

            if byte == &b'-' {
                continue;
            }
            match byte {
                b'K' => self.castling_rights[0] = true,
                b'Q' => self.castling_rights[1] = true,
                b'k' => self.castling_rights[2] = true,
                b'q' => self.castling_rights[3] = true,
                _ => {
                    break;
                }
            }
        }
        //parse en passant target square
        let byte = fen_iter.next().unwrap();

        if byte == &b'-' {
            fen_iter.next();
        } else {
            let col = byte - b'a';
            let row = fen_iter.next().unwrap() - b'1';

            let square = 8 * row + 7 - col;
            self.en_passant_target_square = 0b1u64 << square;
            fen_iter.next();
        }
        //parse halfmove and fullmove
        loop {
            let byte = fen_iter.next().unwrap();
            if byte != &b' ' {
                let n = byte - b'0';
                self.half_move_clock *= 10;
                self.half_move_clock += n as u32;
            } else {
                break;
            }
        }

        loop {
            let byte_res = fen_iter.next();

            if let Some(byte) = byte_res {
                let n = byte - b'0';
                self.full_moves *= 10;
                self.full_moves += n as u32;
            } else {
                break;
            }
        }
    }

    pub fn fen(&self) -> String {
        let mut fen = String::new();
        let mut empty_count = 0;
        //board
        for mut square in 0..64 {
            square = 63 - square;

            match self.pieces[square] as usize {
                PAWN => {
                    if empty_count != 0 {
                        fen += &format!("{}", empty_count);
                        empty_count = 0;
                    }
                    let bit_square = square_to_bitboard(square as u8);
                    if bit_square & self.w_board != 0 {
                        fen += "P";
                    } else {
                        fen += "p";
                    }
                }
                KNIGHT => {
                    if empty_count != 0 {
                        fen += &format!("{}", empty_count);
                        empty_count = 0;
                    }
                    let bit_square = square_to_bitboard(square as u8);
                    if bit_square & self.w_board != 0 {
                        fen += "N";
                    } else {
                        fen += "n";
                    }
                }
                BISHOP => {
                    if empty_count != 0 {
                        fen += &format!("{}", empty_count);
                        empty_count = 0;
                    }
                    let bit_square = square_to_bitboard(square as u8);
                    if bit_square & self.w_board != 0 {
                        fen += "B";
                    } else {
                        fen += "b";
                    }
                }
                ROOK => {
                    if empty_count != 0 {
                        fen += &format!("{}", empty_count);
                        empty_count = 0;
                    }
                    let bit_square = square_to_bitboard(square as u8);
                    if bit_square & self.w_board != 0 {
                        fen += "R";
                    } else {
                        fen += "r";
                    }
                }
                KING => {
                    if empty_count != 0 {
                        fen += &format!("{}", empty_count);
                        empty_count = 0;
                    }
                    let bit_square = square_to_bitboard(square as u8);
                    if bit_square & self.w_board != 0 {
                        fen += "K";
                    } else {
                        fen += "k";
                    }
                }
                QUEEN => {
                    if empty_count != 0 {
                        fen += &format!("{}", empty_count);
                        empty_count = 0;
                    }
                    let bit_square = square_to_bitboard(square as u8);
                    if bit_square & self.w_board != 0 {
                        fen += "Q";
                    } else {
                        fen += "q";
                    }
                }
                _ => {
                    empty_count += 1;
                }
            }
            if square % 8 == 0 && square != 0 {
                if empty_count != 0 {
                    fen += &format!("{}", empty_count);
                    empty_count = 0;
                }
                fen += "/";
            }
        }
        fen += " ";
        //turn
        match self.color_to_move {
            WHITE => fen += "w",
            BLACK => fen += "b",
        }
        fen += " ";
        //castling rights
        if self.castling_rights[0] {
            fen += "K";
        }
        if self.castling_rights[1] {
            fen += "Q";
        }
        if self.castling_rights[2] {
            fen += "k";
        }
        if self.castling_rights[3] {
            fen += "q";
        }
        fen += " ";
        //en passant target square
        if self.en_passant_target_square == EMPTY {
            fen += "- "
        } else {
            let square = bitboard_to_square(self.en_passant_target_square);
            let row = ((square / 8) as u8 + b'1') as char;
            let col = (7 - (square % 8) as u8 + b'a') as char;
            fen += &format!("{}{} ", col, row);
        }

        //Halfmove clock
        fen += &format!("{} ", self.half_move_clock);
        //Full move number
        fen += &format!("{}", self.full_moves);
        fen
    }

    pub fn print(&self) {
        let mut rank = 8;
        for mut square in 0..64u8 {
            if square % 8 == 0 {
                print!("\n{} ", rank);
                rank -= 1;
            }
            square = 63 - square;
            let ptr = 0b1u64 << square;
            let mut no_piece_count = 0;
            for piece in 0..6 {
                if ptr & self.w_piece_board[piece] != 0 {
                    match piece {
                        QUEEN => {
                            print!("♕");
                        }
                        ROOK => {
                            print!("♖");
                        }
                        BISHOP => {
                            print!("♗");
                        }
                        KNIGHT => {
                            print!("♘");
                        }
                        PAWN => {
                            print!("♙");
                        }
                        KING => {
                            print!("♔");
                        }
                        _ => {}
                    };
                } else if ptr & self.b_piece_board[piece] != 0 {
                    match piece {
                        QUEEN => {
                            print!("♛");
                        }
                        ROOK => {
                            print!("♜");
                        }
                        BISHOP => {
                            print!("♝");
                        }
                        KNIGHT => {
                            print!("♞");
                        }
                        PAWN => {
                            print!("♟︎");
                        }
                        KING => {
                            print!("♚");
                        }
                        _ => {}
                    };
                } else {
                    no_piece_count += 1;
                }
            }
            if no_piece_count == 6 {
                print!("_")
            }
        }
        println!("\n  ABCDEFGH");
        if self.en_passant_target_square != EMPTY {
            println!(
                "En passant target square: {}",
                bitboard_to_square(self.en_passant_target_square)
            );
        } else {
            println!("En passant target square: None")
        }
        print!("Castling rights: ");
        if self.castling_rights[0] {
            print!("K")
        }
        if self.castling_rights[1] {
            print!("Q")
        }
        if self.castling_rights[2] {
            print!("k")
        }
        if self.castling_rights[3] {
            print!("q")
        }
        println!();
        println!(
            "halfmove: {}, fullmove {}",
            self.half_move_clock, self.full_moves
        );
        if self.color_to_move == WHITE {
            println!("White to move");
        } else {
            println!("Black to move");
        }
    }
}

pub fn print_bitboard(board: Bitboard) {
    for mut square in 0..64 {
        if square % 8 == 0 {
            println!();
        }
        square = 63 - square;

        let ptr = 0b1u64 << square;
        if board & ptr == ptr {
            print!("1")
        } else {
            print!(".")
        }
    }
    println!();
}
