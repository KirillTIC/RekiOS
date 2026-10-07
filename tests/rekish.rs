// Host-side tests for the actual editor/console code, without booting the kernel.
extern crate alloc;
#[path = "../kernel_src/drivers/key.rs"]
pub mod key;
mod drivers {
    pub use crate::key;
}
#[path = "../kernel_src/shell/console.rs"]
mod console;
#[path = "../kernel_src/shell/editor.rs"]
mod editor;
use console::{Console, InputLayout};
use editor::LineEditor;
use key::Key;

fn type_text(editor: &mut LineEditor, text: &str) {
    for ch in text.chars() {
        editor.edit(Key::Char(ch));
    }
}
const COMMANDS: &[&str] = &[
    "help", "echo", "clear", "calc", "cat", "ls", "lsmod", "insmod", "rmmod",
];
const MODULES: &[&str] = &["hello", "ahci", "ext2"];

#[test]
fn editing_and_deletion_respect_character_boundaries() {
    let mut e = LineEditor::new();
    e.edit(Key::Backspace);
    e.edit(Key::Delete);
    e.edit(Key::Left);
    type_text(&mut e, "aЖc");
    e.edit(Key::Left);
    e.edit(Key::Backspace);
    type_text(&mut e, "б");
    assert_eq!(e.text(), "aбc");
    assert_eq!(e.cursor(), 2);
    e.edit(Key::Delete);
    assert_eq!(e.text(), "aб");
    e.edit(Key::Home);
    e.edit(Key::Backspace);
    type_text(&mut e, "界");
    e.edit(Key::End);
    e.edit(Key::Right);
    type_text(&mut e, "!");
    assert_eq!(e.submit(), "界aб!");
    assert_eq!(e.cursor(), 0);
    assert!(e.chars().is_empty());
}
#[test]
fn history_restores_draft_and_its_cursor_without_mutating_saved_commands() {
    let mut e = LineEditor::new();
    for s in ["first", "second"] {
        type_text(&mut e, s);
        e.submit();
    }
    type_text(&mut e, "draft");
    e.edit(Key::Left);
    e.edit(Key::Up);
    assert_eq!(e.text(), "second");
    e.edit(Key::Backspace);
    e.edit(Key::Up);
    assert_eq!(e.text(), "first");
    e.edit(Key::Up);
    assert_eq!(e.text(), "first");
    e.edit(Key::Down);
    assert_eq!(e.text(), "second");
    e.edit(Key::Down);
    assert_eq!(e.text(), "draft");
    assert_eq!(e.cursor(), 4);
    e.edit(Key::Down);
    assert_eq!(e.cursor(), 4);
}
#[test]
fn history_is_bounded_and_skips_blank_and_consecutive_duplicates() {
    let mut e = LineEditor::new();
    for i in 0..40 {
        type_text(&mut e, &i.to_string());
        e.submit();
    }
    type_text(&mut e, "39");
    e.submit();
    type_text(&mut e, "   ");
    e.submit();
    for _ in 0..50 {
        e.edit(Key::Up);
    }
    assert_eq!(e.text(), "8");
    for _ in 0..31 {
        e.edit(Key::Down);
    }
    assert_eq!(e.text(), "39");
    e.edit(Key::Down);
    assert_eq!(e.text(), "");
}
#[test]
fn completion_handles_unique_ambiguous_and_module_tokens() {
    let mut e = LineEditor::new();
    type_text(&mut e, "he");
    assert!(e.complete(COMMANDS, MODULES).is_empty());
    assert_eq!(e.text(), "help ");
    e.submit();
    type_text(&mut e, "c");
    assert!(!e.complete(COMMANDS, MODULES).is_empty());
    assert_eq!(e.text(), "c");
    e.submit();
    type_text(&mut e, "ca");
    assert_eq!(e.complete(COMMANDS, MODULES), vec!["calc", "cat"]);
    e.submit();
    type_text(&mut e, "  insmod ex");
    e.complete(COMMANDS, MODULES);
    assert_eq!(e.text(), "  insmod ext2 ");
    e.submit();
    type_text(&mut e, "echo ex");
    e.complete(COMMANDS, MODULES);
    assert_eq!(e.text(), "echo ex");
}
#[test]
fn completion_extends_shared_prefix_and_preserves_arguments() {
    let mut e = LineEditor::new();
    type_text(&mut e, "l");
    e.complete(COMMANDS, MODULES);
    assert_eq!(e.text(), "ls");
    assert_eq!(e.complete(COMMANDS, MODULES), vec!["ls", "lsmod"]);
    e.submit();
    type_text(&mut e, "heXXX argument");
    e.edit(Key::Home);
    e.edit(Key::Right);
    e.edit(Key::Right);
    e.complete(COMMANDS, MODULES);
    assert_eq!(e.text(), "help argument");
    assert_eq!(e.cursor(), 4);
}
#[test]
fn input_limit_is_enforced_for_typing_and_completion() {
    let mut e = LineEditor::new();
    type_text(&mut e, &"x".repeat(editor::MAX_INPUT_CHARS + 100));
    assert_eq!(e.chars().len(), editor::MAX_INPUT_CHARS);
    e.submit();
    type_text(
        &mut e,
        &format!("h {}", "x".repeat(editor::MAX_INPUT_CHARS - 2)),
    );
    e.edit(Key::Home);
    e.edit(Key::Right);
    e.complete(COMMANDS, MODULES);
    assert_eq!(e.chars().len(), editor::MAX_INPUT_CHARS);
    assert!(e.text().starts_with("h "));
}
#[test]
fn output_cannot_change_draft_or_be_erased_by_backspace() {
    let mut e = LineEditor::new();
    let mut c = Console::new(5);
    type_text(&mut e, "123456789");
    e.edit(Key::Left);
    c.write("background output\n");
    let output = c.lines.clone();
    for _ in 0..20 {
        e.edit(Key::Backspace);
    }
    assert_eq!(e.text(), "9");
    assert_eq!(c.lines, output);
    c.commit_input(&e.submit());
    assert_eq!(
        c.lines[c.lines.len() - 2]
            .iter()
            .map(|cell| cell.ch)
            .collect::<String>(),
        "> 9"
    );
}
#[test]
fn output_is_bounded_and_preserves_color_and_newlines() {
    let mut c = Console::new(4);
    c.write("\x02abcde\n");
    assert_eq!(c.lines.len(), 3);
    assert_eq!(c.lines[1][0].ch, 'e');
    assert_eq!(c.lines[1][0].color, console::GREEN);
    for _ in 0..2000 {
        c.write("row\n");
    }
    assert_eq!(c.lines.len(), console::SCROLLBACK_LINES);
    c.clear();
    assert_eq!(c.input_row(), 0);
    c.write("x");
    assert_eq!(c.input_row(), 1);
    c.commit_input("help");
    assert_eq!(c.lines[0][0].ch, 'x');
}
#[test]
fn cursor_stays_inside_viewport_at_wrap_and_scrollback_boundaries() {
    for columns in [1, 2, 8, 160] {
        for rows in [1, 3, 50] {
            for base in [0, 5, 511, 512] {
                for len in [0, 1, 6, 158, 159, 160, 1024] {
                    for cursor in 0..=len {
                        let l = InputLayout::new(base, len, cursor, columns, rows);
                        assert!(l.cursor_col < columns);
                        assert!(l.cursor_row >= l.start_row);
                        assert!(l.cursor_row < l.start_row + rows);
                    }
                }
            }
        }
    }
    let l = InputLayout::new(0, 158, 158, 160, 50);
    assert_eq!((l.cursor_row, l.cursor_col), (1, 0));
}
#[test]
fn interrupt_queue_is_fifo_bounded_and_reusable_after_wrapping() {
    let mut q = key::KeyQueue::new();
    for _ in 0..300 {
        q.push(Key::Left);
    }
    for _ in 0..256 {
        assert_eq!(q.pop(), Some(Key::Left));
    }
    assert_eq!(q.pop(), None);
    for _ in 0..400 {
        q.push(Key::Char('x'));
        q.push(Key::Enter);
        assert_eq!(q.pop(), Some(Key::Char('x')));
        assert_eq!(q.pop(), Some(Key::Enter));
    }
}
