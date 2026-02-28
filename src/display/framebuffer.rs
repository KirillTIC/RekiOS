extern crate alloc;
use crate::display::psf_parser::Psf2Font;
use alloc::vec;
use alloc::vec::Vec;
use bootloader_api::info::{FrameBufferInfo, PixelFormat};

pub struct FrameBuffer {
    buffer: &'static mut [u8],
    back_buffer: Vec<u8>,
    info: FrameBufferInfo,
    pub dirty: bool,
}

impl FrameBuffer {
    pub fn new(fb: &'static mut bootloader_api::info::FrameBuffer) -> Self {
        let info = fb.info();
        let buffer = fb.buffer_mut();
        let back_buffer = vec![0u8; buffer.len()];
        let dirty = false;
        Self {
            buffer,
            back_buffer,
            info,
            dirty,
        }
    }
    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        if r == 0 && g == 0 && b == 0 {
            self.back_buffer.fill(0);
            return;
        }
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
                self.back_buffer[offset] = r;
                self.back_buffer[offset + 1] = g;
                self.back_buffer[offset + 2] = b;
            }
            PixelFormat::Bgr => {
                self.back_buffer[offset] = b;
                self.back_buffer[offset + 1] = g;
                self.back_buffer[offset + 2] = r;
            }
            PixelFormat::U8 => {
                self.back_buffer[offset] = (r / 3) + (g / 3) + (b / 3);
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
    pub fn swap(&mut self) {
        self.buffer.copy_from_slice(&self.back_buffer.as_slice());
        self.dirty = false;
    }
    pub fn width(&self) -> usize {
        self.info.width
    }
    pub fn height(&self) -> usize {
        self.info.height
    }
}
