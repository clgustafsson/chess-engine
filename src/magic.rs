use crate::board::{Bitboard, Square, FILE, RANK};
use crate::masks::{
    bishop_all_blockers_mask, bishop_mask, rook_all_blockers_mask, rook_mask,
    BISHOP_ALL_BLOCKERS_MASK, ROOK_ALL_BLOCKERS_MASK,
};
use crate::rand::Wyrand;
use Sliding::{Bishop, Rook};

pub type MagicNumber = u64;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sliding {
    Rook,
    Bishop,
}

pub struct Magic {
    pub rook_lookup: [[Bitboard; 4096]; 64],
    pub rook_magic: [MagicNumber; 64],
    pub bishop_lookup: [[Bitboard; 512]; 64],
    pub bishop_magic: [MagicNumber; 64],
}

pub static mut MAGIC: Magic = Magic {
    rook_lookup: [[0; 4096]; 64],
    rook_magic: [0; 64],
    bishop_lookup: [[0; 512]; 64],
    bishop_magic: [0; 64],
};

pub const ROOK_MAGIC_SHIFT: Square = 52;
pub const BISHOP_MAGIC_SHIFT: Square = 55;

pub fn init_magic() {
    for square in 0..64 as Square {
        find_magic(Rook, square);
        find_magic(Bishop, square);
    }
}

//finds a magic number for a square and updates MAGIC with the magic number and the lookup table for that square
fn find_magic(piece: Sliding, square: Square) {
    let mut rng = Wyrand(16113163697346267551); //best seed out of >100_000 random seeds (73292 magic candidates for 128 magics)

    //looping through random numbers until a magic number is found
    loop {
        //rand & rand & rand to get a low amounts of 1s leads to better candidates
        let maybe_magic = rng.next() & rng.next() & rng.next();
        let is_magic: bool = check_if_magic(piece, square, maybe_magic);
        if is_magic {
            break;
        }
    }
}

//checks if a number is magic by looking for hash conditions, magic number should have no
//collisions as they should create a perfect hash function
fn check_if_magic(piece: Sliding, square: Square, magic_candidate: MagicNumber) -> bool {
    let all_blockers_set;
    unsafe {
        if piece == Rook {
            MAGIC.rook_lookup[square as usize] = [0; 4096];
        } else {
            MAGIC.bishop_lookup[square as usize] = [0; 512];
        }
        all_blockers_set = if piece == Rook {
            ROOK_ALL_BLOCKERS_MASK[square as usize]
        } else {
            BISHOP_ALL_BLOCKERS_MASK[square as usize]
        };
    }

    let mut blocker_subset: Bitboard = 0;

    //Carry-Rippler trick to enumerate all subsets in a set
    //https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
    //the set is a mask containing all possible blocking squares
    //so the subsets will be all possible configurations of blocker boards
    loop {
        let move_mask = if piece == Rook {
            rook_mask(0b1u64 << square, blocker_subset)
        } else {
            bishop_mask(0b1u64 << square, blocker_subset)
        };

        //a magic index is the blocker board for the square multiplied with a magic number and then shifted by the amount of relevant blocker squares
        //magic index = (blocker*magic number)>>(magic bitshift);
        //move mask = lookup table [magic index];
        //https://www.chessprogramming.org/Magic_Bitboards
        //this is how the move mask later can be accessed from the lookup table
        let magic_index = if piece == Rook {
            blocker_subset.wrapping_mul(magic_candidate) >> ROOK_MAGIC_SHIFT
        } else {
            blocker_subset.wrapping_mul(magic_candidate) >> BISHOP_MAGIC_SHIFT
        };
        unsafe {
            if piece == Rook {
                if MAGIC.rook_lookup[square as usize][magic_index as usize] == 0 {
                    MAGIC.rook_lookup[square as usize][magic_index as usize] = move_mask;
                } else if MAGIC.rook_lookup[square as usize][magic_index as usize] != move_mask {
                    //bad hash collision
                    //this candidate is not magic!
                    return false;
                }
            } else if MAGIC.bishop_lookup[square as usize][magic_index as usize] == 0 {
                MAGIC.bishop_lookup[square as usize][magic_index as usize] = move_mask;
            } else if MAGIC.bishop_lookup[square as usize][magic_index as usize] != move_mask {
                //bad hash collision
                //this candidate is not magic!
                return false;
            }
        }

        //Carry-Rippler
        blocker_subset = blocker_subset.wrapping_sub(all_blockers_set) & all_blockers_set;
        if blocker_subset == 0 {
            break;
        }
    }
    //no bad hash collisions
    //this candidate is magic!
    if piece == Rook {
        unsafe { MAGIC.rook_magic[square as usize] = magic_candidate }
    } else {
        unsafe { MAGIC.bishop_magic[square as usize] = magic_candidate }
    }
    true
}
