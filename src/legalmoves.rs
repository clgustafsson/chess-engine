use crate::{
    board::{
        bitboard_to_square, print_bitboard, Bitboard, Position, Result, Square, EMPTY, FILE,
        NOT_ON_A_FILE, NOT_ON_H_FILE, RANK, WHITE,
    },
    magic::MAGIC,
    masks::*,
    mve::{
        move_to_algebraic, Move, BISHOP_PROMOTION, BISHOP_PROMOTION_CAPTURE, CAPTURE,
        DOUBLE_PAWN_PUSH, EN_PASSANT_CAPTURE, KING_CASTLE, KNIGHT_PROMOTION,
        KNIGHT_PROMOTION_CAPTURE, QUEEN_CASTLE, QUEEN_PROMOTION, QUEEN_PROMOTION_CAPTURE,
        ROOK_PROMOTION, ROOK_PROMOTION_CAPTURE,
    },
    piece::{BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK},
};

const W_K_CASTLE_MUST_BE_SAFE_SQUARES: Bitboard = 0b111u64 << 1;
const W_Q_CASTLE_MUST_BE_SAFE_SQUARES: Bitboard = 0b111u64 << 3;
const W_K_CASTLE_MUST_BE_EMPTY_SQUARES: Bitboard = 0b11u64 << 1;
const W_Q_CASTLE_MUST_BE_EMPTY_SQUARES: Bitboard = 0b111u64 << 4;

const B_K_CASTLE_MUST_BE_SAFE_SQUARES: Bitboard = 0b111u64 << 57;
const B_Q_CASTLE_MUST_BE_SAFE_SQUARES: Bitboard = 0b111u64 << 59;
const B_K_CASTLE_MUST_BE_EMPTY_SQUARES: Bitboard = 0b11u64 << 57;
const B_Q_CASTLE_MUST_BE_EMPTY_SQUARES: Bitboard = 0b111u64 << 60;

const MAX_NUMBER_OF_LEGAL_MOVES: usize = 218;

impl Position {
    pub fn print_legal_moves(&mut self) {
        let moves = self.legal_moves();
        for i in 0..moves.1 {
            let mve = moves.0[i];
            print!("{} ", move_to_algebraic(mve));
        }
        println!();
    }

    //returns array with legal moves and the amount of legal moves
    pub fn legal_moves(&mut self) -> ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize) {
        let mut legal_moves: ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize) =
            ([0; MAX_NUMBER_OF_LEGAL_MOVES], 0);

        let num_checks = self.update_check_and_pinned();

        //If the king is checked twice, only king moves can be legal, so let not check for anything else
        if num_checks == 2 {
            if self.color_to_move == WHITE {
                //To get legal king moves we must know which squares are seen by the opponent
                //Calculating this at once is faster compared to looking for attackers for every square
                let seen_by_opponent = self.seen_by_black();
                let king_pos = bitboard_to_square(self.w_piece_board[KING]);
                let king_moves = self.w_king_move(king_pos) & !seen_by_opponent;
                add_moves_from_bitboard(king_pos, king_moves & !self.b_board, &mut legal_moves);
                add_capture_moves_from_bitboard(
                    king_pos,
                    king_moves & self.b_board,
                    &mut legal_moves,
                );
                if legal_moves.1 == 0 {
                    self.result = Result::Checkmate;
                }
                legal_moves
            } else {
                //To get legal king moves we must know which squares are seen by the opponent
                //Calculating this at once is faster compared to looking for attackers for every square
                let seen_by_opponent = self.seen_by_white();
                let king_pos = bitboard_to_square(self.b_piece_board[KING]);
                let king_moves = self.b_king_move(king_pos) & !seen_by_opponent;
                add_moves_from_bitboard(king_pos, king_moves & !self.w_board, &mut legal_moves);
                add_capture_moves_from_bitboard(
                    king_pos,
                    king_moves & self.w_board,
                    &mut legal_moves,
                );
                if legal_moves.1 == 0 {
                    self.result = Result::Checkmate;
                }
                legal_moves
            }
        } else {
            //1 or 0 checks
            if self.color_to_move == WHITE {
                //Pinned knights can never move
                let mut knights =
                    self.w_piece_board[KNIGHT] & !(self.orthogonal_pin | self.diagonal_pin);
                while knights != 0 {
                    let knight_pos: Square = bitboard_to_square(knights & !(knights - 1)); //getting the last bit
                    knights &= knights - 1; //removing the last bit
                    let moves = self.w_knight_move(knight_pos);
                    add_capture_moves_from_bitboard(
                        knight_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(knight_pos, moves & !self.b_board, &mut legal_moves);
                }
                //Queens are handled like a rook and a bishop on the same square

                //Diagonally pinned rooks can never move
                let rooks =
                    (self.w_piece_board[ROOK] | self.w_piece_board[QUEEN]) & !self.diagonal_pin;
                //Orthogonally pinned bishops can never move
                let bishops =
                    (self.w_piece_board[BISHOP] | self.w_piece_board[QUEEN]) & !self.orthogonal_pin;

                //Orthogonally pinned rook moves.
                let mut pinned_rooks = rooks & self.orthogonal_pin;
                while pinned_rooks != 0 {
                    let rook_pos: Square = bitboard_to_square(pinned_rooks & !(pinned_rooks - 1)); //getting the last bit
                    pinned_rooks &= pinned_rooks - 1; //removing the last bit
                    let moves = self.w_pinned_rook_moves(rook_pos);
                    add_capture_moves_from_bitboard(
                        rook_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(rook_pos, moves & !self.b_board, &mut legal_moves);
                }

                //Free rook moves.
                let mut free_rooks = rooks & !self.orthogonal_pin;
                while free_rooks != 0 {
                    let rook_pos: Square = bitboard_to_square(free_rooks & !(free_rooks - 1)); //getting the last bit
                    free_rooks &= free_rooks - 1; //removing the last bit
                    let moves = self.w_rook_moves(rook_pos);
                    add_capture_moves_from_bitboard(
                        rook_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(rook_pos, moves & !self.b_board, &mut legal_moves);
                }

                //Diagonally pinned bishop moves.
                let mut pinned_bishops = bishops & self.diagonal_pin;
                while pinned_bishops != 0 {
                    let bishop_pos: Square =
                        bitboard_to_square(pinned_bishops & !(pinned_bishops - 1)); //getting the last bit
                    pinned_bishops &= pinned_bishops - 1; //removing the last bit
                    let moves = self.w_pinned_bishop_moves(bishop_pos);
                    add_capture_moves_from_bitboard(
                        bishop_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(bishop_pos, moves & !self.b_board, &mut legal_moves);
                }

                //Free bishop moves.
                let mut free_bishops = bishops & !self.diagonal_pin;
                while free_bishops != 0 {
                    let bishop_pos: Square = bitboard_to_square(free_bishops & !(free_bishops - 1)); //getting the last bit
                    free_bishops &= free_bishops - 1; //removing the last bit
                    let moves = self.w_bishop_moves(bishop_pos);
                    add_capture_moves_from_bitboard(
                        bishop_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(bishop_pos, moves & !self.b_board, &mut legal_moves);
                }

                let pawns = self.w_piece_board[PAWN];
                //Diagonally pinned pawn can never go forward
                let marching_pawns = pawns & !self.diagonal_pin;
                //Orthogonally pinned pawns can never capture
                let capturing_pawns = pawns & !self.orthogonal_pin;

                let double_marchers = marching_pawns & RANK[1];

                //Orthogonally pinned marchers
                let mut pinned_marchers = marching_pawns & self.orthogonal_pin;
                while pinned_marchers != 0 {
                    let pawn_pos: Bitboard = pinned_marchers & !(pinned_marchers - 1); //getting the last bit
                    pinned_marchers &= pinned_marchers - 1; //removing the last bit
                    let moves = self.w_pinned_pawn_forward_mask(&pawn_pos);
                    add_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[7],
                        &mut legal_moves,
                    );
                    add_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[7],
                        &mut legal_moves,
                    );
                }
                //Free marchers
                let mut marchers = marching_pawns & !self.orthogonal_pin;
                while marchers != 0 {
                    let pawn_pos: Bitboard = marchers & !(marchers - 1); //getting the last bit
                    marchers &= marchers - 1; //removing the last bit
                    let moves = self.w_pawn_forward_mask(&pawn_pos);
                    add_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[7],
                        &mut legal_moves,
                    );
                    add_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[7],
                        &mut legal_moves,
                    )
                }
                //Orthogonally pinned double marchers
                let mut pinned_marchers = double_marchers & self.orthogonal_pin;
                while pinned_marchers != 0 {
                    let pawn_pos: Bitboard = pinned_marchers & !(pinned_marchers - 1); //getting the last bit
                    pinned_marchers &= pinned_marchers - 1; //removing the last bit
                    let moves = self.w_pinned_pawn_doubleforward_mask(&pawn_pos);
                    add_double_forward_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves,
                        &mut legal_moves,
                    );
                }
                //Free double marchers
                let mut marchers = double_marchers & !self.orthogonal_pin;
                while marchers != 0 {
                    let pawn_pos: Bitboard = marchers & !(marchers - 1); //getting the last bit
                    marchers &= marchers - 1; //removing the last bit
                    let moves = self.w_pawn_doubleforward_mask(&pawn_pos);
                    add_double_forward_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves,
                        &mut legal_moves,
                    );
                }

                //Diagonally pinned capturers
                let mut pinned_capturers = capturing_pawns & self.diagonal_pin;
                while pinned_capturers != 0 {
                    let pawn_pos: Bitboard = pinned_capturers & !(pinned_capturers - 1); //getting the last bit
                    pinned_capturers &= pinned_capturers - 1; //removing the last bit
                    let moves = self.w_pinned_pawn_capture_mask(&pawn_pos);
                    add_capture_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[7],
                        &mut legal_moves,
                    );
                    add_capture_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[7],
                        &mut legal_moves,
                    );
                }

                //Free capturers
                let mut capturers = capturing_pawns & !self.diagonal_pin;
                while capturers != 0 {
                    let pawn_pos: Bitboard = capturers & !(capturers - 1); //getting the last bit
                    capturers &= capturers - 1; //removing the last bit
                    let moves = self.w_pawn_capture_mask(&pawn_pos);
                    add_capture_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[7],
                        &mut legal_moves,
                    );
                    add_capture_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[7],
                        &mut legal_moves,
                    );
                }
                //The best way to find the candiates is to use the enemy capture mask from the en passant target square
                let en_passant_pawns =
                    capturing_pawns & b_pawn_capture_mask(&self.en_passant_target_square);

                //Diagonally pinned en passant
                let mut pinned_en_passant = en_passant_pawns & self.diagonal_pin;
                while pinned_en_passant != 0 {
                    let pawn_pos: Bitboard = pinned_en_passant & !(pinned_en_passant - 1); //getting the last bit
                    pinned_en_passant &= pinned_en_passant - 1; //removing the last bit
                    let moves = self.w_pinned_pawn_en_passant_mask(&pawn_pos);
                    self.add_w_en_passant_moves(pawn_pos, moves, &mut legal_moves);
                }

                //En passant
                let mut en_passant = en_passant_pawns & !self.diagonal_pin;
                while en_passant != 0 {
                    let pawn_pos: Bitboard = en_passant & !(en_passant - 1); //getting the last bit
                    en_passant &= en_passant - 1; //removing the last bit
                    let moves = self.w_pawn_en_passant_mask(&pawn_pos);
                    self.add_w_en_passant_moves(pawn_pos, moves, &mut legal_moves);
                }

                //To get legal king moves we must know which squares are seen by the opponent
                //Calculating this at once is faster compared to looking for attackers for every square
                let seen_by_opponent = self.seen_by_black();
                let king_pos = bitboard_to_square(self.w_piece_board[KING]);
                let king_moves = self.w_king_move(king_pos) & !seen_by_opponent;
                add_moves_from_bitboard(king_pos, king_moves & !self.b_board, &mut legal_moves);
                add_capture_moves_from_bitboard(
                    king_pos,
                    king_moves & self.b_board,
                    &mut legal_moves,
                );

                //castling moves
                if self.castling_rights[0]
                    && (W_K_CASTLE_MUST_BE_SAFE_SQUARES & seen_by_opponent)
                        | (W_K_CASTLE_MUST_BE_EMPTY_SQUARES & self.blocker_board)
                        == 0
                {
                    let mve = 3 | (1 << 6) | KING_CASTLE;
                    unsafe {
                        let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
                        *ptr = mve;
                    }
                    legal_moves.1 += 1;
                }

                if self.castling_rights[1]
                    && (W_Q_CASTLE_MUST_BE_SAFE_SQUARES & seen_by_opponent)
                        | (W_Q_CASTLE_MUST_BE_EMPTY_SQUARES & self.blocker_board)
                        == 0
                {
                    let mve = 3 | (5 << 6) | QUEEN_CASTLE;
                    unsafe {
                        let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
                        *ptr = mve;
                    }
                    legal_moves.1 += 1;
                }
                if legal_moves.1 == 0 {
                    self.result = if num_checks == 0 {
                        Result::Draw
                    } else {
                        Result::Checkmate
                    }
                }
                legal_moves
            } else {
                //Pinned knights can never move
                let mut knights =
                    self.b_piece_board[KNIGHT] & !(self.orthogonal_pin | self.diagonal_pin);
                while knights != 0 {
                    let knight_pos: Square = bitboard_to_square(knights & !(knights - 1)); //getting the last bit
                    knights &= knights - 1; //removing the last bit
                    let moves = self.b_knight_move(knight_pos);
                    add_capture_moves_from_bitboard(
                        knight_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(knight_pos, moves & !self.w_board, &mut legal_moves);
                }
                //Queens are handled like a rook and a bishop on the same square

                //Diagonally pinned rooks can never move
                let rooks =
                    (self.b_piece_board[ROOK] | self.b_piece_board[QUEEN]) & !self.diagonal_pin;
                //Orthogonally pinned bishops can never move
                let bishops =
                    (self.b_piece_board[BISHOP] | self.b_piece_board[QUEEN]) & !self.orthogonal_pin;

                //Orthogonally pinned rook moves.
                let mut pinned_rooks = rooks & self.orthogonal_pin;
                while pinned_rooks != 0 {
                    let rook_pos: Square = bitboard_to_square(pinned_rooks & !(pinned_rooks - 1)); //getting the last bit
                    pinned_rooks &= pinned_rooks - 1; //removing the last bit
                    let moves = self.b_pinned_rook_moves(rook_pos);
                    add_capture_moves_from_bitboard(
                        rook_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(rook_pos, moves & !self.w_board, &mut legal_moves);
                }

                //Free rook moves.
                let mut free_rooks = rooks & !self.orthogonal_pin;
                while free_rooks != 0 {
                    let rook_pos: Square = bitboard_to_square(free_rooks & !(free_rooks - 1)); //getting the last bit
                    free_rooks &= free_rooks - 1; //removing the last bit
                    let moves = self.b_rook_moves(rook_pos);
                    add_capture_moves_from_bitboard(
                        rook_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(rook_pos, moves & !self.w_board, &mut legal_moves);
                }

                //Diagonally pinned bishop moves.
                let mut pinned_bishops = bishops & self.diagonal_pin;
                while pinned_bishops != 0 {
                    let bishop_pos: Square =
                        bitboard_to_square(pinned_bishops & !(pinned_bishops - 1)); //getting the last bit
                    pinned_bishops &= pinned_bishops - 1; //removing the last bit
                    let moves = self.b_pinned_bishop_moves(bishop_pos);
                    add_capture_moves_from_bitboard(
                        bishop_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(bishop_pos, moves & !self.w_board, &mut legal_moves);
                }

                //Free bishop moves.
                let mut free_bishops = bishops & !self.diagonal_pin;
                while free_bishops != 0 {
                    let bishop_pos: Square = bitboard_to_square(free_bishops & !(free_bishops - 1)); //getting the last bit
                    free_bishops &= free_bishops - 1; //removing the last bit
                    let moves = self.b_bishop_moves(bishop_pos);
                    add_capture_moves_from_bitboard(
                        bishop_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                    add_moves_from_bitboard(bishop_pos, moves & !self.w_board, &mut legal_moves);
                }

                let pawns = self.b_piece_board[PAWN];
                //Diagonally pinned pawn can never go forward
                let marching_pawns = pawns & !self.diagonal_pin;
                //Orthogonally pinned pawns can never capture
                let capturing_pawns = pawns & !self.orthogonal_pin;

                let double_marchers = marching_pawns & RANK[6];

                //Orthogonally pinned marchers
                let mut pinned_marchers = marching_pawns & self.orthogonal_pin;
                while pinned_marchers != 0 {
                    let pawn_pos: Bitboard = pinned_marchers & !(pinned_marchers - 1); //getting the last bit
                    pinned_marchers &= pinned_marchers - 1; //removing the last bit
                    let moves = self.b_pinned_pawn_forward_mask(&pawn_pos);
                    add_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[0],
                        &mut legal_moves,
                    );
                    add_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[0],
                        &mut legal_moves,
                    );
                }
                //Free marchers
                let mut marchers = marching_pawns & !self.orthogonal_pin;
                while marchers != 0 {
                    let pawn_pos: Bitboard = marchers & !(marchers - 1); //getting the last bit
                    marchers &= marchers - 1; //removing the last bit
                    let moves = self.b_pawn_forward_mask(&pawn_pos);
                    add_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[0],
                        &mut legal_moves,
                    );
                    add_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[0],
                        &mut legal_moves,
                    )
                }
                //Orthogonally pinned double marchers
                let mut pinned_marchers = double_marchers & self.orthogonal_pin;
                while pinned_marchers != 0 {
                    let pawn_pos: Bitboard = pinned_marchers & !(pinned_marchers - 1); //getting the last bit
                    pinned_marchers &= pinned_marchers - 1; //removing the last bit
                    let moves = self.b_pinned_pawn_doubleforward_mask(&pawn_pos);
                    add_double_forward_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves,
                        &mut legal_moves,
                    );
                }
                //Free double marchers
                let mut marchers = double_marchers & !self.orthogonal_pin;
                while marchers != 0 {
                    let pawn_pos: Bitboard = marchers & !(marchers - 1); //getting the last bit
                    marchers &= marchers - 1; //removing the last bit
                    let moves = self.b_pawn_doubleforward_mask(&pawn_pos);
                    add_double_forward_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves,
                        &mut legal_moves,
                    );
                }

                //Diagonally pinned capturers
                let mut pinned_capturers = capturing_pawns & self.diagonal_pin;
                while pinned_capturers != 0 {
                    let pawn_pos: Bitboard = pinned_capturers & !(pinned_capturers - 1); //getting the last bit
                    pinned_capturers &= pinned_capturers - 1; //removing the last bit
                    let moves = self.b_pinned_pawn_capture_mask(&pawn_pos);
                    add_capture_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[0],
                        &mut legal_moves,
                    );
                    add_capture_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[0],
                        &mut legal_moves,
                    );
                }

                //Free capturers
                let mut capturers = capturing_pawns & !self.diagonal_pin;
                while capturers != 0 {
                    let pawn_pos: Bitboard = capturers & !(capturers - 1); //getting the last bit
                    capturers &= capturers - 1; //removing the last bit
                    let moves = self.b_pawn_capture_mask(&pawn_pos);
                    add_capture_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[0],
                        &mut legal_moves,
                    );
                    add_capture_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[0],
                        &mut legal_moves,
                    );
                }
                //The best way to find the candiates is to use the enemy capture mask from the en passant target square
                let en_passant_pawns =
                    capturing_pawns & w_pawn_capture_mask(&self.en_passant_target_square);

                //Diagonally pinned en passant
                let mut pinned_en_passant = en_passant_pawns & self.diagonal_pin;
                while pinned_en_passant != 0 {
                    let pawn_pos: Bitboard = pinned_en_passant & !(pinned_en_passant - 1); //getting the last bit
                    pinned_en_passant &= pinned_en_passant - 1; //removing the last bit
                    let moves = self.b_pinned_pawn_en_passant_mask(&pawn_pos);
                    self.add_b_en_passant_moves(pawn_pos, moves, &mut legal_moves);
                }

                //En passant
                let mut en_passant = en_passant_pawns & !self.diagonal_pin;
                while en_passant != 0 {
                    let pawn_pos: Bitboard = en_passant & !(en_passant - 1); //getting the last bit
                    en_passant &= en_passant - 1; //removing the last bit
                    let moves = self.b_pawn_en_passant_mask(&pawn_pos);
                    self.add_b_en_passant_moves(pawn_pos, moves, &mut legal_moves);
                }

                //To get legal king moves we must know which squares are seen by the opponent
                //Calculating this at once is faster compared to looking for attackers for every square
                let seen_by_opponent = self.seen_by_white();
                let king_pos = bitboard_to_square(self.b_piece_board[KING]);
                let king_moves = self.b_king_move(king_pos) & !seen_by_opponent;
                add_moves_from_bitboard(king_pos, king_moves & !self.w_board, &mut legal_moves);
                add_capture_moves_from_bitboard(
                    king_pos,
                    king_moves & self.w_board,
                    &mut legal_moves,
                );

                //castling moves
                if self.castling_rights[2]
                    && (B_K_CASTLE_MUST_BE_SAFE_SQUARES & seen_by_opponent)
                        | (B_K_CASTLE_MUST_BE_EMPTY_SQUARES & self.blocker_board)
                        == 0
                {
                    let mve = 59 | (57 << 6) | KING_CASTLE;
                    unsafe {
                        let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
                        *ptr = mve;
                    }
                    legal_moves.1 += 1;
                }

                if self.castling_rights[3]
                    && (B_Q_CASTLE_MUST_BE_SAFE_SQUARES & seen_by_opponent)
                        | (B_Q_CASTLE_MUST_BE_EMPTY_SQUARES & self.blocker_board)
                        == 0
                {
                    let mve = 59 | (61 << 6) | QUEEN_CASTLE;
                    unsafe {
                        let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
                        *ptr = mve;
                    }
                    legal_moves.1 += 1;
                }
                if legal_moves.1 == 0 {
                    self.result = if num_checks == 0 {
                        Result::Draw
                    } else {
                        Result::Checkmate
                    }
                }
                legal_moves
            }
        }
    }
}

//It is faster to have special case functions than using branching to add the move flags
pub fn add_moves_from_bitboard(
    from: Square,
    mut to: Bitboard,
    legal_moves: &mut ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize),
) {
    while to != 0 {
        let last_bit: Bitboard = to & !(to - 1); //getting the last bit
        to &= to - 1; //removing the last bit
        let mve = from as Move | ((last_bit.trailing_zeros() as Move) << 6);
        unsafe {
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve;
        }
        legal_moves.1 += 1;
    }
}
pub fn add_double_forward_moves_from_bitboard(
    from: Square,
    mut to: Bitboard,
    legal_moves: &mut ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize),
) {
    if to != 0 {
        let mve = from as Move | ((to.trailing_zeros() as Move) << 6);
        unsafe {
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | DOUBLE_PAWN_PUSH;
        }
        legal_moves.1 += 1;
    }
}
pub fn add_capture_moves_from_bitboard(
    from: Square,
    mut to: Bitboard,
    legal_moves: &mut ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize),
) {
    while to != 0 {
        let last_bit: Bitboard = to & !(to - 1); //getting the last bit
        to &= to - 1; //removing the last bit
        let mve = from as Move | ((last_bit.trailing_zeros() as Move) << 6) | CAPTURE;
        unsafe {
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve;
        }
        legal_moves.1 += 1;
    }
}
pub fn add_promotion_moves_from_bitboard(
    from: Square,
    mut to: Bitboard,
    legal_moves: &mut ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize),
) {
    while to != 0 {
        let last_bit: Bitboard = to & !(to - 1); //getting the last bit
        to &= to - 1; //removing the last bit
        let mve = from as Move | ((last_bit.trailing_zeros() as Move) << 6);
        unsafe {
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | QUEEN_PROMOTION;
            legal_moves.1 += 1;
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | KNIGHT_PROMOTION;
            legal_moves.1 += 1;
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | ROOK_PROMOTION;
            legal_moves.1 += 1;
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | BISHOP_PROMOTION;
            legal_moves.1 += 1;
        }
    }
}
pub fn add_capture_promotion_moves_from_bitboard(
    from: Square,
    mut to: Bitboard,
    legal_moves: &mut ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize),
) {
    while to != 0 {
        let last_bit: Bitboard = to & !(to - 1); //getting the last bit
        to &= to - 1; //removing the last bit
        let mve = from as Move | ((last_bit.trailing_zeros() as Move) << 6);
        unsafe {
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | QUEEN_PROMOTION_CAPTURE;
            legal_moves.1 += 1;
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | KNIGHT_PROMOTION_CAPTURE;
            legal_moves.1 += 1;
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | ROOK_PROMOTION_CAPTURE;
            legal_moves.1 += 1;
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | BISHOP_PROMOTION_CAPTURE;
            legal_moves.1 += 1;
        }
    }
}

impl Position {
    pub fn add_w_en_passant_moves(
        &mut self,
        from: Bitboard,
        to: Bitboard,
        legal_moves: &mut ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize),
    ) {
        if to == EMPTY {
            return;
        }
        //the only edge case which the pinned bitboards does not work for is the orthogonal pin
        //on two pieces caused by enpassant being the only move which can remove pieces from two squares

        //Optimization to only check for the edge case if the friendly king and an enemy orthogonal slider exist
        //on the 5th rank
        if (self.w_piece_board[KING] & RANK[4])
            | ((self.b_piece_board[ROOK] | self.b_piece_board[QUEEN]) & RANK[4])
            != 0
        {
            let pawns_to_be_removed = from | (to >> 8);
            //We then do a rook attack from the friendly king and check if it attacks any
            //enemy orthogonal slider along the 5th rank
            let king_vision = self.seen_by_rook_custom_blocker(
                bitboard_to_square(self.w_piece_board[KING]),
                self.blocker_board ^ pawns_to_be_removed,
            ) & RANK[4];
            if king_vision & (self.b_piece_board[ROOK] | self.b_piece_board[QUEEN]) != 0 {
                //en passant is illegal
                return;
            }
        }
        let mve = from.trailing_zeros() as Move | ((to.trailing_zeros() as Move) << 6);
        unsafe {
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | EN_PASSANT_CAPTURE;
        }
        legal_moves.1 += 1;
    }

    pub fn add_b_en_passant_moves(
        &mut self,
        from: Bitboard,
        to: Bitboard,
        legal_moves: &mut ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize),
    ) {
        if to == EMPTY {
            return;
        }
        //the only edge case which the pinned bitboards does not work for is the orthogonal pin
        //on two pieces caused by enpassant being the only move which can remove pieces from two squares

        //Optimization to only check for the edge case if the friendly king and an enemy orthogonal slider exist
        //on the 4th rank
        if (self.b_piece_board[KING] & RANK[3])
            | ((self.w_piece_board[ROOK] | self.w_piece_board[QUEEN]) & RANK[3])
            != 0
        {
            let pawns_to_be_removed = from | (to << 8);
            //We then do a rook attack from the friendly king and check if it attacks any
            //enemy orthogonal slider along the 4th rank
            let king_vision = self.seen_by_rook_custom_blocker(
                bitboard_to_square(self.b_piece_board[KING]),
                self.blocker_board ^ pawns_to_be_removed,
            ) & RANK[3];
            if king_vision & (self.w_piece_board[ROOK] | self.w_piece_board[QUEEN]) != 0 {
                //en passant is illegal
                return;
            }
        }
        let mve = from.trailing_zeros() as Move | ((to.trailing_zeros() as Move) << 6);
        unsafe {
            let ptr = legal_moves.0.get_unchecked_mut(legal_moves.1);
            *ptr = mve | EN_PASSANT_CAPTURE;
        }
        legal_moves.1 += 1;
    }

    //updates the checkmask and pinned masks on the position and returns the amount of checks
    //these masks can then be used to generate legal moves instead of pseudo legal moves
    //legal move generation is about 5 times faster than pseudo legal move generation
    pub fn update_check_and_pinned(&mut self) -> u8 {
        self.orthogonal_pin = 0;
        self.diagonal_pin = 0;

        let mut check_mask: Bitboard = EMPTY;
        let mut num_checks = 0;

        let friendly_piece_board;
        let enemy_piece_board;
        let friendly_board;
        let enemy_board;
        let king_bit_pos;

        if self.color_to_move == WHITE {
            friendly_piece_board = &self.w_piece_board;
            enemy_piece_board = &self.b_piece_board;
            friendly_board = &self.w_board;
            enemy_board = &self.b_board;
            king_bit_pos = friendly_piece_board[KING];
            check_mask |= (w_pawn_capture_mask(&king_bit_pos) & enemy_piece_board[PAWN]);
        } else {
            friendly_piece_board = &self.b_piece_board;
            enemy_piece_board = &self.w_piece_board;
            friendly_board = &self.b_board;
            enemy_board = &self.w_board;
            king_bit_pos = friendly_piece_board[KING];
            check_mask |= (b_pawn_capture_mask(&king_bit_pos) & enemy_piece_board[PAWN]);
        }

        let king_pos = bitboard_to_square(king_bit_pos) as usize;

        unsafe {
            check_mask |= (KNIGHT_MASK[king_pos] & enemy_piece_board[KNIGHT]);
        }

        //there will never be both a pawn and knight check so we only need to check the combined mask
        if check_mask != 0 {
            num_checks += 1;
        }

        let mut ptr = king_bit_pos;
        let mut friendly_pieces = 0;
        let mut maybe_sliding_check: Bitboard = EMPTY;
        let mut maybe_pin: Bitboard = EMPTY;

        //up
        while ptr & RANK[7] == 0 {
            ptr <<= 8;
            if friendly_pieces == 0 {
                maybe_sliding_check |= ptr;
            }
            maybe_pin |= ptr;

            if ptr & friendly_board != 0 {
                friendly_pieces += 1;
                if friendly_pieces == 2 {
                    break;
                }
            } else if ptr & enemy_board != 0 {
                if ptr & (enemy_piece_board[ROOK] | enemy_piece_board[QUEEN]) != 0 {
                    if friendly_pieces == 0 {
                        check_mask |= maybe_sliding_check;
                        num_checks += 1;
                        break;
                    } else if friendly_pieces == 1 {
                        self.orthogonal_pin |= maybe_pin;
                    }
                } else {
                    break;
                }
            }
        }
        ptr = king_bit_pos;
        friendly_pieces = 0;
        maybe_sliding_check = EMPTY;
        maybe_pin = EMPTY;
        //down
        while ptr & RANK[0] == 0 {
            ptr >>= 8;
            if friendly_pieces == 0 {
                maybe_sliding_check |= ptr;
            }
            maybe_pin |= ptr;

            if ptr & friendly_board != 0 {
                friendly_pieces += 1;
                if friendly_pieces == 2 {
                    break;
                }
            } else if ptr & enemy_board != 0 {
                if ptr & (enemy_piece_board[ROOK] | enemy_piece_board[QUEEN]) != 0 {
                    if friendly_pieces == 0 {
                        check_mask |= maybe_sliding_check;
                        num_checks += 1;
                        break;
                    } else if friendly_pieces == 1 {
                        self.orthogonal_pin |= maybe_pin;
                    }
                } else {
                    break;
                }
            }
        }
        ptr = king_bit_pos;
        friendly_pieces = 0;
        maybe_sliding_check = EMPTY;
        maybe_pin = EMPTY;
        //left
        while ptr & FILE[0] == 0 {
            ptr <<= 1;
            if friendly_pieces == 0 {
                maybe_sliding_check |= ptr;
            }
            maybe_pin |= ptr;

            if ptr & friendly_board != 0 {
                friendly_pieces += 1;
                if friendly_pieces == 2 {
                    break;
                }
            } else if ptr & enemy_board != 0 {
                if ptr & (enemy_piece_board[ROOK] | enemy_piece_board[QUEEN]) != 0 {
                    if friendly_pieces == 0 {
                        check_mask |= maybe_sliding_check;
                        num_checks += 1;
                        break;
                    } else if friendly_pieces == 1 {
                        self.orthogonal_pin |= maybe_pin;
                    }
                } else {
                    break;
                }
            }
        }
        ptr = king_bit_pos;
        friendly_pieces = 0;
        maybe_sliding_check = EMPTY;
        maybe_pin = EMPTY;
        //right
        while ptr & FILE[7] == 0 {
            ptr >>= 1;
            if friendly_pieces == 0 {
                maybe_sliding_check |= ptr;
            }
            maybe_pin |= ptr;

            if ptr & friendly_board != 0 {
                friendly_pieces += 1;
                if friendly_pieces == 2 {
                    break;
                }
            } else if ptr & enemy_board != 0 {
                if ptr & (enemy_piece_board[ROOK] | enemy_piece_board[QUEEN]) != 0 {
                    if friendly_pieces == 0 {
                        check_mask |= maybe_sliding_check;
                        num_checks += 1;
                        break;
                    } else if friendly_pieces == 1 {
                        self.orthogonal_pin |= maybe_pin;
                    }
                } else {
                    break;
                }
            }
        }
        ptr = king_bit_pos;
        friendly_pieces = 0;
        maybe_sliding_check = EMPTY;
        maybe_pin = EMPTY;

        //up right
        while ptr & (RANK[7] | FILE[7]) == 0 {
            ptr <<= 7;
            if friendly_pieces == 0 {
                maybe_sliding_check |= ptr;
            }
            maybe_pin |= ptr;

            if ptr & friendly_board != 0 {
                friendly_pieces += 1;
                if friendly_pieces == 2 {
                    break;
                }
            } else if ptr & enemy_board != 0 {
                if ptr & (enemy_piece_board[BISHOP] | enemy_piece_board[QUEEN]) != 0 {
                    if friendly_pieces == 0 {
                        check_mask |= maybe_sliding_check;
                        num_checks += 1;
                        break;
                    } else if friendly_pieces == 1 {
                        self.diagonal_pin |= maybe_pin;
                    }
                } else {
                    break;
                }
            }
        }
        ptr = king_bit_pos;
        friendly_pieces = 0;
        maybe_sliding_check = EMPTY;
        maybe_pin = EMPTY;
        //up left
        while ptr & (RANK[7] | FILE[0]) == 0 {
            ptr <<= 9;
            if friendly_pieces == 0 {
                maybe_sliding_check |= ptr;
            }
            maybe_pin |= ptr;

            if ptr & friendly_board != 0 {
                friendly_pieces += 1;
                if friendly_pieces == 2 {
                    break;
                }
            } else if ptr & enemy_board != 0 {
                if ptr & (enemy_piece_board[BISHOP] | enemy_piece_board[QUEEN]) != 0 {
                    if friendly_pieces == 0 {
                        check_mask |= maybe_sliding_check;
                        num_checks += 1;
                        break;
                    } else if friendly_pieces == 1 {
                        self.diagonal_pin |= maybe_pin;
                    }
                } else {
                    break;
                }
            }
        }
        ptr = king_bit_pos;
        friendly_pieces = 0;
        maybe_sliding_check = EMPTY;
        maybe_pin = EMPTY;
        //down right
        while ptr & (RANK[0] | FILE[7]) == 0 {
            ptr >>= 9;
            if friendly_pieces == 0 {
                maybe_sliding_check |= ptr;
            }
            maybe_pin |= ptr;

            if ptr & friendly_board != 0 {
                friendly_pieces += 1;
                if friendly_pieces == 2 {
                    break;
                }
            } else if ptr & enemy_board != 0 {
                if ptr & (enemy_piece_board[BISHOP] | enemy_piece_board[QUEEN]) != 0 {
                    if friendly_pieces == 0 {
                        check_mask |= maybe_sliding_check;
                        num_checks += 1;
                        break;
                    } else if friendly_pieces == 1 {
                        self.diagonal_pin |= maybe_pin;
                    }
                } else {
                    break;
                }
            }
        }
        ptr = king_bit_pos;
        friendly_pieces = 0;
        maybe_sliding_check = EMPTY;
        maybe_pin = EMPTY;
        //down left
        while ptr & (RANK[0] | FILE[0]) == 0 {
            ptr >>= 7;
            if friendly_pieces == 0 {
                maybe_sliding_check |= ptr;
            }
            maybe_pin |= ptr;

            if ptr & friendly_board != 0 {
                friendly_pieces += 1;
                if friendly_pieces == 2 {
                    break;
                }
            } else if ptr & enemy_board != 0 {
                if ptr & (enemy_piece_board[BISHOP] | enemy_piece_board[QUEEN]) != 0 {
                    if friendly_pieces == 0 {
                        check_mask |= maybe_sliding_check;
                        num_checks += 1;
                        break;
                    } else if friendly_pieces == 1 {
                        self.diagonal_pin |= maybe_pin;
                    }
                } else {
                    break;
                }
            }
        }
        if check_mask == EMPTY {
            self.checked_squares = !check_mask;
        } else {
            self.checked_squares = check_mask;
        }
        num_checks
    }

    fn seen_by_white(&self) -> Bitboard {
        unsafe {
            let mut seen = self.seen_by_w_pawns()
                | KING_MASK[self.w_piece_board[KING].trailing_zeros() as usize];
            let mut knights = self.w_piece_board[KNIGHT];
            while knights != 0 {
                seen |= KNIGHT_MASK[bitboard_to_square(knights & !(knights - 1)) as usize];
                knights &= knights - 1; //removing the last bit
            }
            //sliding pieces can see through the friendly king
            let blocker_without_king = self.blocker_board ^ self.b_piece_board[KING];

            let mut rooks = self.w_piece_board[ROOK] | self.w_piece_board[QUEEN];
            while rooks != 0 {
                seen |= self.seen_by_rook_custom_blocker(
                    bitboard_to_square(rooks & !(rooks - 1)),
                    blocker_without_king,
                );
                rooks &= rooks - 1; //removing the last bit
            }
            let mut bishops = self.w_piece_board[BISHOP] | self.w_piece_board[QUEEN];
            while bishops != 0 {
                seen |= self.seen_by_bishop_custom_blocker(
                    bitboard_to_square(bishops & !(bishops - 1)),
                    blocker_without_king,
                );
                bishops &= bishops - 1; //removing the last bit
            }
            seen
        }
    }

    fn seen_by_black(&self) -> Bitboard {
        unsafe {
            let mut seen = self.seen_by_b_pawns()
                | KING_MASK[bitboard_to_square(self.b_piece_board[KING]) as usize];
            let mut knights = self.b_piece_board[KNIGHT];
            while knights != 0 {
                seen |= KNIGHT_MASK[bitboard_to_square(knights & !(knights - 1)) as usize];
                knights &= knights - 1; //removing the last bit
            }
            //sliding pieces can see through the friendly king
            let blocker_without_king = self.blocker_board ^ self.w_piece_board[KING];
            let mut rooks = self.b_piece_board[ROOK] | self.b_piece_board[QUEEN];
            while rooks != 0 {
                seen |= self.seen_by_rook_custom_blocker(
                    bitboard_to_square(rooks & !(rooks - 1)),
                    blocker_without_king,
                );
                rooks &= rooks - 1; //removing the last bit
            }
            let mut bishops = self.b_piece_board[BISHOP] | self.b_piece_board[QUEEN];
            while bishops != 0 {
                seen |= self.seen_by_bishop_custom_blocker(
                    bitboard_to_square(bishops & !(bishops - 1)),
                    blocker_without_king,
                );
                bishops &= bishops - 1; //removing the last bit
            }
            seen
        }
    }

    //quiescence_search only looks at captures so lets make a modified legal_move function for only finding captures
    //returns array with legal captures and the amount of legal moves
    pub fn legal_captures(&mut self) -> ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize) {
        let mut legal_moves: ([Move; MAX_NUMBER_OF_LEGAL_MOVES], usize) =
            ([0; MAX_NUMBER_OF_LEGAL_MOVES], 0);

        let num_checks = self.update_check_and_pinned();

        //If the king is checked twice, only king moves can be legal, so let not check for anything else
        if num_checks == 2 {
            if self.color_to_move == WHITE {
                //To get legal king moves we must know which squares are seen by the opponent
                //Calculating this at once is faster compared to looking for attackers for every square
                let seen_by_opponent = self.seen_by_black();
                let king_pos = bitboard_to_square(self.w_piece_board[KING]);
                let king_moves = self.w_king_move(king_pos) & !seen_by_opponent;
                add_capture_moves_from_bitboard(
                    king_pos,
                    king_moves & self.b_board,
                    &mut legal_moves,
                );
                if legal_moves.1 == 0 {
                    self.result = Result::Checkmate;
                }
                legal_moves
            } else {
                //To get legal king moves we must know which squares are seen by the opponent
                //Calculating this at once is faster compared to looking for attackers for every square
                let seen_by_opponent = self.seen_by_white();
                let king_pos = bitboard_to_square(self.b_piece_board[KING]);
                let king_moves = self.b_king_move(king_pos) & !seen_by_opponent;
                add_capture_moves_from_bitboard(
                    king_pos,
                    king_moves & self.w_board,
                    &mut legal_moves,
                );
                if legal_moves.1 == 0 {
                    self.result = Result::Checkmate;
                }
                legal_moves
            }
        } else {
            //1 or 0 checks
            if self.color_to_move == WHITE {
                //Pinned knights can never move
                let mut knights =
                    self.w_piece_board[KNIGHT] & !(self.orthogonal_pin | self.diagonal_pin);
                while knights != 0 {
                    let knight_pos: Square = bitboard_to_square(knights & !(knights - 1)); //getting the last bit
                    knights &= knights - 1; //removing the last bit
                    let moves = self.w_knight_move(knight_pos);
                    add_capture_moves_from_bitboard(
                        knight_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                }
                //Queens are handled like a rook and a bishop on the same square

                //Diagonally pinned rooks can never move
                let rooks =
                    (self.w_piece_board[ROOK] | self.w_piece_board[QUEEN]) & !self.diagonal_pin;
                //Orthogonally pinned bishops can never move
                let bishops =
                    (self.w_piece_board[BISHOP] | self.w_piece_board[QUEEN]) & !self.orthogonal_pin;

                //Orthogonally pinned rook moves.
                let mut pinned_rooks = rooks & self.orthogonal_pin;
                while pinned_rooks != 0 {
                    let rook_pos: Square = bitboard_to_square(pinned_rooks & !(pinned_rooks - 1)); //getting the last bit
                    pinned_rooks &= pinned_rooks - 1; //removing the last bit
                    let moves = self.w_pinned_rook_moves(rook_pos);
                    add_capture_moves_from_bitboard(
                        rook_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                }

                //Free rook moves.
                let mut free_rooks = rooks & !self.orthogonal_pin;
                while free_rooks != 0 {
                    let rook_pos: Square = bitboard_to_square(free_rooks & !(free_rooks - 1)); //getting the last bit
                    free_rooks &= free_rooks - 1; //removing the last bit
                    let moves = self.w_rook_moves(rook_pos);
                    add_capture_moves_from_bitboard(
                        rook_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                }

                //Diagonally pinned bishop moves.
                let mut pinned_bishops = bishops & self.diagonal_pin;
                while pinned_bishops != 0 {
                    let bishop_pos: Square =
                        bitboard_to_square(pinned_bishops & !(pinned_bishops - 1)); //getting the last bit
                    pinned_bishops &= pinned_bishops - 1; //removing the last bit
                    let moves = self.w_pinned_bishop_moves(bishop_pos);
                    add_capture_moves_from_bitboard(
                        bishop_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                }

                //Free bishop moves.
                let mut free_bishops = bishops & !self.diagonal_pin;
                while free_bishops != 0 {
                    let bishop_pos: Square = bitboard_to_square(free_bishops & !(free_bishops - 1)); //getting the last bit
                    free_bishops &= free_bishops - 1; //removing the last bit
                    let moves = self.w_bishop_moves(bishop_pos);
                    add_capture_moves_from_bitboard(
                        bishop_pos,
                        moves & self.b_board,
                        &mut legal_moves,
                    );
                }

                let pawns = self.w_piece_board[PAWN];
                //Orthogonally pinned pawns can never capture
                let capturing_pawns = pawns & !self.orthogonal_pin;

                //Diagonally pinned capturers
                let mut pinned_capturers = capturing_pawns & self.diagonal_pin;
                while pinned_capturers != 0 {
                    let pawn_pos: Bitboard = pinned_capturers & !(pinned_capturers - 1); //getting the last bit
                    pinned_capturers &= pinned_capturers - 1; //removing the last bit
                    let moves = self.w_pinned_pawn_capture_mask(&pawn_pos);
                    add_capture_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[7],
                        &mut legal_moves,
                    );
                    add_capture_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[7],
                        &mut legal_moves,
                    );
                }

                //Free capturers
                let mut capturers = capturing_pawns & !self.diagonal_pin;
                while capturers != 0 {
                    let pawn_pos: Bitboard = capturers & !(capturers - 1); //getting the last bit
                    capturers &= capturers - 1; //removing the last bit
                    let moves = self.w_pawn_capture_mask(&pawn_pos);
                    add_capture_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[7],
                        &mut legal_moves,
                    );
                    add_capture_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[7],
                        &mut legal_moves,
                    );
                }
                //The best way to find the candiates is to use the enemy capture mask from the en passant target square
                let en_passant_pawns =
                    capturing_pawns & b_pawn_capture_mask(&self.en_passant_target_square);

                //Diagonally pinned en passant
                let mut pinned_en_passant = en_passant_pawns & self.diagonal_pin;
                while pinned_en_passant != 0 {
                    let pawn_pos: Bitboard = pinned_en_passant & !(pinned_en_passant - 1); //getting the last bit
                    pinned_en_passant &= pinned_en_passant - 1; //removing the last bit
                    let moves = self.w_pinned_pawn_en_passant_mask(&pawn_pos);
                    self.add_w_en_passant_moves(pawn_pos, moves, &mut legal_moves);
                }

                //En passant
                let mut en_passant = en_passant_pawns & !self.diagonal_pin;
                while en_passant != 0 {
                    let pawn_pos: Bitboard = en_passant & !(en_passant - 1); //getting the last bit
                    en_passant &= en_passant - 1; //removing the last bit
                    let moves = self.w_pawn_en_passant_mask(&pawn_pos);
                    self.add_w_en_passant_moves(pawn_pos, moves, &mut legal_moves);
                }

                //To get legal king moves we must know which squares are seen by the opponent
                //Calculating this at once is faster compared to looking for attackers for every square
                let seen_by_opponent = self.seen_by_black();
                let king_pos = bitboard_to_square(self.w_piece_board[KING]);
                let king_moves = self.w_king_move(king_pos) & !seen_by_opponent;
                add_capture_moves_from_bitboard(
                    king_pos,
                    king_moves & self.b_board,
                    &mut legal_moves,
                );
                if legal_moves.1 == 0 {
                    self.result = if num_checks == 0 {
                        Result::Draw
                    } else {
                        Result::Checkmate
                    }
                }
                legal_moves
            } else {
                //Pinned knights can never move
                let mut knights =
                    self.b_piece_board[KNIGHT] & !(self.orthogonal_pin | self.diagonal_pin);
                while knights != 0 {
                    let knight_pos: Square = bitboard_to_square(knights & !(knights - 1)); //getting the last bit
                    knights &= knights - 1; //removing the last bit
                    let moves = self.b_knight_move(knight_pos);
                    add_capture_moves_from_bitboard(
                        knight_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                }
                //Queens are handled like a rook and a bishop on the same square

                //Diagonally pinned rooks can never move
                let rooks =
                    (self.b_piece_board[ROOK] | self.b_piece_board[QUEEN]) & !self.diagonal_pin;
                //Orthogonally pinned bishops can never move
                let bishops =
                    (self.b_piece_board[BISHOP] | self.b_piece_board[QUEEN]) & !self.orthogonal_pin;

                //Orthogonally pinned rook moves.
                let mut pinned_rooks = rooks & self.orthogonal_pin;
                while pinned_rooks != 0 {
                    let rook_pos: Square = bitboard_to_square(pinned_rooks & !(pinned_rooks - 1)); //getting the last bit
                    pinned_rooks &= pinned_rooks - 1; //removing the last bit
                    let moves = self.b_pinned_rook_moves(rook_pos);
                    add_capture_moves_from_bitboard(
                        rook_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                }

                //Free rook moves.
                let mut free_rooks = rooks & !self.orthogonal_pin;
                while free_rooks != 0 {
                    let rook_pos: Square = bitboard_to_square(free_rooks & !(free_rooks - 1)); //getting the last bit
                    free_rooks &= free_rooks - 1; //removing the last bit
                    let moves = self.b_rook_moves(rook_pos);
                    add_capture_moves_from_bitboard(
                        rook_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                }

                //Diagonally pinned bishop moves.
                let mut pinned_bishops = bishops & self.diagonal_pin;
                while pinned_bishops != 0 {
                    let bishop_pos: Square =
                        bitboard_to_square(pinned_bishops & !(pinned_bishops - 1)); //getting the last bit
                    pinned_bishops &= pinned_bishops - 1; //removing the last bit
                    let moves = self.b_pinned_bishop_moves(bishop_pos);
                    add_capture_moves_from_bitboard(
                        bishop_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                }

                //Free bishop moves.
                let mut free_bishops = bishops & !self.diagonal_pin;
                while free_bishops != 0 {
                    let bishop_pos: Square = bitboard_to_square(free_bishops & !(free_bishops - 1)); //getting the last bit
                    free_bishops &= free_bishops - 1; //removing the last bit
                    let moves = self.b_bishop_moves(bishop_pos);
                    add_capture_moves_from_bitboard(
                        bishop_pos,
                        moves & self.w_board,
                        &mut legal_moves,
                    );
                }

                let pawns = self.b_piece_board[PAWN];
                //Orthogonally pinned pawns can never capture
                let capturing_pawns = pawns & !self.orthogonal_pin;

                //Diagonally pinned capturers
                let mut pinned_capturers = capturing_pawns & self.diagonal_pin;
                while pinned_capturers != 0 {
                    let pawn_pos: Bitboard = pinned_capturers & !(pinned_capturers - 1); //getting the last bit
                    pinned_capturers &= pinned_capturers - 1; //removing the last bit
                    let moves = self.b_pinned_pawn_capture_mask(&pawn_pos);
                    add_capture_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[0],
                        &mut legal_moves,
                    );
                    add_capture_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[0],
                        &mut legal_moves,
                    );
                }

                //Free capturers
                let mut capturers = capturing_pawns & !self.diagonal_pin;
                while capturers != 0 {
                    let pawn_pos: Bitboard = capturers & !(capturers - 1); //getting the last bit
                    capturers &= capturers - 1; //removing the last bit
                    let moves = self.b_pawn_capture_mask(&pawn_pos);
                    add_capture_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & !RANK[0],
                        &mut legal_moves,
                    );
                    add_capture_promotion_moves_from_bitboard(
                        bitboard_to_square(pawn_pos),
                        moves & RANK[0],
                        &mut legal_moves,
                    );
                }
                //The best way to find the candiates is to use the enemy capture mask from the en passant target square
                let en_passant_pawns =
                    capturing_pawns & w_pawn_capture_mask(&self.en_passant_target_square);

                //Diagonally pinned en passant
                let mut pinned_en_passant = en_passant_pawns & self.diagonal_pin;
                while pinned_en_passant != 0 {
                    let pawn_pos: Bitboard = pinned_en_passant & !(pinned_en_passant - 1); //getting the last bit
                    pinned_en_passant &= pinned_en_passant - 1; //removing the last bit
                    let moves = self.b_pinned_pawn_en_passant_mask(&pawn_pos);
                    self.add_b_en_passant_moves(pawn_pos, moves, &mut legal_moves);
                }

                //En passant
                let mut en_passant = en_passant_pawns & !self.diagonal_pin;
                while en_passant != 0 {
                    let pawn_pos: Bitboard = en_passant & !(en_passant - 1); //getting the last bit
                    en_passant &= en_passant - 1; //removing the last bit
                    let moves = self.b_pawn_en_passant_mask(&pawn_pos);
                    self.add_b_en_passant_moves(pawn_pos, moves, &mut legal_moves);
                }

                //To get legal king moves we must know which squares are seen by the opponent
                //Calculating this at once is faster compared to looking for attackers for every square
                let seen_by_opponent = self.seen_by_white();
                let king_pos = bitboard_to_square(self.b_piece_board[KING]);
                let king_moves = self.b_king_move(king_pos) & !seen_by_opponent;
                add_capture_moves_from_bitboard(
                    king_pos,
                    king_moves & self.w_board,
                    &mut legal_moves,
                );

                if legal_moves.1 == 0 {
                    self.result = if num_checks == 0 {
                        Result::Draw
                    } else {
                        Result::Checkmate
                    }
                }
                legal_moves
            }
        }
    }
}
