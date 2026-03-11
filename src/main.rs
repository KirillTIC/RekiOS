#![no_std]
#![no_main]
extern crate alloc;

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig, entry_point};
use reki_os::shell::interpreter;
use reki_os::{hlt_loop, println};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    reki_os::init(boot_info);

    println!(
        "Reki OS | Copyright (c) 2026 czeplenok -- MIT License\n{}",
        interpreter::fetch()
    );

    hlt_loop();
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    println!("\x03KERNEL PANIC --- {}", _info);
    if let Some(mut guard) = reki_os::shell::shell::SHELL.try_lock() {
        if let Some(shell) = guard.as_mut() {
            shell.flush();
        }
    }
    loop {
        x86_64::instructions::hlt();
    }
}
