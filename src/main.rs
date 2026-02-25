#![no_std]
#![no_main]

use reki_os::{clear, print_colored, println};

use bootloader_api::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    reki_os::init(boot_info);

    clear!();
    println!("Hello, World!");
    print_colored!(0, 255, 0, "OS INIT\n");

    panic!("TODO")

    //loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print_colored!(255, 0, 0, "KERNEL PANIC --- {}", _info);
    loop {}
}
