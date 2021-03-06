#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
use core::panic::PanicInfo;
use bentos::{exit_qemu, QemuExitCode,serial_print,serial_println};
use x86_64::structures::idt::{InterruptDescriptorTable,InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    static ref TEST_IDT:InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
            .set_handler_fn(test_double_fault_handler)// ... for double fault
            .set_stack_index(bentos::gdt::DOUBLE_FAULT_IST_INDEX);//set DOUBLE_FAULT_IST_INDEX(0) as stack for double fault
        }
        idt
    };
}
pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame:&mut InterruptStackFrame, _error_code:u64)->! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start()->! {
    serial_print!("stack_overflow...");
    bentos::gdt::init();
    init_test_idt();

    stack_overflow();
    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bentos::test_panic_handler(info)
}