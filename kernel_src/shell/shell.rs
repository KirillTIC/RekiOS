extern crate alloc;
use crate::display::framebuffer::FrameBuffer;
use crate::display::psf_parser::Psf2Font;
use crate::drivers::keyboard;
use crate::shell::interpreter::{self, CommandResult};
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
    pub fb: FrameBuffer,
    fg: (u8, u8, u8),
    str_buffer: Vec<Vec<(char, (u8, u8, u8))>>,
    input_buffer: String,
    pub cursor_visible: bool,
}

impl Shell {
    pub fn new(fb: FrameBuffer) -> Self {
        let mut shell = Self {
            fb,
            fg: (255, 255, 255),
            str_buffer: vec![vec![]],
            input_buffer: String::from(""),
            cursor_visible: true,
        };
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

        if self.cursor_visible {
            let x = self.str_buffer.last().map_or(0, |l| l.len()) * char_width;
            let y = (self
                .str_buffer
                .len()
                .saturating_sub(1)
                .saturating_sub(start))
                * char_height;
            self.fb.draw_glyph(&FONT, x, y, '█', 255, 255, 255);
        }
    }

    pub fn cursor_update(&mut self) {
        self.cursor_visible = !self.cursor_visible;
        self.fb.dirty = true;
    }

    pub fn puts(&mut self, s: String) {
        for c in s.chars() {
            match c {
                '\x01' => self.set_color(255, 255, 255), // white
                '\x02' => self.set_color(0, 255, 0),     // green
                '\x03' => self.set_color(255, 0, 0),     // red
                '\x04' => self.set_color(0, 0, 255),     // blue
                '\x05' => self.set_color(255, 255, 0),   // yellow
                '\x06' => self.set_color(0, 255, 255),   // cyan
                '\x0B' => self.set_color(255, 0, 255),   // magenta
                '\x0C' => self.set_color(255, 165, 0),   // orange
                '\x0E' => self.set_color(192, 192, 192), // light gray
                '\x0F' => self.set_color(128, 128, 128), // dark gray
                _ => self.write_char(c),
            }
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
            if c != '\n' && c != 0x08 as char {
                self.write_char(c);
                self.input_buffer.push(c);
            } else if c == 0x08 as char {
                self.input_buffer.pop();
                if let Some(line) = self.str_buffer.last_mut() {
                    line.pop();
                }
                self.fb.dirty = true;
            } else {
                if self.input_buffer != "" {
                    self.write_char('\n');
                    match interpreter::read(&self.input_buffer) {
                        CommandResult::Output(s) => {
                            self.puts(s);
                            self.set_color(255, 255, 255);
                            self.write_char('\n');
                        }
                        CommandResult::Clear => {
                            self.clear(0, 0, 0);
                        }
                        CommandResult::Spawned => {}
                        CommandResult::None => {
                            self.write_char('\n');
                        }
                    }
                    self.input_buffer.clear();
                } else {
                    self.write_char('\n');
                }
            }
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
        self.puts(String::from(s));
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
                shell.set_color(255, 255, 255);
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
