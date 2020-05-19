#![no_std]//tests folder will execute seperately from main.rs so it needs its own #!s.
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![test_runner(bentos::test_runner)]//test_runner located in lib.rs, it can be reached by 'bentos::'
use core::panic::PanicInfo;
use bentos::{println,serial_print,serial_println};

#[no_mangle]
pub extern "C" fn _start()->!{
    test_main();
    loop{}
}

fn test_runner(tests:&[&dyn Fn()]) {
    unimplemented!();
}

#[panic_handler]
fn panic(info:&PanicInfo)->!{
    bentos::test_panic_handler(info)//from lib.rs
}

#[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}