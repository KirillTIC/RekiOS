use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        crate::arch::pic::PICS
            .lock()
            .notify_end_of_interrupt(crate::arch::pic::InterruptIndex::Timer.as_u8());
    }
}
