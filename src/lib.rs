#![no_std]
#![feature(abi_x86_interrupt)]

pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod psf_parser;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}
