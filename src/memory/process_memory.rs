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

        // Upper half (kernel space): share directly
        for i in 256..512 {
            if !kernel_p4[i].is_unused() {
                p4[i] = kernel_p4[i].clone();
            }
        }

        // Lower half: deep-copy P3 and P2 tables so each process
        // gets its own copies and map_to won't pollute shared tables
        for i in 0..256 {
            if kernel_p4[i].is_unused() {
                continue;
            }
            let kernel_p3_phys = kernel_p4[i].frame().unwrap().start_address().as_u64();
            let kernel_p3: &PageTable =
                &*((physical_memory_offset + kernel_p3_phys).as_ptr());

            let user_p3_frame = frame_allocator
                .allocate_frame()
                .expect("No frame for user P3");
            let user_p3: &mut PageTable =
                &mut *((physical_memory_offset + user_p3_frame.start_address().as_u64())
                    .as_mut_ptr());
            user_p3.zero();

            for j in 0..512 {
                if kernel_p3[j].is_unused() {
                    continue;
                }
                if kernel_p3[j].flags().contains(PageTableFlags::HUGE_PAGE) {
                    user_p3[j] = kernel_p3[j].clone();
                    continue;
                }

                let kernel_p2_phys = kernel_p3[j].frame().unwrap().start_address().as_u64();
                let kernel_p2: &PageTable =
                    &*((physical_memory_offset + kernel_p2_phys).as_ptr());

                let user_p2_frame = frame_allocator
                    .allocate_frame()
                    .expect("No frame for user P2");
                let user_p2: &mut PageTable = &mut *((physical_memory_offset
                    + user_p2_frame.start_address().as_u64())
                .as_mut_ptr());

                core::ptr::copy_nonoverlapping(
                    kernel_p2 as *const PageTable,
                    user_p2 as *mut PageTable,
                    1,
                );

                user_p3[j].set_frame(user_p2_frame, kernel_p3[j].flags());
            }

            p4[i].set_frame(user_p3_frame, kernel_p4[i].flags());
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
