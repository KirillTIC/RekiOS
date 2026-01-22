use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

//Edit print macros
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
#[macro_export]
macro_rules! print_color {
    ($color:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        use $crate::vga_buffer::{WRITER, Color, ColorCode};
        let mut writer = WRITER.lock();
        let original_color = writer.color_code;
        writer.color_code = ColorCode::new($color, Color::Black);

        writer.write_fmt(format_args!($($arg)*)).unwrap();

        writer.color_code = original_color;
    }};
}
#[macro_export]
macro_rules! println_color {
    ($color:expr, $($arg:tt)*) => {
        $crate::print_color!($color, "{}\n", format_args!($($arg)*));
    };
}
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::print_color!($crate::vga_buffer::Color::Red, "[ERROR] ");
        $crate::println!($($arg)*);
    };
}

//Create global mutable static writer
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut VgaBuffer) },
    });
}

//enumerate all colors in VGA
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

//desribing and implement color color code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);
impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

//desribing character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    ascii: u8,
    color_code: ColorCode,
}

//desribing VGA buffer
#[repr(transparent)]
struct VgaBuffer {
    chars: [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

//desribing writer
pub struct Writer {
    column_position: usize,
    pub color_code: ColorCode,
    buffer: &'static mut VgaBuffer,
}

impl Writer {
    //writing one byte (char)
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.next_line(),
            byte => {
                //check overflow
                if self.column_position >= BUFFER_WIDTH {
                    self.next_line()
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(Char {
                    ascii: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    //writing string
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn next_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let cc = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(cc);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank_char = Char {
            ascii: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank_char);
        }
    }
}
//implement macros !write with formating for our VGA writer
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
