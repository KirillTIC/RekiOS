#![no_std]
#![feature(abi_x86_interrupt)]

pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod psf_parser;
pub mod shell;

pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    gdt::init();
    interrupts::init_idt();
    shell::init(framebuffer::FrameBuffer::new(boot_info));
}
