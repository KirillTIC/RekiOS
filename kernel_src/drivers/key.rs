/// Logical input events; the line editor never needs hardware scancodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Tab,
}

/// Fixed storage avoids allocating from the keyboard interrupt handler.
pub struct KeyQueue {
    keys: [Option<Key>; 256],
    head: usize,
    len: usize,
}

impl KeyQueue {
    pub const fn new() -> Self {
        Self {
            keys: [None; 256],
            head: 0,
            len: 0,
        }
    }
    pub fn push(&mut self, key: Key) {
        if self.len < self.keys.len() {
            self.keys[(self.head + self.len) % self.keys.len()] = Some(key);
            self.len += 1;
        }
    }
    pub fn pop(&mut self) -> Option<Key> {
        if self.len == 0 {
            return None;
        }
        let key = self.keys[self.head].take();
        self.head = (self.head + 1) % self.keys.len();
        self.len -= 1;
        key
    }
}
