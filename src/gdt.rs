use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

// create a static GDT that includes a segment for TSS static:
use x86_64::structures::gdt::{GlobalDescriptorTable,Descriptor,SegmentSelector};
struct Selectors {
    code_selector:SegmentSelector,
    tss_selector:SegmentSelector,
}
lazy_static! {
    static ref GDT:(GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt,Selectors {code_selector,tss_selector})
    };
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);//reload code segment
        load_tss(GDT.1.tss_selector);//load TSS
    }
}

pub const DOUBLE_FAULT_IST_INDEX:u16 = 0; //define 0th IST(Interrupt Stack Table) entry as double fault stack
lazy_static! {
    static ref TSS: TaskStateSegment = {//got a tss
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE:usize = 4096;
            static mut STACK:[u8;STACK_SIZE] = [0;STACK_SIZE];//size of STACK is 4096byte

            let stack_start = VirtAddr::from_ptr(unsafe{&STACK});//static mut risks data race
            let stack_end = stack_start + STACK_SIZE;
            stack_end//write to highest address coz stacks on x86 grow downwards
        };
        tss
    };
}