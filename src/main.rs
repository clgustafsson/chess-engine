#![allow(unused)]
mod board;
mod eval;
mod init;
mod legalmoves;
mod magic;
mod makemove;
mod masks;
mod moveorder;
mod mve;
mod perft;
mod piece;
mod rand;
mod search;
mod uci;
mod ucigo;

use uci::uci;

fn main() {
    uci();
}
