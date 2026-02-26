use crate::display::psf_parser::Psf2Font;
use bootloader_api::info::{BootInfo, FrameBufferInfo, PixelFormat};

pub struct FrameBuffer {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl FrameBuffer {
    pub fn new(boot_info: &'static mut BootInfo) -> Self {
        let fb = boot_info.framebuffer.as_mut().unwrap();
        let info = fb.info();
        let buffer = fb.buffer_mut();
        Self { buffer, info }
    }
    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        let (w, h) = (self.info.width, self.info.height);
        for y in 0..h {
            for x in 0..w {
                self.put_pixel(x, y, r, g, b);
            }
        }
    }
    pub fn put_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let offset =
            y * self.info.stride * self.info.bytes_per_pixel + x * self.info.bytes_per_pixel;
        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[offset] = r;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = b;
            }
            PixelFormat::Bgr => {
                self.buffer[offset] = b;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = r;
            }
            PixelFormat::U8 => {
                self.buffer[offset] = (r / 3) + (g / 3) + (b / 3);
            }
            _ => {}
        }
    }
    pub fn draw_glyph(
        &mut self,
        font: &Psf2Font,
        x: usize,
        y: usize,
        c: char,
        r: u8,
        g: u8,
        b: u8,
    ) {
        let glyph = font.get_glyph(c);
        let width = font.width() as usize;
        let height = font.height() as usize;
        let bytes_per_row = (width + 7) / 8;

        for row in 0..height {
            for col in 0..width {
                let byte = glyph[row * bytes_per_row + col / 8];
                if byte & (0x80 >> (col % 8)) != 0 {
                    self.put_pixel(x + col, y + row, r, g, b);
                }
            }
        }
    }
    pub fn width(&self) -> usize {
        self.info.width
    }
    pub fn height(&self) -> usize {
        self.info.height
    }
}
