pub fn idle_task() -> ! {
    loop {
        super::scheduler::yield_now();
        x86_64::instructions::hlt();
    }
}
