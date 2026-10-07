use super::{
    editor::LineEditor,
    interpreter::{COMMANDS, CommandResult, MODULES},
    terminal::Terminal,
};
use crate::display::framebuffer::FrameBuffer;
use crate::drivers::{key::Key, keyboard};
use alloc::string::String;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SHELL: Mutex<Option<Shell>> = Mutex::new(None);
}

/// Coordinates editing and command results. Rendering belongs to Terminal.
pub struct Shell {
    terminal: Terminal,
    editor: LineEditor,
    accepting_input: bool,
}
impl Shell {
    pub fn new(fb: FrameBuffer) -> Self {
        let mut shell = Self {
            terminal: Terminal::new(fb),
            editor: LineEditor::new(),
            accepting_input: true,
        };
        shell.flush();
        shell
    }
    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        self.terminal.set_color((r, g, b));
    }
    pub fn clear(&mut self) {
        self.terminal.clear();
    }
    pub fn cursor_update(&mut self) {
        if self.accepting_input {
            self.terminal.blink();
        }
    }
    pub fn poll_command(&mut self) -> Option<String> {
        if !self.accepting_input {
            return None;
        }
        while let Some(key) = keyboard::pop_key() {
            match key {
                Key::Enter => {
                    let command = self.editor.submit();
                    self.terminal.commit_input(&command);
                    self.terminal.input_changed();
                    if !command.trim().is_empty() {
                        self.accepting_input = false;
                        return Some(command);
                    }
                }
                Key::Tab => {
                    let matches = self.editor.complete(COMMANDS, MODULES);
                    if !matches.is_empty() {
                        self.terminal.commit_input(&self.editor.text());
                        self.terminal.write(&matches.join("  "));
                        self.terminal.write("\n");
                    }
                    self.terminal.input_changed();
                }
                key => {
                    self.editor.edit(key);
                    self.terminal.input_changed();
                }
            }
        }
        None
    }
    pub fn handle_result(&mut self, result: CommandResult) {
        match result {
            CommandResult::Output(s) => {
                self.terminal.write(&s);
                self.terminal.write("\n");
            }
            CommandResult::Clear => self.terminal.clear(),
            CommandResult::Spawned | CommandResult::None => {}
        }
        self.terminal.set_color((255, 255, 255));
        self.accepting_input = true;
        self.terminal.input_changed();
    }
    pub fn flush(&mut self) {
        self.terminal
            .flush(self.accepting_input.then_some(&self.editor));
    }
}
impl fmt::Write for Shell {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.terminal.write(text);
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
                shell.clear();
            }
        })
    };
}

pub fn init(fb: FrameBuffer) {
    *SHELL.lock() = Some(Shell::new(fb));
}
