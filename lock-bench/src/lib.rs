use std::hint::black_box;

use parking_lot::Mutex;

pub fn incr(mut i: i32, count: usize) -> i32 {
    for _ in 0..count {
        i = black_box(i + 1);
    }
    i
}

pub fn incr_locked(i: i32, count: usize) -> i32 {
    let il = Mutex::new(i);
    for _ in 0..count {
        let mut i = il.lock();
        *i = black_box(*i + 1);
    }
    il.into_inner()
}
