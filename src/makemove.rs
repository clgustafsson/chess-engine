use crate::{
    board::{square_to_bitboard, Position, BLACK, WHITE},
    mve::{
        move_to_algebraic, Move, BISHOP_PROMOTION, BISHOP_PROMOTION_CAPTURE, CAPTURE,
        DOUBLE_PAWN_PUSH, EN_PASSANT_CAPTURE, KING_CASTLE, KNIGHT_PROMOTION,
        KNIGHT_PROMOTION_CAPTURE, MOVE_FLAG, MOVE_FROM, MOVE_TO, PROMOTION, QUEEN_CASTLE,
        QUEEN_PROMOTION, QUEEN_PROMOTION_CAPTURE, QUIET_MOVE, ROOK_PROMOTION,
        ROOK_PROMOTION_CAPTURE,
    },
    piece::{BISHOP, KING, KNIGHT, NONE, PAWN, QUEEN, ROOK},
};

impl Position {
    //plays a move on the current position
    pub fn make_move(&mut self, mve: Move) {
        let from = mve & MOVE_FROM;
        let to = (mve & MOVE_TO) >> 6;
        let bit_from = square_to_bitboard(from as u8);
        let bit_to = square_to_bitboard(to as u8);

        let move_flag = mve & MOVE_FLAG;

        let bit_move = bit_from | bit_to;

        let mut do_not_reset_half_move: bool = true;

        let piece;
        let occupation;
        unsafe {
            let ptr = self.pieces.get_unchecked(from as usize);
            piece = *ptr;
            let ptr = self.pieces.get_unchecked(to as usize);
            occupation = *ptr;
        }

        do_not_reset_half_move = piece != PAWN as u8;

        self.en_passant_target_square = 0;

        if self.color_to_move == WHITE {
            //updating castling rights
            if piece == KING as u8 {
                (self.castling_rights[0], self.castling_rights[1]) = (false, false);
            } else if piece == ROOK as u8 {
                if from == 0 {
                    self.castling_rights[0] = false;
                } else if from == 7 {
                    self.castling_rights[1] = false;
                }
            }
            //updating board
            self.w_board ^= bit_move;
            unsafe {
                let ptr = self.pieces.get_unchecked_mut(to as usize);
                *ptr = piece;
                let ptr = self.pieces.get_unchecked_mut(from as usize);
                *ptr = NONE;
                let ptr = self.w_piece_board.get_unchecked_mut(piece as usize);
                *ptr ^= bit_move;
            }

            if move_flag & CAPTURE == CAPTURE {
                do_not_reset_half_move = false;
                if move_flag == EN_PASSANT_CAPTURE {
                    self.b_board ^= bit_to >> 8;
                    self.b_piece_board[PAWN] ^= bit_to >> 8;
                    self.pieces[to as usize - 8] = NONE;
                } else {
                    self.b_board ^= bit_to;
                    unsafe {
                        let ptr = self.b_piece_board.get_unchecked_mut(occupation as usize);
                        *ptr ^= bit_to;
                    }
                    if to == 56 {
                        self.castling_rights[2] = false;
                    } else if to == 63 {
                        self.castling_rights[3] = false;
                    }
                }
            }
            match move_flag {
                QUIET_MOVE => {}
                DOUBLE_PAWN_PUSH => {
                    self.en_passant_target_square = bit_to >> 8;
                }
                KING_CASTLE => {
                    self.w_board ^= 0b1u64 | (0b1u64 << 2);
                    self.w_piece_board[ROOK] ^= 0b1u64 | (0b1u64 << 2);
                    self.pieces[0] = NONE;
                    self.pieces[2] = ROOK as u8;
                }
                QUEEN_CASTLE => {
                    self.w_board ^= (0b1u64 << 4) | (0b1u64 << 7);
                    self.w_piece_board[ROOK] ^= (0b1u64 << 4) | (0b1u64 << 7);
                    self.pieces[7] = NONE;
                    self.pieces[4] = ROOK as u8;
                }
                _ => {
                    if move_flag & PROMOTION == PROMOTION {
                        let promotion = ((move_flag >> 12) & 3) as usize;
                        self.w_piece_board[PAWN] ^= bit_to;
                        unsafe {
                            let ptr = self.w_piece_board.get_unchecked_mut(promotion);
                            *ptr |= bit_to;
                            let ptr = self.pieces.get_unchecked_mut(to as usize);
                            *ptr = promotion as u8;
                        }
                    }
                }
            }
        } else {
            //full moves are incremented after black moves
            self.full_moves += 1;

            //updating castling rights
            if piece == KING as u8 {
                (self.castling_rights[2], self.castling_rights[3]) = (false, false);
            } else if piece == ROOK as u8 {
                if from == 56 {
                    self.castling_rights[2] = false;
                } else if from == 63 {
                    self.castling_rights[3] = false;
                }
            }
            //updating board
            self.b_board ^= bit_move;
            unsafe {
                let ptr = self.pieces.get_unchecked_mut(to as usize);
                *ptr = piece;
                let ptr = self.pieces.get_unchecked_mut(from as usize);
                *ptr = NONE;
                let ptr = self.b_piece_board.get_unchecked_mut(piece as usize);
                *ptr ^= bit_move;
            }

            if move_flag & CAPTURE == CAPTURE {
                do_not_reset_half_move = false;
                if move_flag == EN_PASSANT_CAPTURE {
                    self.w_board ^= bit_to << 8;
                    self.w_piece_board[PAWN] ^= bit_to << 8;
                    self.pieces[to as usize + 8] = NONE;
                } else {
                    self.w_board ^= bit_to;
                    unsafe {
                        let ptr = self.w_piece_board.get_unchecked_mut(occupation as usize);
                        *ptr ^= bit_to;
                    }
                    if to == 0 {
                        self.castling_rights[0] = false;
                    } else if to == 7 {
                        self.castling_rights[1] = false;
                    }
                }
            }

            match move_flag {
                QUIET_MOVE => {}
                DOUBLE_PAWN_PUSH => {
                    self.en_passant_target_square = bit_to << 8;
                }
                KING_CASTLE => {
                    self.b_board ^= (0b1u64 << 56) | (0b1u64 << 58);
                    self.b_piece_board[ROOK] ^= (0b1u64 << 56) | (0b1u64 << 58);
                    self.pieces[56] = NONE;
                    self.pieces[58] = ROOK as u8;
                }
                QUEEN_CASTLE => {
                    self.b_board ^= (0b1u64 << 60) | (0b1u64 << 63);
                    self.b_piece_board[ROOK] ^= (0b1u64 << 60) | (0b1u64 << 63);
                    self.pieces[63] = NONE;
                    self.pieces[60] = ROOK as u8;
                }
                _ => {
                    if move_flag & PROMOTION == PROMOTION {
                        let promotion = ((move_flag >> 12) & 3) as usize;
                        self.b_piece_board[PAWN] ^= bit_to;
                        unsafe {
                            let ptr = self.b_piece_board.get_unchecked_mut(promotion);
                            *ptr |= bit_to;
                            let ptr = self.pieces.get_unchecked_mut(to as usize);
                            *ptr = promotion as u8;
                        }
                    }
                }
            }
        }
        self.half_move_clock += 1;
        self.half_move_clock *= (do_not_reset_half_move as u32);
        self.blocker_board = self.w_board | self.b_board;
        self.color_to_move = !self.color_to_move;
    }
}
