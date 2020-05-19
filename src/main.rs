#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(bentos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use core::panic::PanicInfo;
use alloc::{boxed::Box,vec,vec::Vec,rc::Rc};
use bentos::{print,println};
use bootloader::{BootInfo,entry_point};

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) ->! {
    use bentos::memory::{self,BootInfoFrameAllocator};
    use bentos::allocator;
    use x86_64::{VirtAddr,structures::paging::MapperAllSizes,structures::paging::Page};//import the MapperAllSizes trait in order to use the translate_addr method it provides.

    bentos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);//get virt addr offset from boot info
    let mut mapper = unsafe {memory::init(phys_mem_offset)};//init a new Offsetpage mapper
    let mut frame_allocator = unsafe{BootInfoFrameAllocator::init(&boot_info.memory_map)};//get usable physical memory info

    /*map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));//build a page contain VirtAddr deadbeaf000
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);//map virtaar deadbeaf000 to 0xb8000(VGA)
 
    let page_ptr:*mut u64 = page.start_address().as_mut_ptr();
    unsafe{page_ptr.offset(300).write_volatile(0x_f021_f077_f065_f04e)};
    */
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let x = Box::new(41);
    println!("heap value address:{:p}",x);
    println!("heap value :{}",*x);

    let mut vec = Vec::new();
    for i in 0..500{
        vec.push(i);
    }
    println!("vec at {:p}",vec.as_slice());

    let refcnt = Rc::new(vec![1,2,3]);
    let clo = refcnt.clone();
    println!("current refcnt is {}",Rc::strong_count(&clo));
    core::mem::drop(refcnt);
    println!("current refcnt is {}",Rc::strong_count(&clo));

    #[cfg(test)]
    test_main();
    
    println!("bentOS is an embedded system lives on tender materials." );
    bentos::hlt_loop();
}

///This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info:&PanicInfo) -> ! {
    println!("{}",info);
    bentos::hlt_loop();
}

///Specify panic function in test-mode.
#[cfg(test)]
#[panic_handler]
fn panic(info:&PanicInfo) -> !{
    bentos::test_panic_handler(info)
}