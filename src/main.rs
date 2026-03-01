#![no_std]
#![no_main]
extern crate alloc;

use reki_os::{clear, hlt_loop, println_colored};

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig, entry_point};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    reki_os::init(boot_info);

    clear!();
    println_colored!(0, 255, 0, "OS INIT\n");

    hlt_loop();
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println_colored!(255, 0, 0, "KERNEL PANIC --- {}", _info);
    hlt_loop();
}
