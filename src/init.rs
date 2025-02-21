use crate::{magic::*, masks::init_masks};

pub fn initialize_engine() {
    init_masks();
    init_magic();
}
