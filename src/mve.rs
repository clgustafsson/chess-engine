use std::fmt::format;

use crate::{
    board::{square_to_bitboard, Position, EMPTY},
    piece::{KING, NONE, PAWN},
};

use std::cmp::{max, min};

pub type Move = u16;

//https://www.chessprogramming.org/Encoding_Moves

//4 bits are used as a flag

pub const MOVE_FLAG: Move = 0b1111 << 12;

//bit	flag
//1		promotion
//2		capture
//3 	special 1
//4		special 0

// code
// 0	0	0	0	0	quiet moves
// 1	0	0	0	1	double pawn push
// 2	0	0	1	0	king castle
// 3	0	0	1	1	queen castle
// 4	0	1	0	0	captures
// 5	0	1	0	1	ep-capture
// 8	1	0	0	0	knight-promotion
// 9	1	0	0	1	bishop-promotion
// 10	1	0	1	0	rook-promotion
// 11	1	0	1	1	queen-promotion
// 12	1	1	0	0	knight-promo capture
// 13	1	1	0	1	bishop-promo capture
// 14	1	1	1	0	rook-promo capture
// 15	1	1	1	1	queen-promo capture

pub const QUIET_MOVE: Move = 0 << 12;
pub const DOUBLE_PAWN_PUSH: Move = 1 << 12;
pub const KING_CASTLE: Move = 2 << 12;
pub const QUEEN_CASTLE: Move = 3 << 12;
pub const CAPTURE: Move = 4 << 12;
pub const EN_PASSANT_CAPTURE: Move = 5 << 12;
pub const KNIGHT_PROMOTION: Move = 8 << 12;
pub const BISHOP_PROMOTION: Move = 9 << 12;
pub const ROOK_PROMOTION: Move = 10 << 12;
pub const QUEEN_PROMOTION: Move = 11 << 12;
pub const KNIGHT_PROMOTION_CAPTURE: Move = 12 << 12;
pub const BISHOP_PROMOTION_CAPTURE: Move = 13 << 12;
pub const ROOK_PROMOTION_CAPTURE: Move = 14 << 12;
pub const QUEEN_PROMOTION_CAPTURE: Move = 15 << 12;
pub const PROMOTION: Move = 8 << 12;

//12 bits are used to store from and to squares

pub const MOVE_FROM: Move = 0b111111;
pub const MOVE_TO: Move = 0b111111 << 6;

pub const NULL_MOVE: Move = 0;

pub fn move_to_algebraic(mve: Move) -> String {
    let from = mve & MOVE_FROM;
    let from_row = ((from / 8) as u8 + b'1') as char;
    let from_col = (7 - (from % 8) as u8 + b'a') as char;
    let to = (mve & MOVE_TO) >> 6;
    let to_row = ((to / 8) as u8 + b'1') as char;
    let to_col = (7 - (to % 8) as u8 + b'a') as char;
    let mut promotion = "";
    //note that order for promotion matters here
    if mve & QUEEN_PROMOTION == QUEEN_PROMOTION {
        promotion = "q";
    } else if mve & ROOK_PROMOTION == ROOK_PROMOTION {
        promotion = "r";
    } else if mve & BISHOP_PROMOTION == BISHOP_PROMOTION {
        promotion = "b";
    } else if mve & KNIGHT_PROMOTION == KNIGHT_PROMOTION {
        promotion = "n";
    }
    format!("{}{}{}{}{}", from_col, from_row, to_col, to_row, promotion)
}

//as this engine makes use of move flags, the current position is
//requiered to convert algebraic moves to create internal flags
pub fn algebraic_to_move(pos: &Position, algebraic: &str) -> Move {
    let from = &algebraic[..2];
    let from_col = from.chars().next().unwrap() as u8 - b'a';
    let from_row = from.chars().nth(1).unwrap() as u8 - b'1';
    let to = &algebraic[2..];
    let to_col = to.chars().next().unwrap() as u8 - b'a';
    let to_row = to.chars().nth(1).unwrap() as u8 - b'1';

    let from_square = (from_row) * 8 + (7 - from_col);
    let to_square = (to_row) * 8 + (7 - to_col);

    let mut mve: Move = from_square as u16 | ((to_square as u16) << 6);

    if pos.pieces[to_square as usize] != NONE {
        mve |= CAPTURE;
    } else if pos.en_passant_target_square == square_to_bitboard(to_square) {
        mve |= EN_PASSANT_CAPTURE;
    }
    if pos.pieces[from_square as usize] == PAWN as u8 {
        if max(from_square, to_square) - min(from_square, to_square) == 16 {
            mve |= DOUBLE_PAWN_PUSH;
        }
    } else if pos.pieces[from_square as usize] == KING as u8
        && max(from_square, to_square) - min(from_square, to_square) == 2
    {
        if to_square == 1 || to_square == 57 {
            mve |= KING_CASTLE;
        } else if to_square == 5 || to_square == 61 {
            mve |= QUEEN_CASTLE;
        }
    }
    if to.len() == 3 {
        match to.chars().nth(2).unwrap() {
            'q' => mve |= QUEEN_PROMOTION,
            'n' => mve |= KNIGHT_PROMOTION,
            'r' => mve |= ROOK_PROMOTION,
            'b' => mve |= BISHOP_PROMOTION,
            _ => panic!(),
        }
    }
    mve
}
