use x86_64::structures::idt::{InterruptDescriptorTable,InterruptStackFrame,PageFaultErrorCode};
use crate::{print,println,gdt};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use crate::hlt_loop;

pub const PIC_1_OFFSET:u8 = 32;
pub const PIC_2_OFFSET:u8 = PIC_1_OFFSET + 8;

#[derive(Debug,Clone,Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // Timer interrupt arrives at the CPU as interrupt 32
    Keyboard, //by default its 33
} impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS:spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)});


lazy_static! {
    static ref IDT:InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();//idt is a struct: https://docs.rs/x86_64/0.10.3/x86_64/structures/idt/struct.InterruptDescriptorTable.html
        idt.breakpoint.set_handler_fn(breakpoint_handler);//register handler for breakpoint
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)// ... for double fault
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);//set DOUBLE_FAULT_IST_INDEX(0) as stack for double fault
        }
        idt[InterruptIndex::Timer.as_usize()]
        .set_handler_fn(timer_interrupt_handler);
        
        idt[InterruptIndex::Keyboard.as_usize()]
        .set_handler_fn(keyboard_interrupt_handler);

        idt.page_fault.set_handler_fn(page_fault_handler);

        idt
    };
}
pub fn init_idt() {
    IDT.load();//make CPU load IDT we created
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame:&mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame:&mut InterruptStackFrame, _error_code:u64)->! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame:&mut InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame:&mut InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts,DecodedKey,HandleControl,Keyboard,ScancodeSet1};
    use spin::Mutex;

    lazy_static! {//creare a static Keyboard object.
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key,ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);//reading a byte from 0x60, the data port of the PS/2 controller
    let scancode:u8 = unsafe {port.read()};//read from 0x60

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {//add_byte method return Option<KeyEvent> which contains which key was pressed or released
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}",character),
                DecodedKey::RawKey(key) => print!("{:?}",key),
            }
        }
    }
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame:&mut InterruptStackFrame, error_code:PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;//CR2 register automatically set by the CPU on a page fault and contains the accessed virtual address that caused the page fault. 
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address:{:?}", Cr2::read());
    println!("Error Code:{:?}",error_code);//giving type of operation which caused page fault(read or write?)
    println!("{:#?}",stack_frame);
    hlt_loop();
}

#[cfg(test)]
use crate::{serial_print,serial_println};

#[test_case]
fn test_breakpoint_exception() {
    serial_print!("test_breakpoint_exception...");
    //invoke a breakpoint ECPT
    x86_64::instructions::interrupts::int3();
    serial_println!("[ok]");
}