use crate::framebuffer::FrameBuffer;
use crate::psf_parser::Psf2Font;
use lazy_static::lazy_static;
use spin::Mutex;

static FONT_DATA: &[u8] = include_bytes!("../assets/fonts/default8x16.psfu");

lazy_static! {
    static ref FONT: Psf2Font = Psf2Font::new(FONT_DATA);
    pub static ref SHELL: Mutex<Option<Shell>> = Mutex::new(None);
}

pub struct Shell {
    fb: FrameBuffer,
    cursor_x: usize,
    cursor_y: usize,
    fg: (u8, u8, u8),
}

impl Shell {
    pub fn new(fb: FrameBuffer) -> Self {
        Self {
            fb,
            cursor_x: 0,
            cursor_y: 0,
            fg: (255, 255, 255),
        }
    }
    pub fn write_char(&mut self, c: char) {
        let char_width = FONT.width() as usize;
        let char_height = FONT.height() as usize;

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
                    .draw_glyph(&FONT, self.cursor_x, self.cursor_y, c, r, g, b);
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
        self.cursor_x = 0;
        self.cursor_y = 0;
    }
}

pub fn init(fb: FrameBuffer) {
    *SHELL.lock() = Some(Shell::new(fb));
}
