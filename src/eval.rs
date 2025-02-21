use crate::{
    board::{bitboard_to_square, Position, Square, EMPTY, WHITE},
    piece::{BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK},
};

//Values and heatmaps from
//https://www.chessprogramming.org/Simplified_Evaluation_Function

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 0;
const EMPTY_VALUE: i32 = 0;

pub const PIECE_VALUES: [i32; 7] = [
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    PAWN_VALUE,
    KING_VALUE,
    EMPTY_VALUE,
];

pub static mut COUNT: i32 = 0;

//returns the evaluation of the position from the current players perspective
pub fn evaluate(pos: &Position) -> i32 {
    let mut eval = 0;

    let mut w_pieces = pos.w_board;
    while w_pieces != 0 {
        let square = (w_pieces & !(w_pieces - 1)).trailing_zeros() as usize;
        eval += PIECE_VALUES[pos.pieces[square] as usize];
        eval += PIECE_HEATMAP[pos.pieces[square] as usize][63 - square];
        w_pieces &= w_pieces - 1; //removing the last bit
    }
    let mut b_pieces = pos.b_board;
    while b_pieces != 0 {
        let square = (b_pieces & !(b_pieces - 1)).trailing_zeros() as usize;
        eval -= PIECE_VALUES[pos.pieces[square] as usize];
        eval -= PIECE_HEATMAP[pos.pieces[square] as usize][square];
        b_pieces &= b_pieces - 1; //removing the last bit
    }
    if pos.blocker_board.count_ones() < 18 {
        //count position as endgame
        let w_king_pos = pos.w_piece_board[KING].trailing_zeros() as usize;
        let b_king_pos = pos.b_piece_board[KING].trailing_zeros() as usize;
        //reverting king heatmap eval
        eval -= PIECE_HEATMAP[KING][63 - w_king_pos];
        eval += PIECE_HEATMAP[KING][b_king_pos];
        //applying endgame king heatmap instead
        eval += KING_END_GAME_HEATMAP[63 - w_king_pos];
        eval -= KING_END_GAME_HEATMAP[b_king_pos];
    }
    //converting color_to_move to 1 if WHITE and -1 if BLACK without branching to change perspective for negamax
    eval * (2 * (pos.color_to_move as i8) - 1) as i32
}

//heatmaps for all pieces as pieces placed on better squares are worth more
const PIECE_HEATMAP: [[i32; 64]; 6] = [
    KNIGHT_HEATMAP,
    BISHOP_HEATMAP,
    ROOK_HEATMAP,
    QUEEN_HEATMAP,
    PAWN_HEATMAP,
    KING_HEATMAP,
];

#[rustfmt::skip]
const PAWN_HEATMAP: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10, -20, -20, 10, 10, 5,
     0,  0,  0,   0,   0,  0,  0, 0,
];

#[rustfmt::skip]
const KNIGHT_HEATMAP: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_HEATMAP: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_HEATMAP: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN_HEATMAP: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
const KING_HEATMAP: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

#[rustfmt::skip]
const KING_END_GAME_HEATMAP: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50,
];
