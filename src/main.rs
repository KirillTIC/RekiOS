#![no_std]
#![no_main]

use reki_os::{print_ok, print_panic, println_color, vga_buffer};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    reki_os::init();

    test_print();

    x86_64::instructions::interrupts::int3();

    panic!("TODO");
    //loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_panic!("{info}");

    loop {}
}

fn test_print() {
    print_ok!("OS was init");
    println_color!(vga_buffer::Color::Pink, "Just test of color output");
}
