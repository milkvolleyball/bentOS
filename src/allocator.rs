use alloc::alloc::{GlobalAlloc,Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError,FrameAllocator,Mapper,Page,PageTableFlags,Size4KiB,
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;
use bump::BumpAllocator;

pub mod bump;

pub const HEAP_START:usize = 0x_4444_4444_0000;
pub const HEAP_SIZE:usize = 100*1024;//100*1Kib = 100Kib

/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}impl<A> Locked<A> {
    pub const fn new(inner:A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Align the given address `addr` upwards to alignment `align`.
fn align_up(addr:usize, align:usize) -> usize {
    let remainder = addr%align;
    if remainder ==0 {
        addr //addr already aligned
    } else {
        addr - remainder + align
    }
}
/*
/// more efficient but confused version:
/// Align the given address `addr` upwards to alignment `align`.
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
*/

pub fn init_heap(
    mapper:&mut impl Mapper<Size4KiB>,
    frame_allocator:&mut impl FrameAllocator<Size4KiB>,
) -> Result<(),MapToError<Size4KiB>> {
    //specify virtual memory address and size for heap
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = VirtAddr::new(HEAP_START as u64 + HEAP_SIZE as u64 - 1);//prefer an inclusive end bound (the last byte of the address included), so subtract 1
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
            ALLOCATOR.lock().init(HEAP_START,HEAP_SIZE as usize);
        }
    }
    Ok(())
}

/// LockedHeap uses spinning_top::Spinlock for synchronization.
/// This is required because multiple threads could access the ALLOCATOR static at the same time.
/// when using a spinlock or mutex,be care of deadlock.
/// This means that shouldn't perform any allocations in interrupt handlers
/// since they can run anytime and might interrupt an in-progress allocation.
//we declared BumpAllocator::new and Locked::new as const functions.
//If they were normal functions, a compilation error would occur
//due to initialization expression of a static must evaluable at compile time.
#[global_allocator]
static ALLOCATOR:Locked<BumpAllocator> = Locked::new(BumpAllocator::new());