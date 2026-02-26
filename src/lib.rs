#![no_std]
#![feature(abi_x86_interrupt)]

pub mod arch;
pub mod display;
pub mod drivers;

pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    unsafe {
        arch::pic::PICS.lock().initialize();
        arch::pic::PICS.lock().write_masks(0, 0);
    }
    arch::gdt::init();
    arch::interrupts::init_idt();
    display::shell::init(display::framebuffer::FrameBuffer::new(boot_info));
    x86_64::instructions::interrupts::enable();
}
