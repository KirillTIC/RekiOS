use crate::shell::shell::SHELL;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::InterruptStackFrame;

static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let ticks = TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    if ticks % 10 == 0 {
        if let Some(mut guard) = SHELL.try_lock() {
            if let Some(shell) = guard.as_mut() {
                shell.cursor_update();
            }
        }
    }
    unsafe {
        crate::arch::pic::PICS
            .lock()
            .notify_end_of_interrupt(crate::arch::pic::InterruptIndex::Timer.as_u8());
    }
    crate::task::scheduler::tick();
}
