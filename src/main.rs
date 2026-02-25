#![no_std]
#![no_main]

static FONT_DATA: &[u8] = include_bytes!("../assets/fonts/default8x16.psfu");

use bootloader_api::{BootInfo, entry_point};
use reki_os::framebuffer::FrameBuffer;
use reki_os::psf_parser::Psf2Font;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut fb = FrameBuffer::new(boot_info);
    let font = Psf2Font::new(FONT_DATA);

    reki_os::init();

    fb.clear(0, 0, 0);
    fb.draw_glyph(&font, 0, 0, 'H', 255, 255, 255);
    fb.draw_glyph(&font, 8, 0, 'e', 255, 255, 255);
    fb.draw_glyph(&font, 16, 0, 'l', 255, 255, 255);
    fb.draw_glyph(&font, 24, 0, 'l', 255, 255, 255);
    fb.draw_glyph(&font, 32, 0, 'o', 255, 255, 255);
    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
