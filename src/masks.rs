use crate::board::{
    Bitboard, Square, FILE, NOT_ON_AB_FILE, NOT_ON_A_FILE, NOT_ON_GH_FILE, NOT_ON_H_FILE, RANK,
};

pub static mut KING_MASK: [Bitboard; 64] = [0; 64];
pub static mut KNIGHT_MASK: [Bitboard; 64] = [0; 64];
pub static mut ROOK_ALL_BLOCKERS_MASK: [Bitboard; 64] = [0; 64];
pub static mut BISHOP_ALL_BLOCKERS_MASK: [Bitboard; 64] = [0; 64];

pub fn init_masks() {
    for square in 0..64 {
        let bit_square = 0b1u64 << square;
        unsafe {
            KING_MASK[square] = king_mask(bit_square);
            KNIGHT_MASK[square] = knight_mask(bit_square);
            ROOK_ALL_BLOCKERS_MASK[square] = rook_all_blockers_mask(square as Square);
            BISHOP_ALL_BLOCKERS_MASK[square] = bishop_all_blockers_mask(square as Square)
        }
    }
}

#[inline]
pub fn w_pawn_capture_mask(bitboard_square: &Bitboard) -> Bitboard {
    ((*bitboard_square & NOT_ON_H_FILE) << 7) | ((*bitboard_square & NOT_ON_A_FILE) << 9)
}

#[inline]
pub fn b_pawn_capture_mask(bitboard_square: &Bitboard) -> Bitboard {
    ((*bitboard_square & NOT_ON_H_FILE) >> 9) | ((*bitboard_square & NOT_ON_A_FILE) >> 7)
}

fn king_mask(bitboard_square: Bitboard) -> Bitboard {
    ((bitboard_square & NOT_ON_H_FILE) << 7)
        | ((bitboard_square & NOT_ON_H_FILE) >> 1)
        | ((bitboard_square & NOT_ON_H_FILE) >> 9)
        | (bitboard_square << 8)
        | (bitboard_square >> 8)
        | ((bitboard_square & NOT_ON_A_FILE) << 9)
        | ((bitboard_square & NOT_ON_A_FILE) << 1)
        | ((bitboard_square & NOT_ON_A_FILE) >> 7)
}

fn knight_mask(bitboard_square: Bitboard) -> Bitboard {
    ((bitboard_square & NOT_ON_A_FILE) << 17)
        | ((bitboard_square & NOT_ON_A_FILE) >> 15)
        | ((bitboard_square & NOT_ON_H_FILE) << 15)
        | ((bitboard_square & NOT_ON_H_FILE) >> 17)
        | ((bitboard_square & NOT_ON_AB_FILE) << 10)
        | ((bitboard_square & NOT_ON_AB_FILE) >> 6)
        | ((bitboard_square & NOT_ON_GH_FILE) << 6)
        | ((bitboard_square & NOT_ON_GH_FILE) >> 10)
}

//returns a bitboard with all squares of potential blockers for rooks
//pieces on edges can never block
pub fn rook_all_blockers_mask(square: Square) -> Bitboard {
    let file = 7 - (square % 8);
    let rank = square / 8;
    let not_on_ah = !(FILE[0] | FILE[7]);
    let not_on_18 = !(RANK[0] | RANK[7]);
    ((FILE[file as usize] & not_on_18) ^ (RANK[rank as usize] & not_on_ah)) & !(0b1u64 << square)
}

pub fn rook_mask(bitboard_square: Bitboard, blocker_board: Bitboard) -> Bitboard {
    let mut mask = 0b0u64;
    let mut ptr = bitboard_square;
    //up
    while ptr & RANK[7] == 0 {
        ptr <<= 8;
        mask |= ptr;
        if ptr & blocker_board != 0 {
            break;
        }
    }
    ptr = bitboard_square;
    //down
    while ptr & RANK[0] == 0 {
        ptr >>= 8;
        mask |= ptr;
        if ptr & blocker_board != 0 {
            break;
        }
    }
    ptr = bitboard_square;
    //left
    while ptr & FILE[0] == 0 {
        ptr <<= 1;
        mask |= ptr;
        if ptr & blocker_board != 0 {
            break;
        }
    }
    ptr = bitboard_square;
    //right
    while ptr & FILE[7] == 0 {
        ptr >>= 1;
        mask |= ptr;
        if ptr & blocker_board != 0 {
            break;
        }
    }
    mask
}

//returns a bitboard with all squares of potential blockers for bishops
//pieces on edges can never block
pub fn bishop_all_blockers_mask(square: Square) -> Bitboard {
    let bitboard_edges = FILE[0] | FILE[7] | RANK[0] | RANK[7];
    bishop_mask(0b1u64 << square, 0b0u64) & !bitboard_edges
}

pub fn bishop_mask(bitboard_square: Bitboard, blocker_board: Bitboard) -> Bitboard {
    let mut mask = 0b0u64;
    let mut ptr = bitboard_square;
    //up right
    while ptr & (RANK[7] | FILE[7]) == 0 {
        ptr <<= 7;
        mask |= ptr;
        if ptr & blocker_board != 0 {
            break;
        }
    }
    ptr = bitboard_square;
    //down right
    while ptr & (RANK[0] | FILE[7]) == 0 {
        ptr >>= 9;
        mask |= ptr;
        if ptr & blocker_board != 0 {
            break;
        }
    }
    ptr = bitboard_square;
    //up left
    while ptr & (RANK[7] | FILE[0]) == 0 {
        ptr <<= 9;
        mask |= ptr;
        if ptr & blocker_board != 0 {
            break;
        }
    }
    ptr = bitboard_square;
    //down left
    while ptr & (RANK[0] | FILE[0]) == 0 {
        ptr >>= 7;
        mask |= ptr;
        if ptr & blocker_board != 0 {
            break;
        }
    }
    mask
}
