use crate::{
    board::{Bitboard, Color, Position, Square, NOT_ON_A_FILE, NOT_ON_H_FILE, RANK, WHITE},
    magic::{BISHOP_MAGIC_SHIFT, MAGIC, ROOK_MAGIC_SHIFT},
    masks::{BISHOP_ALL_BLOCKERS_MASK, KING_MASK, KNIGHT_MASK, ROOK_ALL_BLOCKERS_MASK},
    mve::Move,
};

pub const KNIGHT: usize = 0;
pub const BISHOP: usize = 1;
pub const ROOK: usize = 2;
pub const QUEEN: usize = 3;
pub const PAWN: usize = 4;
pub const KING: usize = 5;
pub const NONE: u8 = 6;

impl Position {
    //Pawn moves are handled by directly shifting bits (no lookup)
    #[inline]
    pub fn w_pawn_forward_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        (*bitboard_square << 8) & !self.blocker_board & self.checked_squares
    }
    #[inline]
    pub fn w_pawn_doubleforward_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        ((*bitboard_square & (RANK[1])) << 16)
            & !(self.blocker_board | (self.blocker_board << 8))
            & self.checked_squares
    }
    #[inline]
    pub fn w_pawn_capture_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        (((*bitboard_square & NOT_ON_H_FILE) << 7) | ((*bitboard_square & NOT_ON_A_FILE) << 9))
            & self.b_board
            & self.checked_squares
    }
    #[inline]
    pub fn w_pawn_en_passant_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        //subtle but really important detail is that the check mask must be shifted on this
        //particular edge case to be able to remove checking pawns with en passant
        ((((*bitboard_square & NOT_ON_H_FILE) << 7) | ((*bitboard_square & NOT_ON_A_FILE) << 9))
            & self.en_passant_target_square)
            & (self.checked_squares << 8)
    }
    #[inline]
    pub fn b_pawn_forward_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        (*bitboard_square >> 8) & !self.blocker_board & self.checked_squares
    }
    #[inline]
    pub fn b_pawn_doubleforward_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        ((*bitboard_square & (RANK[6])) >> 16)
            & !(self.blocker_board | (self.blocker_board >> 8))
            & self.checked_squares
    }
    #[inline]
    pub fn b_pawn_capture_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        (((*bitboard_square & NOT_ON_H_FILE) >> 9) | ((*bitboard_square & NOT_ON_A_FILE) >> 7))
            & self.w_board
            & self.checked_squares
    }
    #[inline]
    pub fn b_pawn_en_passant_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        //subtle but really important detail is that the check mask must be shifted on this
        //particular edge case to be able to remove checking pawns with en passant
        ((((*bitboard_square & NOT_ON_H_FILE) >> 9) | ((*bitboard_square & NOT_ON_A_FILE) >> 7))
            & self.en_passant_target_square)
            & (self.checked_squares >> 8)
    }
    #[inline]
    pub fn w_pinned_pawn_forward_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        self.w_pawn_forward_mask(bitboard_square) & self.orthogonal_pin
    }
    #[inline]
    pub fn w_pinned_pawn_doubleforward_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        self.w_pawn_doubleforward_mask(bitboard_square) & self.orthogonal_pin
    }
    #[inline]
    pub fn w_pinned_pawn_capture_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        self.w_pawn_capture_mask(bitboard_square) & self.diagonal_pin
    }
    #[inline]
    pub fn w_pinned_pawn_en_passant_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        self.w_pawn_en_passant_mask(bitboard_square) & self.diagonal_pin
    }
    #[inline]
    pub fn b_pinned_pawn_forward_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        self.b_pawn_forward_mask(bitboard_square) & self.orthogonal_pin
    }
    #[inline]
    pub fn b_pinned_pawn_doubleforward_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        self.b_pawn_doubleforward_mask(bitboard_square) & self.orthogonal_pin
    }
    #[inline]
    pub fn b_pinned_pawn_capture_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        self.b_pawn_capture_mask(bitboard_square) & self.diagonal_pin
    }
    #[inline]
    pub fn b_pinned_pawn_en_passant_mask(&self, bitboard_square: &Bitboard) -> Bitboard {
        self.b_pawn_en_passant_mask(bitboard_square) & self.diagonal_pin
    }

    //when calculating all seen squares by pawns it is possible to calculate all attacks at once
    //by shifting the full pawn bitboard
    #[inline]
    pub fn seen_by_w_pawns(&self) -> Bitboard {
        ((self.w_piece_board[PAWN] & NOT_ON_H_FILE) << 7)
            | ((self.w_piece_board[PAWN] & NOT_ON_A_FILE) << 9)
    }
    #[inline]
    pub fn seen_by_b_pawns(&self) -> Bitboard {
        ((self.b_piece_board[PAWN] & NOT_ON_H_FILE) >> 9)
            | ((self.b_piece_board[PAWN] & NOT_ON_A_FILE) >> 7)
    }

    //All other pieces use lookups

    //Sliding pieces
    pub fn seen_by_rook(&self, square: Square) -> Bitboard {
        unsafe {
            let rook_blocker_board = self.blocker_board & ROOK_ALL_BLOCKERS_MASK[square as usize];
            let magic_number = MAGIC.rook_magic[square as usize];
            let lookup = &MAGIC.rook_lookup[square as usize];
            let magic_index = rook_blocker_board.wrapping_mul(magic_number) >> ROOK_MAGIC_SHIFT;
            lookup[magic_index as usize]
        }
    }

    pub fn seen_by_rook_custom_blocker(&self, square: Square, blocker: Bitboard) -> Bitboard {
        unsafe {
            let rook_blocker_board = blocker & ROOK_ALL_BLOCKERS_MASK[square as usize];
            let magic_number = MAGIC.rook_magic[square as usize];
            let lookup = &MAGIC.rook_lookup[square as usize];
            let magic_index = rook_blocker_board.wrapping_mul(magic_number) >> ROOK_MAGIC_SHIFT;
            lookup[magic_index as usize]
        }
    }

    #[inline]
    pub fn w_rook_moves(&self, square: Square) -> Bitboard {
        self.seen_by_rook(square) & !self.w_board & self.checked_squares
    }

    #[inline]
    pub fn b_rook_moves(&self, square: Square) -> Bitboard {
        self.seen_by_rook(square) & !self.b_board & self.checked_squares
    }

    pub fn seen_by_bishop(&self, square: Square) -> Bitboard {
        unsafe {
            let bishop_blocker_board =
                self.blocker_board & BISHOP_ALL_BLOCKERS_MASK[square as usize];
            let magic_number = MAGIC.bishop_magic[square as usize];
            let lookup = &MAGIC.bishop_lookup[square as usize];
            let magic_index = bishop_blocker_board.wrapping_mul(magic_number) >> BISHOP_MAGIC_SHIFT;
            lookup[magic_index as usize]
        }
    }
    pub fn seen_by_bishop_custom_blocker(&self, square: Square, blocker: Bitboard) -> Bitboard {
        unsafe {
            let bishop_blocker_board = blocker & BISHOP_ALL_BLOCKERS_MASK[square as usize];
            let magic_number = MAGIC.bishop_magic[square as usize];
            let lookup = &MAGIC.bishop_lookup[square as usize];
            let magic_index = bishop_blocker_board.wrapping_mul(magic_number) >> BISHOP_MAGIC_SHIFT;
            lookup[magic_index as usize]
        }
    }

    #[inline]
    pub fn w_bishop_moves(&self, square: Square) -> Bitboard {
        self.seen_by_bishop(square) & !self.w_board & self.checked_squares
    }

    #[inline]
    pub fn b_bishop_moves(&self, square: Square) -> Bitboard {
        self.seen_by_bishop(square) & !self.b_board & self.checked_squares
    }

    //Sliding pieces pinned in the same direction as they move
    #[inline]
    pub fn w_pinned_rook_moves(&self, square: Square) -> Bitboard {
        self.w_rook_moves(square) & self.orthogonal_pin
    }
    #[inline]
    pub fn w_pinned_bishop_moves(&self, square: Square) -> Bitboard {
        self.w_bishop_moves(square) & self.diagonal_pin
    }
    #[inline]
    pub fn b_pinned_rook_moves(&self, square: Square) -> Bitboard {
        self.b_rook_moves(square) & self.orthogonal_pin
    }
    #[inline]
    pub fn b_pinned_bishop_moves(&self, square: Square) -> Bitboard {
        self.b_bishop_moves(square) & self.diagonal_pin
    }

    //Non sliding pieces
    #[inline]
    pub fn w_king_move(&self, square: Square) -> Bitboard {
        unsafe { KING_MASK[square as usize] & !self.w_board }
    }
    #[inline]
    pub fn w_knight_move(&self, square: Square) -> Bitboard {
        unsafe { KNIGHT_MASK[square as usize] & !self.w_board & self.checked_squares }
    }
    #[inline]
    pub fn b_king_move(&self, square: Square) -> Bitboard {
        unsafe { KING_MASK[square as usize] & !self.b_board }
    }
    #[inline]
    pub fn b_knight_move(&self, square: Square) -> Bitboard {
        unsafe { KNIGHT_MASK[square as usize] & !self.b_board & self.checked_squares }
    }
}
