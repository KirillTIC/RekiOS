#![no_std]
#![feature(abi_x86_interrupt)]

use bootloader_api::info::MemoryRegions;
use shell::shell::SHELL;
use spin::Once;
use x86_64::VirtAddr;

pub mod arch;
pub mod display;
pub mod drivers;
pub mod memory;
pub mod shell;

static MEMORY_REGIONS: Once<&'static MemoryRegions> = Once::new();

pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    MEMORY_REGIONS.call_once(|| &boot_info.memory_regions);
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let memory_regions = &boot_info.memory_regions;
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();

    unsafe {
        arch::pic::PICS.lock().initialize();
        arch::pic::PICS.lock().write_masks(0, 0);
    }
    arch::gdt::init();
    arch::interrupts::init();

    let mut page_table = unsafe { memory::page_table::init(phys_offset) };
    let mut frame_allocator =
        unsafe { memory::frame_allocator::BumpFrameAllocator::new(memory_regions) };

    memory::heap::init(&mut page_table, &mut frame_allocator);
    shell::shell::init(display::framebuffer::FrameBuffer::new(framebuffer));
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();

        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(shell) = SHELL.lock().as_mut() {
                shell.flush();
            }
        });
    }
}
