extern crate alloc;
use crate::arch::pic;
use alloc::collections::VecDeque;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
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
            match key {
                DecodedKey::Unicode(character) => push_key(character),
                DecodedKey::RawKey(_) => {}
            }
        }
    }

    unsafe {
        pic::PICS
            .lock()
            .notify_end_of_interrupt(pic::InterruptIndex::Keyboard.as_u8());
    }
}

lazy_static! {
    static ref KEY_QUEUE: Mutex<VecDeque<char>> = Mutex::new(VecDeque::new());
}
pub fn push_key(c: char) {
    KEY_QUEUE.lock().push_back(c);
}
pub fn pop_key() -> Option<char> {
    KEY_QUEUE.lock().pop_front()
}
