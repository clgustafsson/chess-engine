use crate::{
    board::Position,
    eval::PIECE_VALUES,
    mve::{Move, CAPTURE, MOVE_FLAG, MOVE_FROM, MOVE_TO, PROMOTION},
};

//https://www.chessprogramming.org/MVV-LVA
pub fn mvv_lva(pos: &mut Position, mve: &Move) -> i32 {
    let from = mve & MOVE_FROM;
    let to = (mve & MOVE_TO) >> 6;
    let victim;
    let agressor;
    unsafe {
        let ptr = pos.pieces.get_unchecked(to as usize);
        victim = *ptr;
        let ptr = pos.pieces.get_unchecked(from as usize);
        agressor = *ptr;
    }
    //this will be sorted low to high so low is better
    unsafe {
        let agressor_ptr = PIECE_VALUES.get_unchecked(agressor as usize);
        let victim_ptr = PIECE_VALUES.get_unchecked(victim as usize);
        *agressor_ptr - (*victim_ptr * 2)
    }
}

const PROMOTION_BONUS: i32 = -100000;

//giving promising moves high search priority (low number = high priority);
pub fn promising(pos: &mut Position, mve: &Move) -> i32 {
    if mve & PROMOTION == PROMOTION {
        let promotion = ((mve >> 12) & 3) as usize;
        unsafe {
            let ptr = PIECE_VALUES.get_unchecked(promotion);
            return PROMOTION_BONUS - *ptr;
        }
    }
    if mve & CAPTURE == CAPTURE {
        return mvv_lva(pos, mve);
    }
    0
}
