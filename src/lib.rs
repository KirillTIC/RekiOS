#![no_std]
#![feature(abi_x86_interrupt)]

#[macro_use]
pub mod vga_buffer;
pub mod gdt;
pub mod interrupts;

pub fn init() {
    interrupts::init_idt();
    gdt::init();
}
