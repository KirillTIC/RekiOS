use crate::drivers::key::Key;
use alloc::{collections::VecDeque, string::String, vec::Vec};

pub const MAX_INPUT_CHARS: usize = 1024;
const HISTORY_LIMIT: usize = 32;

/// Cursor positions are character indexes, independent of UTF-8 and screen wrapping.
pub struct LineEditor {
    text: Vec<char>,
    cursor: usize,
    history: VecDeque<String>,
    history_index: Option<usize>,
    draft: Vec<char>,
    draft_cursor: usize,
}

impl LineEditor {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            cursor: 0,
            history: VecDeque::new(),
            history_index: None,
            draft: Vec::new(),
            draft_cursor: 0,
        }
    }
    pub fn chars(&self) -> &[char] {
        &self.text
    }
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    pub fn text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn edit(&mut self, key: Key) {
        match key {
            Key::Char(c) if !c.is_control() && self.text.len() < MAX_INPUT_CHARS => {
                self.text.insert(self.cursor, c);
                self.cursor += 1;
            }
            Key::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.text.remove(self.cursor);
            }
            Key::Delete if self.cursor < self.text.len() => {
                self.text.remove(self.cursor);
            }
            Key::Left => self.cursor = self.cursor.saturating_sub(1),
            Key::Right => self.cursor = (self.cursor + 1).min(self.text.len()),
            Key::Home => self.cursor = 0,
            Key::End => self.cursor = self.text.len(),
            Key::Up => self.previous(),
            Key::Down => self.next(),
            _ => {}
        }
    }

    fn previous(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let index = match self.history_index {
            None => {
                self.draft.clone_from(&self.text);
                self.draft_cursor = self.cursor;
                self.history.len() - 1
            }
            Some(index) => index.saturating_sub(1),
        };
        self.history_index = Some(index);
        self.text = self.history[index].chars().collect();
        self.cursor = self.text.len();
    }
    fn next(&mut self) {
        let Some(index) = self.history_index else {
            return;
        };
        if index + 1 < self.history.len() {
            self.history_index = Some(index + 1);
            self.text = self.history[index + 1].chars().collect();
            self.cursor = self.text.len();
        } else {
            self.history_index = None;
            self.text.clone_from(&self.draft);
            self.cursor = self.draft_cursor;
        }
    }
    pub fn submit(&mut self) -> String {
        let command = self.text();
        if !command.trim().is_empty() && self.history.back() != Some(&command) {
            if self.history.len() == HISTORY_LIMIT {
                self.history.pop_front();
            }
            self.history.push_back(command.clone());
        }
        self.text.clear();
        self.cursor = 0;
        self.history_index = None;
        self.draft.clear();
        command
    }

    /// Complete the token at the cursor. Ambiguous matches are returned for display.
    pub fn complete(
        &mut self,
        commands: &[&'static str],
        modules: &[&'static str],
    ) -> Vec<&'static str> {
        let start = self.text[..self.cursor]
            .iter()
            .rposition(|c| c.is_whitespace())
            .map_or(0, |i| i + 1);
        let end = self.text[self.cursor..]
            .iter()
            .position(|c| c.is_whitespace())
            .map_or(self.text.len(), |i| self.cursor + i);
        let before: String = self.text[..start].iter().collect();
        let words: Vec<&str> = before.split_whitespace().collect();
        let candidates = match words.as_slice() {
            [] => commands,
            ["insmod" | "rmmod"] => modules,
            _ => return Vec::new(),
        };
        let prefix: String = self.text[start..self.cursor].iter().collect();
        let matches: Vec<&'static str> = candidates
            .iter()
            .copied()
            .filter(|s| s.starts_with(&prefix))
            .collect();
        let Some(first) = matches.first() else {
            return matches;
        };
        let mut replacement: Vec<char> = first.chars().collect();
        for candidate in &matches[1..] {
            let common = replacement
                .iter()
                .copied()
                .zip(candidate.chars())
                .take_while(|(a, b)| a == b)
                .count();
            replacement.truncate(common);
        }
        if matches.len() > 1 && replacement.len() <= self.cursor - start {
            return matches;
        }
        if matches.len() == 1 && end == self.text.len() {
            replacement.push(' ');
        }
        if self.text.len() - (end - start) + replacement.len() <= MAX_INPUT_CHARS {
            self.cursor = start + replacement.len();
            self.text.splice(start..end, replacement);
        }
        Vec::new()
    }
}
