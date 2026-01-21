const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

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
struct ColorCode(u8);
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
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
    chars: [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

//desribing writer
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
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

                self.buffer.chars[row][col] = Char {
                    ascii: byte,
                    color_code,
                };
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

    pub fn next_line(&mut self) {
        /*TO
         * DO*/
    }
}

//TEMP
pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut VgaBuffer) },
    };

    writer.write_string("Hello, World!");
}
