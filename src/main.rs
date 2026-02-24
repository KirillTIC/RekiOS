#![no_std]
#![no_main]

mod vga_buffer;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_print();
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
