use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
};

pub struct BumpFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}
impl BumpFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions) -> Self {
        Self {
            memory_regions,
            next: 0,
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .flat_map(|r| (r.start..r.end).step_by(4096))
            .map(|addr| {
                let phys = PhysAddr::new(addr);
                PhysFrame::containing_address(phys)
            })
    }
}

unsafe impl FrameAllocator<Size4KiB> for BumpFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
