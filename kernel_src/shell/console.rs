use alloc::{collections::VecDeque, vec::Vec};

pub type Color = (u8, u8, u8);
pub const WHITE: Color = (255, 255, 255);
pub const GREEN: Color = (0, 255, 0);
pub const SCROLLBACK_LINES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub color: Color,
}

/// Bounded output history. The editable prompt is deliberately not stored here.
pub struct Console {
    pub lines: VecDeque<Vec<Cell>>,
    pub color: Color,
    columns: usize,
}

impl Console {
    pub fn new(columns: usize) -> Self {
        let mut console = Self {
            lines: VecDeque::new(),
            color: WHITE,
            columns: columns.max(1),
        };
        console.newline();
        console
    }
    fn newline(&mut self) {
        if self.lines.len() == SCROLLBACK_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(Vec::new());
    }
    pub fn fresh_line(&mut self) {
        if !self.lines.back().unwrap().is_empty() {
            self.newline();
        }
    }
    pub fn clear(&mut self) {
        self.lines.clear();
        self.newline();
        self.color = WHITE;
    }
    pub fn write_char(&mut self, ch: char) {
        if ch == '\n' {
            self.newline();
            return;
        }
        if ch == '\t' {
            let spaces = 4 - self.lines.back().unwrap().len() % 4;
            for _ in 0..spaces {
                self.write_char(' ');
            }
            return;
        }
        if ch.is_control() {
            return;
        }
        if self.lines.back().unwrap().len() == self.columns {
            self.newline();
        }
        self.lines.back_mut().unwrap().push(Cell {
            ch,
            color: self.color,
        });
    }
    pub fn write(&mut self, text: &str) {
        for ch in text.chars() {
            match ch {
                '\x01' => self.color = WHITE,
                '\x02' => self.color = GREEN,
                '\x03' => self.color = (255, 0, 0),
                '\x04' => self.color = (0, 0, 255),
                '\x05' => self.color = (255, 255, 0),
                '\x06' => self.color = (0, 255, 255),
                '\x0B' => self.color = (255, 0, 255),
                '\x0C' => self.color = (255, 165, 0),
                '\x0E' => self.color = (192, 192, 192),
                '\x0F' => self.color = (128, 128, 128),
                _ => self.write_char(ch),
            }
        }
    }
    pub fn commit_input(&mut self, text: &str) {
        self.fresh_line();
        self.color = GREEN;
        self.write("> ");
        self.color = WHITE;
        self.write(text);
        self.write_char('\n');
    }
    pub fn input_row(&self) -> usize {
        self.lines.len() - usize::from(self.lines.back().unwrap().is_empty())
    }
}

/// Keep the editing cursor visible, including an extra row at the right edge.
#[derive(Debug)]
pub struct InputLayout {
    pub start_row: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
}
impl InputLayout {
    pub fn new(base: usize, input_len: usize, cursor: usize, columns: usize, rows: usize) -> Self {
        let columns = columns.max(1);
        let cursor_row = base + (2 + cursor) / columns;
        let total_rows = base + (2 + input_len) / columns + 1;
        Self {
            start_row: total_rows.saturating_sub(rows.max(1)).min(cursor_row),
            cursor_row,
            cursor_col: (2 + cursor) % columns,
        }
    }
}
