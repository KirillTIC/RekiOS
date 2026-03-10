use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
};

pub unsafe fn create_user_page_table(
    physical_memory_offset: VirtAddr,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> PhysFrame {
    unsafe {
        let p4_frame = frame_allocator
            .allocate_frame()
            .expect("No one frame for P4 table");
        let p4_virt = physical_memory_offset + p4_frame.start_address().as_u64();
        let p4: &mut PageTable = &mut *(p4_virt.as_mut_ptr());
        p4.zero();

        let (kernel_p4_frame, _) = Cr3::read();
        let kernel_p4_virt = physical_memory_offset + kernel_p4_frame.start_address().as_u64();
        let kernel_p4: &PageTable = &*(kernel_p4_virt.as_ptr());

        for i in 0..512 {
            if !kernel_p4[i].is_unused() {
                p4[i] = kernel_p4[i].clone();
            }
        }

        p4_frame
    }
}
pub unsafe fn map_user_segment(
    page_table: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    virt_addr: VirtAddr,
    size: u64,
) {
    unsafe {
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::USER_ACCESSIBLE
            | PageTableFlags::NO_EXECUTE;
        let start_page: Page<Size4KiB> = Page::containing_address(virt_addr);
        let end_page: Page<Size4KiB> = Page::containing_address(virt_addr + size - 1u64);

        for page in Page::range_inclusive(start_page, end_page) {
            let frame = frame_allocator
                .allocate_frame()
                .expect("No one frame for segment");
            page_table
                .map_to(page, frame, flags, frame_allocator)
                .expect("Error of mapping segment")
                .flush();
        }
    }
}
