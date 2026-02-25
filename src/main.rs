#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use reki_os::shell::SHELL;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    reki_os::init(boot_info);

    if let Some(shell) = SHELL.lock().as_mut() {
        shell.clear(0, 0, 0);
        shell.write_str("Hello, World!\n");
        shell.set_color(255, 0, 0);
        shell.write_str("Red\n");
    }

    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
