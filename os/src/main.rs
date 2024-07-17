#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
// #![feature(panic_info_message)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod lang_items;
mod loader;
mod mm;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[Kernel]Helo World");
    mm::init();
    mm::remap_test();
    
    trap::init();
    // trap::enable_interrupt();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

fn clear_bss(){
    extern "C"{
        fn sbss();
        fn ebss();
    }   
    unsafe{
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8 , ebss as usize - sbss as usize)
            .fill(0);
    }
}