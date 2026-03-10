use core::arch::asm;
use x86_64::VirtAddr;

pub unsafe fn jump_to_userspace(
    entry_point: VirtAddr,
    user_stack_top: VirtAddr,
    user_code_selector: u16,
    user_data_selector: u16,
) -> ! {
    unsafe {
        asm!(
            "push {user_ds}",
            "push {user_rsp}",
            "push 0x200",
            "push {user_cs}",
            "push {entry}",
            "iretq",

            user_ds = in(reg) user_data_selector as u64,
            user_rsp = in(reg) user_stack_top.as_u64(),
            user_cs = in(reg) user_code_selector as u64,
            entry = in(reg) entry_point.as_u64(),
            options(noreturn)
        );
    }
}
