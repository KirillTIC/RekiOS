use linked_list_allocator::LockedHeap;
use x86_64::{
    VirtAddr,
    structures::paging::{OffsetPageTable, PageTableFlags as Flags},
};

use crate::memory::{frame_allocator::BumpFrameAllocator, page_table::map_range};

const HEAP_START: u64 = 0x_4444_4444_0000;
const HEAP_SIZE: u64 = 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(page_table: &mut OffsetPageTable, frame_allocator: &mut BumpFrameAllocator) {
    let flags = Flags::PRESENT | Flags::WRITABLE | Flags::NO_EXECUTE;

    map_range(
        page_table,
        frame_allocator,
        VirtAddr::new(HEAP_START),
        HEAP_SIZE,
        flags,
    );

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as *mut u8, HEAP_SIZE as usize);
    }
}
