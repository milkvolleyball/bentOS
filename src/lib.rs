#![no_std]
#![cfg_attr(test,no_main)]//enable no_main in test-mode, so lib.rs need to own a _start entry and a panic handler 
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]//feature gate for handler function when allocation error occur

use core::panic::PanicInfo;
extern crate alloc;

//mod which added here can be used for test
pub mod serial; //'pub mod' make module usable out of lib.rs.
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;

pub fn hlt_loop()->! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    interrupts::init_idt();
    gdt::init();
    unsafe{ interrupts::PICS.lock().initialize()};
    x86_64::instructions::interrupts::enable();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout)->!{
    panic!("allocation error:{:?}",layout)
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code:QemuExitCode){
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

///User-specified runner function
pub fn test_runner(tests: &[&dyn Fn()]) {//'tests' are functions which annotated with #[test_case]
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}


pub fn test_panic_handler(info:&PanicInfo)->! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

/// Entry point for `cargo xtest`
#[cfg(test)]
use bootloader::{entry_point,BootInfo};
#[cfg(test)]
entry_point!(test_kernel_main);
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}


