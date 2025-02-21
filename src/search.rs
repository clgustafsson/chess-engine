use crate::{
    board::{Position, Result},
    eval::evaluate,
    moveorder::{mvv_lva, promising},
    mve::{move_to_algebraic, Move, NULL_MOVE},
};
use std::{
    cmp::max,
    sync::{
        mpsc::{channel, Receiver},
        Mutex, MutexGuard,
    },
    thread,
    time::Duration,
};

//using a global mutable var is safe in this case
//since the only race condition that can occur is not dangerous
//using a mutex here would only introduce overhead
static mut STOP_SEARCH: bool = false;

pub static SEARCH_ONGOING: Mutex<bool> = Mutex::new(false);

#[inline]
fn stop_search() -> bool {
    unsafe { STOP_SEARCH }
}

fn start_search() {
    unsafe {
        STOP_SEARCH = false;
    }
}

enum SearchTimerSignal {
    StopLight,
}

//stops the search after a specified duration, the planned stop can be canceled by sending a stop light to the reciever
fn stop_search_after(search_time: Duration, stop_light: Receiver<SearchTimerSignal>) {
    thread::sleep(search_time);
    match stop_light.try_recv() {
        Ok(stop_signal) => {} //if a stop light is recieved, the search is already manually cancelled
        //do nothing as cancelling may stop the next search unintentionally
        _ => unsafe {
            STOP_SEARCH = true;
        },
    }
}

pub fn stop_search_immediately() {
    unsafe {
        STOP_SEARCH = true;
    }
}

pub fn search_is_ongoing() -> bool {
    {
        let mut guard: MutexGuard<bool> = SEARCH_ONGOING.lock().unwrap();
        *guard
    }
}

const INF: i32 = i32::MAX;
const NEG_INF: i32 = i32::MIN + 1;

//this can be any number as it wont be used
//this const only exist for readability
const EVAL_WONT_BE_USED: i32 = 0;

//searches the position at depth 1, then 2, then 3... this is actually faster than seaching to
//a specified depth directly thanks to alpha beta pruning and always starting searches with the best move
//from the previous depth. If a forced checkmate is found the seach will immediatly finish as the position
//will be hard solved, it will always find the shortest mate for the attacking player, and the longest delaying
//sequence for the defending player
pub fn search_iterative_deepening(pos: &mut Position, search_time: Option<Duration>) {
    let mut depth = 1;
    let mut best_move = NULL_MOVE;

    let (sender, receiver) = channel::<SearchTimerSignal>();
    start_search();
    if let Some(time) = search_time {
        thread::spawn(move || {
            stop_search_after(time, receiver);
        });
    }

    loop {
        let (mve, eval) = search(pos, depth, best_move);
        if stop_search() {
            println!("bestmove {}", move_to_algebraic(mve));
            break;
        } else if eval == INF || eval == NEG_INF {
            println!(
                "info depth {} score mate {} pv {}",
                depth,
                if eval == INF {
                    depth as i8 / 2
                } else {
                    -(depth as i8 / 2)
                },
                move_to_algebraic(mve),
            );
            println!("bestmove {}", move_to_algebraic(mve));
            break;
        }

        println!(
            "info depth {} score cp {} pv {}",
            depth,
            eval,
            move_to_algebraic(mve),
        );
        best_move = mve;
        depth += 1;
    }
    //incase the search is manually stopped we need to make sure to cancel the planned stop
    sender.send(SearchTimerSignal::StopLight);
    {
        let mut guard: MutexGuard<bool> = SEARCH_ONGOING.lock().unwrap();
        *guard = false;
    }
}

//negamax seach helper
fn search(pos: &mut Position, depth: u8, prev_best_move: Move) -> (Move, i32) {
    let mut legal_moves = pos.legal_moves();

    let mut alpha = NEG_INF;
    let mut best_move = prev_best_move;

    if prev_best_move != NULL_MOVE {
        let mut pos_clone = pos.clone();
        pos_clone.make_move(prev_best_move);
        alpha = -negamax_search(&mut pos_clone, depth - 1, NEG_INF, -alpha);
    }
    legal_moves.0[..legal_moves.1].sort_unstable_by_key(|x| promising(pos, x));

    for i in 0..legal_moves.1 {
        let mve = legal_moves.0[i];
        //prevent the prev best move form being searched twice
        if mve == prev_best_move {
            continue;
        }
        let mut pos_clone = pos.clone();
        pos_clone.make_move(mve);
        let eval = -negamax_search(&mut pos_clone, depth - 1, NEG_INF, -alpha);
        if stop_search() {
            return (best_move, EVAL_WONT_BE_USED);
        }
        if eval > alpha || best_move == NULL_MOVE {
            alpha = eval;
            best_move = mve;
        }
    }
    (best_move, alpha)
}

//negamax with alpha-beta pruning
fn negamax_search(pos: &mut Position, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    if stop_search() {
        return EVAL_WONT_BE_USED;
    }
    if depth == 0 {
        return quiescence_search(pos, alpha, beta);
    }

    let mut legal_moves = pos.legal_moves();

    if legal_moves.1 == 0 {
        if pos.result == Result::Checkmate {
            return NEG_INF;
        }
        return 0;
    }
    legal_moves.0[..legal_moves.1].sort_unstable_by_key(|x| promising(pos, x));

    for i in 0..legal_moves.1 {
        let mut pos_clone = pos.clone();
        pos_clone.make_move(legal_moves.0[i]);
        let eval = -negamax_search(&mut pos_clone, depth - 1, -beta, -alpha);
        if eval >= beta {
            return beta;
        }
        alpha = max(alpha, eval);
    }
    alpha
}

//evaluating a position when the depth is reached is dangerous
//due to the horizon effect which can be reduced with a quiescence search
//https://www.chessprogramming.org/Quiescence_Search
fn quiescence_search(pos: &mut Position, mut alpha: i32, beta: i32) -> i32 {
    if stop_search() {
        return EVAL_WONT_BE_USED;
    }
    let eval = evaluate(pos);
    if eval >= beta {
        return beta;
    }
    alpha = max(alpha, eval);

    let mut legal_captures = pos.legal_captures();
    legal_captures.0[..legal_captures.1].sort_unstable_by_key(|x| mvv_lva(pos, x));

    for i in 0..legal_captures.1 {
        let mut pos_clone = pos.clone();
        pos_clone.make_move(legal_captures.0[i]);
        let eval = -quiescence_search(&mut pos_clone, -beta, -alpha);
        if eval >= beta {
            return beta;
        }
        alpha = max(alpha, eval);
    }
    alpha
}
