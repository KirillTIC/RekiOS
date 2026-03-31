pub fn idle_task() -> ! {
    x86_64::instructions::interrupts::enable();
    loop {
        super::scheduler::cleanup_dead_and_yield();
        x86_64::instructions::hlt();
    }
}
