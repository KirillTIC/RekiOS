use crate::framebuffer::FrameBuffer;
use crate::psf_parser::Psf2Font;

pub struct Shell<'a> {
    fb: FrameBuffer,
    font: &'a Psf2Font,
    cursor_x: usize,
    cursor_y: usize,
    fg: (u8, u8, u8),
}

impl<'a> Shell<'a> {
    pub fn new(fb: FrameBuffer, font: &'a Psf2Font) -> Self {
        Self {
            fb,
            font,
            cursor_x: 0,
            cursor_y: 0,
            fg: (255, 255, 255),
        }
    }
    pub fn write_char(&mut self, c: char) {
        let char_width = self.font.width() as usize;
        let char_height = self.font.height() as usize;

        match c {
            '\n' => {
                self.cursor_x = 0;
                self.cursor_y += char_height;
            }
            _ => {
                if self.cursor_x + char_width >= self.fb.width() {
                    self.cursor_x = 0;
                    self.cursor_y += char_height;
                }

                let (r, g, b) = self.fg;
                self.fb
                    .draw_glyph(self.font, self.cursor_x, self.cursor_y, c, r, g, b);
                self.cursor_x += char_width;
            }
        }
    }
    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        self.fg = (r, g, b);
    }
    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        self.fb.clear(r, g, b);
    }
}
