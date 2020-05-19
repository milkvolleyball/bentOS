use alloc::alloc::{GlobalAlloc,Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError,FrameAllocator,Mapper,Page,PageTableFlags,Size4KiB,
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;

pub const HEAP_START:u64 = 0x_4444_4444_0000;
pub const HEAP_SIZE:u64 = 100*1024;//100*1Kib = 100Kib


pub fn init_heap(
    mapper:&mut impl Mapper<Size4KiB>,
    frame_allocator:&mut impl FrameAllocator<Size4KiB>,
) -> Result<(),MapToError<Size4KiB>> {
    //specify virtual memory address and size for heap
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START);
        let heap_end = VirtAddr::new(HEAP_START + HEAP_SIZE - 1);//prefer an inclusive end bound (the last byte of the address included), so subtract 1
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    // iterate over to allocate physical frame for those pages
    for page in page_range {
        //allocate a physical frame that the page should be mapped to using the FrameAllocator::allocate_frame method
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        //PRESENT and WRITABLE means page can be read-and-write.which are good for heap memory.
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        //use ?mark to forward error to caller. On success, returns a MapperFlush instance which update Translation Lookaside Buffer by using flush().
        unsafe{
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
            //initialize heap after mapping the heap pages, since init() already tries to write to heap memory.
            ALLOCATOR.lock().init(HEAP_START as usize,HEAP_SIZE as usize);
        }
    }
    Ok(())
}

/// LockedHeap uses spinning_top::Spinlock for synchronization.
/// This is required because multiple threads could access the ALLOCATOR static at the same time.
/// when using a spinlock or mutex,be care of deadlock.
/// This means that shouldn't perform any allocations in interrupt handlers
/// since they can run anytime and might interrupt an in-progress allocation.
#[global_allocator]
static ALLOCATOR:LockedHeap = LockedHeap::empty();
/*
unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        panic!("dealloc should be never called")
    }
}
*/