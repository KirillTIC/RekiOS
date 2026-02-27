#![no_std]
#![feature(abi_x86_interrupt)]

use x86_64::VirtAddr;

pub mod arch;
pub mod display;
pub mod drivers;
pub mod memory;

pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let memory_regions = &boot_info.memory_regions;
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();

    unsafe {
        arch::pic::PICS.lock().initialize();
        arch::pic::PICS.lock().write_masks(0, 0);
    }
    arch::gdt::init();
    arch::interrupts::init_idt();

    display::shell::init(display::framebuffer::FrameBuffer::new(framebuffer));
    x86_64::instructions::interrupts::enable();

    let mut page_table = unsafe { memory::page_table::init(phys_offset) };
    let mut frame_allocator =
        unsafe { memory::frame_allocator::BumpFrameAllocator::new(memory_regions) };

    memory::heap::init_heap(&mut page_table, &mut frame_allocator);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
