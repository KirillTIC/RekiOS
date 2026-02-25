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
                self.buffer[offset] = r;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = b;
            }
            PixelFormat::U8 => {
                self.buffer[offset] = (r / 3) + (g / 3) + (b / 3);
            }
            _ => {}
        }
    }
}
