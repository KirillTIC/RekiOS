pub fn idle_task() -> ! {
    x86_64::instructions::interrupts::enable();
    loop {
        super::scheduler::yield_now();
        x86_64::instructions::hlt();
    }
}
