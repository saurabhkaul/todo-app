extern crate todo_swamp;

use std::io;
use std::io::BufWriter;
use std::io::prelude::*;
use std::time::{Duration, Instant};

use todo_swamp::*;

pub fn main() {
    let mut tl: TodoList = TodoList::new();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let start_time = Instant::now();
    let flush_interval = Duration::from_secs(1);
    let mut next_flush = start_time + flush_interval;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(l) = line {
            runner::run_line(&l, &mut tl, &mut writer);

            if Instant::now() >= next_flush {
                writer.flush().unwrap();
                next_flush += flush_interval;
            }
        }
    }
}
