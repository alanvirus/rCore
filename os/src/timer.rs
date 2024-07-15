use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;

pub fn get_time() -> usize{ //get mtime
    time::read()
}

pub fn get_time_ms() -> usize{ //app's runtime in ms
    time::read() / ( CLOCK_FREQ / MSEC_PER_SEC)
}

pub fn set_next_trigger() { //set mtimecmp, time interrupt per 10 ms
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC );
    //debug println!("{}",get_time_ms() as isize);
}
