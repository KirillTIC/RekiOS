#![no_std]
#![no_main]

static FONT_DATA: &[u8] = include_bytes!("../assets/fonts/default8x16.psfu");

use bootloader_api::{BootInfo, entry_point};
use reki_os::framebuffer::FrameBuffer;
use reki_os::psf_parser::Psf2Font;
use reki_os::shell::Shell;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let fb = FrameBuffer::new(boot_info);
    let font = Psf2Font::new(FONT_DATA);
    let mut shell = Shell::new(fb, &font);

    reki_os::init();

    shell.clear(0, 0, 0);
    shell.write_str("Hello, World!\n");
    shell.set_color(255, 0, 0);
    shell.write_str("Red\n");

    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
