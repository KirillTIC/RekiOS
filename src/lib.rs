#![no_std]
#![feature(abi_x86_interrupt)]

pub mod arch;
pub mod display;
pub mod drivers;

pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    arch::gdt::init();
    arch::interrupts::init_idt();
    display::shell::init(display::framebuffer::FrameBuffer::new(boot_info));
}
