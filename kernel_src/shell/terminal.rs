use super::{
    console::{Cell, Color, Console, GREEN, InputLayout, WHITE},
    editor::LineEditor,
};
use crate::display::{framebuffer::FrameBuffer, psf_parser::Psf2Font};
use lazy_static::lazy_static;

lazy_static! {
    static ref FONT: Psf2Font =
        Psf2Font::new(include_bytes!("../../assets/fonts/default8x16.psfu"));
}

pub struct Terminal {
    fb: FrameBuffer,
    console: Console,
    columns: usize,
    rows: usize,
    redraw: bool,
    cursor_dirty: bool,
    cursor_visible: bool,
}
impl Terminal {
    pub fn new(fb: FrameBuffer) -> Self {
        let columns = (fb.width() / FONT.width() as usize).max(1);
        let rows = (fb.height() / FONT.height() as usize).max(1);
        Self {
            fb,
            console: Console::new(columns),
            columns,
            rows,
            redraw: true,
            cursor_dirty: true,
            cursor_visible: true,
        }
    }
    pub fn write(&mut self, text: &str) {
        self.console.write(text);
        self.redraw = true;
    }
    pub fn set_color(&mut self, color: Color) {
        self.console.color = color;
    }
    pub fn clear(&mut self) {
        self.console.clear();
        self.input_changed();
    }
    pub fn commit_input(&mut self, text: &str) {
        self.console.commit_input(text);
        self.redraw = true;
    }
    pub fn input_changed(&mut self) {
        self.cursor_visible = true;
        self.redraw = true;
    }
    pub fn blink(&mut self) {
        self.cursor_visible = !self.cursor_visible;
        self.cursor_dirty = true;
    }
    pub fn flush(&mut self, editor: Option<&LineEditor>) {
        if !self.redraw && !self.cursor_dirty {
            return;
        }
        let base = self.console.input_row();
        let layout = editor
            .map(|e| InputLayout::new(base, e.chars().len(), e.cursor(), self.columns, self.rows));
        let start = layout.as_ref().map_or_else(
            || self.console.lines.len().saturating_sub(self.rows),
            |l| l.start_row,
        );
        if self.redraw {
            self.fb.clear(0, 0, 0);
            // Borrow output and framebuffer independently; no screen-sized temporary buffer.
            for (row, line) in self
                .console
                .lines
                .iter()
                .enumerate()
                .skip(start)
                .take(self.rows)
            {
                for (col, cell) in line.iter().enumerate() {
                    Self::draw_cell(&mut self.fb, row - start, col, *cell, false);
                }
            }
            if let Some(editor) = editor {
                for (i, ch) in ['>', ' '].iter().chain(editor.chars()).enumerate() {
                    let row = base + i / self.columns;
                    if row >= start && row < start + self.rows {
                        Self::draw_cell(
                            &mut self.fb,
                            row - start,
                            i % self.columns,
                            Cell {
                                ch: *ch,
                                color: if i < 2 { GREEN } else { WHITE },
                            },
                            false,
                        );
                    }
                }
            }
        }
        // Cursor blink repaints only its cell, preserving the character beneath it.
        if let (Some(editor), Some(layout)) = (editor, layout) {
            let ch = editor.chars().get(editor.cursor()).copied().unwrap_or(' ');
            Self::draw_cell(
                &mut self.fb,
                layout.cursor_row - start,
                layout.cursor_col,
                Cell { ch, color: WHITE },
                self.cursor_visible,
            );
        }
        self.fb.swap();
        self.redraw = false;
        self.cursor_dirty = false;
    }
    fn draw_cell(fb: &mut FrameBuffer, row: usize, col: usize, cell: Cell, cursor: bool) {
        let w = FONT.width() as usize;
        let h = FONT.height() as usize;
        let x = col * w;
        let y = row * h;
        let bg = if cursor { 255 } else { 0 };
        for dy in 0..h {
            for dx in 0..w {
                fb.put_pixel(x + dx, y + dy, bg, bg, bg);
            }
        }
        let (r, g, b) = if cursor { (0, 0, 0) } else { cell.color };
        fb.draw_glyph(&FONT, x, y, cell.ch, r, g, b);
    }
}
