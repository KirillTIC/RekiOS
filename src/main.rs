#![no_std]
#![no_main]

static FONT: &[u8] = include_bytes!("../assets/fonts/default8x16.psfu");

use bootloader_api::{BootInfo, entry_point};
use reki_os::framebuffer::FrameBuffer;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut fb = FrameBuffer::new(boot_info);
    reki_os::init();
    fb.clear(0, 0, 0);
    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
