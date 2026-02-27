use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, Size4KiB,
    },
};

use crate::memory::frame_allocator::BumpFrameAllocator;

unsafe fn active_p4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (p4_frame, _) = Cr3::read();
    let phys_addr = p4_frame.start_address();
    let virt_addr = physical_memory_offset + phys_addr.as_u64();
    let ptr = virt_addr.as_mut_ptr::<PageTable>();
    unsafe { &mut *ptr }
}

pub fn map_page(
    page_table: &mut OffsetPageTable,
    frame_allocator: &mut BumpFrameAllocator,
    virtual_addr: VirtAddr,
    flags: PageTableFlags,
) {
    let page: Page<Size4KiB> = Page::containing_address(virtual_addr);
    let frame = frame_allocator
        .allocate_frame()
        .expect("No one of free frames");

    unsafe {
        page_table
            .map_to(page, frame, flags, frame_allocator)
            .expect("Error of mapping")
            .flush();
    }
}
pub fn map_range(
    page_table: &mut OffsetPageTable,
    frame_allocator: &mut BumpFrameAllocator,
    start: VirtAddr,
    size: u64,
    flags: PageTableFlags,
) {
    let page_range = {
        let start_page: Page<Size4KiB> = Page::containing_address(start);
        let end_page = Page::containing_address(start + size - 1u64);
        Page::range_inclusive(start_page, end_page)
    };
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .expect("No one of free frames");

        unsafe {
            page_table
                .map_to(page, frame, flags, frame_allocator)
                .expect("Error of mapping")
                .flush();
        }
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let p4 = active_p4_table(physical_memory_offset);
        OffsetPageTable::new(p4, physical_memory_offset)
    }
}
