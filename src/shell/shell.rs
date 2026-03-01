extern crate alloc;
use crate::display::framebuffer::FrameBuffer;
use crate::display::psf_parser::Psf2Font;
use crate::drivers::keyboard;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

static FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/default8x16.psfu");

lazy_static! {
    static ref FONT: Psf2Font = Psf2Font::new(FONT_DATA);
    pub static ref SHELL: Mutex<Option<Shell>> = Mutex::new(None);
}

pub struct Shell {
    fb: FrameBuffer,
    fg: (u8, u8, u8),
    str_buffer: Vec<Vec<(char, (u8, u8, u8))>>,
    input_buffer: String,
}

impl Shell {
    pub fn new(fb: FrameBuffer) -> Self {
        let mut shell = Self {
            fb,
            fg: (255, 255, 255),
            str_buffer: vec![vec![]],
            input_buffer: String::from(""),
        };
        shell.str_buffer[0].push(('H', (255, 255, 255)));
        shell.render();
        shell.fb.swap();
        shell
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => {
                self.str_buffer.push(vec![]);
            }
            _ => {
                let max_chars = self.fb.width() / FONT.width() as usize;
                if let Some(last) = self.str_buffer.last() {
                    if last.len() >= max_chars {
                        self.str_buffer.push(vec![]);
                    }
                }
                if let Some(line) = self.str_buffer.last_mut() {
                    line.push((c, self.fg));
                }
            }
        }
        self.fb.dirty = true;
    }

    fn render(&mut self) {
        self.fb.clear(0, 0, 0);
        let char_width = FONT.width() as usize;
        let char_height = FONT.height() as usize;
        let visible = self.fb.height() / char_height;
        let start = if self.str_buffer.len() > visible {
            self.str_buffer.len() - visible
        } else {
            0
        };

        for (i, line) in self.str_buffer.iter().skip(start).enumerate() {
            let cursor_y = i * char_height;
            for (j, (c, (r, g, b))) in line.iter().enumerate() {
                let cursor_x = j * char_width;
                self.fb
                    .draw_glyph(&FONT, cursor_x, cursor_y, *c, *r, *g, *b);
            }
        }
    }

    pub fn puts(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        self.fg = (r, g, b);
    }

    pub fn flush(&mut self) {
        if self.fb.dirty {
            self.render();
            self.fb.swap();
        }
        if let Some(c) = keyboard::pop_key() {
            self.write_char(c);
            self.input_buffer.push(c);
        }
    }

    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        self.fb.clear(r, g, b);
        self.str_buffer = vec![vec![]];
        self.fb.dirty = true;
    }
}

impl fmt::Write for Shell {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        x86_64::instructions::interrupts::without_interrupts(|| {
            use core::fmt::Write;
            if let Some(shell) = $crate::shell::shell::SHELL.lock().as_mut() {
                write!(shell, $($arg)*).unwrap();
            }
        })
    };
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
#[macro_export]
macro_rules! print_colored {
    ($r:expr, $g:expr, $b:expr, $($arg:tt)*) => {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(shell) = $crate::shell::shell::SHELL.lock().as_mut() {
                shell.set_color($r, $g, $b);
                use core::fmt::Write;
                write!(shell, $($arg)*).unwrap();
                shell.set_color(255, 255, 255);
            }
        })
    };
}

#[macro_export]
macro_rules! println_colored {
    ($r:expr, $g:expr, $b:expr, $($arg:tt)*) => {
        $crate::print_colored!($r, $g, $b, "{}\n", format_args!($($arg)*))
    };
}
#[macro_export]
macro_rules! clear {
    () => {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(shell) = $crate::shell::shell::SHELL.lock().as_mut() {
                shell.clear(0, 0, 0);
            }
        })
    };
}

pub fn init(fb: FrameBuffer) {
    *SHELL.lock() = Some(Shell::new(fb));
}
