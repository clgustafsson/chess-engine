use std::{
    sync::MutexGuard,
    thread,
    time::{self, Duration},
};

use crate::{
    board::{Position, BLACK, WHITE},
    search::{search_is_ongoing, search_iterative_deepening, SEARCH_ONGOING},
};

//handles the supported uci flags and starts the corresponding search
pub fn ucigo(pos: &Position, flags: &str) {
    if flags.contains("infinite") {
        search_position_infinite(pos);
        return;
    }
    //supported flags
    let mut wtime: Option<u64> = None;
    let mut btime: Option<u64> = None;
    let mut winc: u64 = 0;
    let mut binc: u64 = 0;

    let flag_elements: Vec<&str> = flags.split_whitespace().collect();
    let len = flag_elements.len();

    //start an infinite search if no flags are provided
    if len == 0 {
        search_position_infinite(pos);
        return;
    }

    for i in (0..len).step_by(2) {
        let flag = flag_elements[i];
        if i + 1 >= flag_elements.len() {
            break;
        }
        let flag_value = flag_elements[i + 1];

        match flag {
            "wtime" => {
                if let Ok(number) = flag_value.parse() {
                    wtime = Some(number);
                } else {
                    unexpeced_ucigo_format();
                    return;
                }
            }
            "btime" => {
                if let Ok(number) = flag_value.parse() {
                    btime = Some(number);
                } else {
                    unexpeced_ucigo_format();
                    return;
                }
            }
            "winc" => {
                if let Ok(number) = flag_value.parse() {
                    winc = number;
                } else {
                    unexpeced_ucigo_format();
                    return;
                }
            }
            "binc" => {
                if let Ok(number) = flag_value.parse() {
                    binc = number;
                } else {
                    unexpeced_ucigo_format();
                    return;
                }
            }
            _ => {}
        }
    }
    //to start a timed search both wtime and btime must be provided
    if let (Some(wtime_ms), Some(btime_ms)) = (wtime, btime) {
        search_position_from_time_info(pos, wtime_ms, btime_ms, winc, binc);
    } else {
        unexpeced_ucigo_format();
    }
}

fn unexpeced_ucigo_format() {
    println!("unexpected flag format");
    println!("supported flags: infinite wtime <value> btime <value> winc <value> binc <value>");
}

//search until "stop" or search timer is finshed
pub fn search_position_from_time_info(
    pos: &Position,
    wtime_ms: u64,
    btime_ms: u64,
    winc_ms: u64,
    binc_ms: u64,
) {
    if search_is_ongoing() {
        println!("cannot start two seaches at once, write \"stop\" to stop the ongoing search")
    } else {
        {
            let mut guard: MutexGuard<bool> = SEARCH_ONGOING.lock().unwrap();
            *guard = true;
        }
        let search_time = match pos.color_to_move {
            WHITE => (wtime_ms / 50) + winc_ms,
            BLACK => (btime_ms / 50) + binc_ms,
        };
        let mut pos_clone = pos.clone();
        thread::spawn(move || {
            search_iterative_deepening(&mut pos_clone, Some(Duration::from_millis(search_time)));
        });
    }
}

//search until "stop"
pub fn search_position_infinite(pos: &Position) {
    if search_is_ongoing() {
        println!("cannot start two seaches at once, write \"stop\" to stop the ongoing search")
    } else {
        {
            let mut guard: MutexGuard<bool> = SEARCH_ONGOING.lock().unwrap();
            *guard = true;
        }
        let mut pos_clone = pos.clone();
        thread::spawn(move || {
            search_iterative_deepening(&mut pos_clone, None);
        });
    }
}
