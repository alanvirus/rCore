#![no_std]
#![no_main]
// #![feature(panic_info_message)]

#[macro_use]
mod console;
mod config;
mod loader;
mod lang_items;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;
pub mod task;
use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Helo World");
    trap::init();
    loader::load_apps();
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