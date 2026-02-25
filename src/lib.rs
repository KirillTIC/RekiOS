#![no_std]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupts;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}
