use super::key::{Key, KeyQueue};
use crate::arch::pic;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1, layouts};
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            let event = match key {
                DecodedKey::Unicode('\n' | '\r') => Some(Key::Enter),
                DecodedKey::Unicode('\x08') => Some(Key::Backspace),
                DecodedKey::Unicode('\x7f') => Some(Key::Delete),
                DecodedKey::Unicode('\t') => Some(Key::Tab),
                DecodedKey::Unicode(c) if !c.is_control() => Some(Key::Char(c)),
                DecodedKey::RawKey(code) => match code {
                    KeyCode::ArrowLeft => Some(Key::Left),
                    KeyCode::ArrowRight => Some(Key::Right),
                    KeyCode::ArrowUp => Some(Key::Up),
                    KeyCode::ArrowDown => Some(Key::Down),
                    KeyCode::Home => Some(Key::Home),
                    KeyCode::End => Some(Key::End),
                    KeyCode::Delete => Some(Key::Delete),
                    _ => None,
                },
                _ => None,
            };
            if let Some(event) = event {
                KEY_QUEUE.lock().push(event);
            }
        }
    }

    unsafe {
        pic::PICS
            .lock()
            .notify_end_of_interrupt(pic::InterruptIndex::Keyboard.as_u8());
    }
}

static KEY_QUEUE: Mutex<KeyQueue> = Mutex::new(KeyQueue::new());

pub fn pop_key() -> Option<Key> {
    x86_64::instructions::interrupts::without_interrupts(|| KEY_QUEUE.lock().pop())
}
