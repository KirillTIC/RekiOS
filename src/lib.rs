#![no_std]
#![feature(abi_x86_interrupt)]

use bootloader_api::info::MemoryRegions;
use shell::shell::SHELL;
use spin::{Mutex, Once};
use x86_64::VirtAddr;
use x86_64::structures::paging::OffsetPageTable;

pub mod arch;
pub mod display;
pub mod drivers;
pub mod memory;
pub mod shell;
pub mod task;

static HELLO_PROGRAM: &[u8] = include_bytes!(
    "../programs/my_first_program/target/x86_64-unknown-none/release/my_first_program"
);
static CALC_PROGRAM: &[u8] =
    include_bytes!("../programs/calc/target/x86_64-unknown-none/release/calc");
static MEMORY_REGIONS: Once<&'static MemoryRegions> = Once::new();
static PHYS_OFFSET: Once<VirtAddr> = Once::new();
static FRAME_ALLOCATOR: Once<Mutex<memory::frame_allocator::BumpFrameAllocator>> = Once::new();

pub fn phys_offset() -> VirtAddr {
    *PHYS_OFFSET.get().expect("PHYS_OFFSET not initialized")
}

pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    MEMORY_REGIONS.call_once(|| &boot_info.memory_regions);
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    PHYS_OFFSET.call_once(|| phys_offset);
    let memory_regions = &boot_info.memory_regions;
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();

    unsafe {
        arch::pic::PICS.lock().initialize();
        arch::pic::PICS.lock().write_masks(0, 0);
    }
    arch::gdt::init();
    arch::interrupts::init();

    let sels = arch::gdt::selectors();
    arch::syscall::init_syscalls(
        sels.kernel_code,
        sels.kernel_data,
        sels.user_code,
        sels.user_data,
    );

    let mut page_table = unsafe { memory::page_table::init(phys_offset) };
    let mut frame_allocator =
        unsafe { memory::frame_allocator::BumpFrameAllocator::new(memory_regions) };

    memory::heap::init(&mut page_table, &mut frame_allocator);
    shell::shell::init(display::framebuffer::FrameBuffer::new(framebuffer));

    FRAME_ALLOCATOR.call_once(|| Mutex::new(frame_allocator));

    task::task::init();

    {
        let mut fa = FRAME_ALLOCATOR.get().unwrap().lock();
        let p4_frame =
            unsafe { memory::process_memory::create_user_page_table(phys_offset, &mut *fa) };

        let user_p4_virt = phys_offset + p4_frame.start_address().as_u64();
        let user_p4 =
            unsafe { &mut *(user_p4_virt.as_mut_ptr::<x86_64::structures::paging::PageTable>()) };
        let mut user_pt = unsafe { OffsetPageTable::new(user_p4, phys_offset) };

        let entry = task::elf_loader::load_elf(HELLO_PROGRAM, &mut user_pt, &mut *fa, phys_offset);

        let user_stack_base = VirtAddr::new(0x7FFF_FFFF_0000);
        let user_stack_size = 4096 * 8;
        unsafe {
            memory::process_memory::map_user_segment(
                &mut user_pt,
                &mut *fa,
                user_stack_base,
                user_stack_size,
            );
        }

        task::scheduler::SCHEDULER
            .lock()
            .add_user_task(entry, p4_frame);
    }

    task::scheduler::start();
    x86_64::instructions::interrupts::enable();
}

const ARGS_PAGE_ADDR: u64 = 0x7000_0000_0000;
const ARGS_PAGE_SIZE: usize = 4096;

pub fn spawn_user_program(elf_data: &[u8], args: &str) {
    let args_bytes = args.as_bytes();
    if args_bytes.len() > ARGS_PAGE_SIZE {
        println!(
            "spawn_user_program: args too long ({} > {} bytes)",
            args_bytes.len(),
            ARGS_PAGE_SIZE
        );
        return;
    }

    let phys_offset = phys_offset();
    let mut fa = FRAME_ALLOCATOR.get().unwrap().lock();

    let p4_frame = unsafe { memory::process_memory::create_user_page_table(phys_offset, &mut *fa) };

    let user_p4_virt = phys_offset + p4_frame.start_address().as_u64();
    let user_p4 =
        unsafe { &mut *(user_p4_virt.as_mut_ptr::<x86_64::structures::paging::PageTable>()) };
    let mut user_pt = unsafe { OffsetPageTable::new(user_p4, phys_offset) };

    let entry = task::elf_loader::load_elf(elf_data, &mut user_pt, &mut *fa, phys_offset);

    let user_stack_base = VirtAddr::new(0x7FFF_FFFF_0000);
    let user_stack_size = 4096 * 8;
    unsafe {
        memory::process_memory::map_user_segment(
            &mut user_pt,
            &mut *fa,
            user_stack_base,
            user_stack_size,
        );
    }

    let (rdi, rsi) = if !args_bytes.is_empty() {
        unsafe {
            memory::process_memory::map_user_segment(
                &mut user_pt,
                &mut *fa,
                VirtAddr::new(ARGS_PAGE_ADDR),
                ARGS_PAGE_SIZE as u64,
            );
        }
        let args_phys = task::elf_loader::virt_to_phys(&user_pt, ARGS_PAGE_ADDR);
        let dest = (phys_offset.as_u64() + args_phys) as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(args_bytes.as_ptr(), dest, args_bytes.len());
        }
        (ARGS_PAGE_ADDR, args_bytes.len() as u64)
    } else {
        (0u64, 0u64)
    };

    task::scheduler::SCHEDULER
        .lock()
        .add_user_task_with_args(entry, p4_frame, rdi, rsi);

    task::scheduler::yield_now();
}

pub fn spawn_calc(args: &str) {
    spawn_user_program(CALC_PROGRAM, args);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();

        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(mut guard) = SHELL.try_lock() {
                if let Some(shell) = guard.as_mut() {
                    shell.flush();
                }
            }
        });
    }
}
