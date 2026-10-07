# RekiSh

## Editing

- Left / Right move the cursor by one character.
- Home / End move to the beginning / end of the command.
- Backspace deletes the preceding character; Delete deletes the character under the cursor.
- Up / Down browse the last 32 commands. Going down past the newest entry restores the draft and its cursor position. Consecutive duplicate commands and blank lines are not saved.
- Tab completes the command name, or a module name after `insmod` / `rmmod`. A unique match is completed; multiple matches extend the common prefix, then a further Tab lists alternatives. File path completion is not implemented.
- Enter submits the command. Empty and whitespace-only lines are safe.

Input is limited to 1024 characters. Output retains at most 512 wrapped rows. There is no scrollback navigation yet; the viewport follows the editing cursor. Keyboard decoding currently uses the US layout.

Program output is kept separate from the editable prompt, so it cannot erase the draft or become a target for Backspace. Programs are still scheduled by the existing kernel; this change does not introduce foreground process management, pipes or job control.

## Structure

- `drivers/key.rs`: logical keys and a bounded, allocation-free interrupt queue.
- `drivers/keyboard.rs`: PS/2 decoding into logical key events.
- `shell/editor.rs`: character-based editing, history and completion; independent of graphics and interrupts.
- `shell/console.rs`: bounded colored output and cursor/viewport layout.
- `shell/terminal.rs`: framebuffer rendering and cursor blinking.
- `shell/shell.rs`: input dispatch, submission and presentation of command results.
- `shell/interpreter.rs`: command execution, command names and help.

The editing prompt is rendered as an overlay, never appended to the output history until submission. Commands execute outside the shell lock. Cursor blinking repaints its cell; other changes redraw the visible content. Framebuffer presentation currently copies the full back buffer.

## Checks

Run the editor/console regression tests on the host:

```sh
./tests/rekish.sh
```

Build and run on macOS:

```sh
./build-macos.sh
./run-macos.sh
```

Close a running QEMU instance before rebuilding its disk image or launching it again.

Manual smoke checks:

1. Press Backspace repeatedly at an empty prompt, then submit a line containing only spaces.
2. Type `he`, press Tab and Enter; help should appear.
3. Type `echo ac`, press Left, insert `b`, press End and Enter; output should be `abc`.
4. Browse history with Up, then Down; an unfinished draft and its cursor should return.
5. Type `c` and press Tab; candidates should appear without changing the draft.
6. Type `insmod ex` and press Tab; it should become `insmod ext2 `.
7. Edit a command longer than one screen row, including Backspace across a wrap.
8. Run `calc 3+4` and start typing the next command; the result should appear above the intact draft.
