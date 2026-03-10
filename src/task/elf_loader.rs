use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
    },
};

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const PT_LOAD: u32 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Header {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

pub fn load_elf(
    elf_data: &[u8],
    page_table: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    phys_offset: VirtAddr,
) -> u64 {
    assert!(elf_data.len() >= core::mem::size_of::<Elf64Header>(), "ELF too small");

    let header = unsafe { &*(elf_data.as_ptr() as *const Elf64Header) };
    assert_eq!(&header.e_ident[0..4], &ELF_MAGIC, "Not an ELF file");
    assert_eq!(header.e_ident[4], 2, "Not ELF64");

    let ph_offset = header.e_phoff as usize;
    let ph_size = header.e_phentsize as usize;
    let ph_num = header.e_phnum as usize;

    for i in 0..ph_num {
        let offset = ph_offset + i * ph_size;
        assert!(offset + ph_size <= elf_data.len(), "Program header out of bounds");

        let phdr = unsafe { &*(elf_data.as_ptr().add(offset) as *const Elf64Phdr) };
        if phdr.p_type != PT_LOAD {
            continue;
        }

        load_segment(elf_data, phdr, page_table, frame_allocator, phys_offset);
    }

    header.e_entry
}

fn load_segment(
    elf_data: &[u8],
    phdr: &Elf64Phdr,
    page_table: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    phys_offset: VirtAddr,
) {
    let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
    if phdr.p_flags & 0x2 != 0 {
        flags |= PageTableFlags::WRITABLE;
    }
    if phdr.p_flags & 0x1 == 0 {
        flags |= PageTableFlags::NO_EXECUTE;
    }

    let start_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(phdr.p_vaddr));
    let end_page: Page<Size4KiB> =
        Page::containing_address(VirtAddr::new(phdr.p_vaddr + phdr.p_memsz - 1));

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator
            .allocate_frame()
            .expect("Out of frames for ELF segment");

        unsafe {
            page_table
                .map_to(page, frame, flags, frame_allocator)
                .expect("Failed to map ELF segment")
                .flush();
        }
    }

    let seg_data = &elf_data[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize];
    let dest_ptr = (phys_offset.as_u64() + virt_to_phys(page_table, phdr.p_vaddr)) as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(seg_data.as_ptr(), dest_ptr, seg_data.len());
        let bss_size = (phdr.p_memsz - phdr.p_filesz) as usize;
        if bss_size > 0 {
            core::ptr::write_bytes(dest_ptr.add(seg_data.len()), 0, bss_size);
        }
    }
}

fn virt_to_phys(page_table: &OffsetPageTable, vaddr: u64) -> u64 {
    use x86_64::structures::paging::Translate;
    match page_table.translate(VirtAddr::new(vaddr)) {
        x86_64::structures::paging::mapper::TranslateResult::Mapped { frame, offset, .. } => {
            frame.start_address().as_u64() + offset
        }
        _ => panic!("ELF vaddr not mapped: {:#x}", vaddr),
    }
}
