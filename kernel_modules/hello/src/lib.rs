#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

mod reki {
    include!("../../reki.rs");
}

use reki::*;

#[no_mangle]
pub static module_name: [u8; 6] = *b"hello\0";

#[no_mangle]
pub extern "C" fn module_init() -> i32 {
    unsafe { printk(b"Hello from kernel module!\0".as_ptr()) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn module_exit() {
    printk(b"Goodbye from kernel module!\0".as_ptr());
}
