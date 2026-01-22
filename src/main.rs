#![no_std]
#![no_main]

mod vga_buffer;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    print_ok!("OS was Init");
    panic!("TODO");

    //loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_panic!("{info}");
    loop {}
}
