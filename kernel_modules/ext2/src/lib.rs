#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { loop {} }

mod reki { include!("../../reki.rs"); }
use reki::*;

#[repr(C, packed)]
struct Superblock {
    s_inodes_count:      u32,
    s_blocks_count:      u32,
    _r1:                 u32,
    s_free_blocks_count: u32,
    s_free_inodes_count: u32,
    s_first_data_block:  u32,
    s_log_block_size:    u32,
    _r2:                 u32,
    s_blocks_per_group:  u32,
    _r3:                 u32,
    s_inodes_per_group:  u32,
    _r4:                 [u32; 3],
    s_magic:             u16,
    _r5:                 [u8; 30],
    s_inode_size:        u16,
}

#[repr(C, packed)]
struct Bgd {
    bg_block_bitmap: u32,
    bg_inode_bitmap: u32,
    bg_inode_table:  u32,
    bg_free_blocks:  u16,
    bg_free_inodes:  u16,
    bg_used_dirs:    u16,
    _pad:            [u8; 14],
}

#[repr(C, packed)]
struct Inode {
    i_mode:   u16,
    i_uid:    u16,
    i_size:   u32,
    i_atime:  u32,
    i_ctime:  u32,
    i_mtime:  u32,
    i_dtime:  u32,
    i_gid:    u16,
    i_links:  u16,
    i_blocks: u32,
    i_flags:  u32,
    _osd1:    u32,
    i_block:  [u32; 15],
    _rest:    [u32; 7],
}

type ReadFn = extern "C" fn(u64, u16, u64) -> i32;

struct Ext2State {
    read_fn:           ReadFn,
    block_size:        u32,
    sectors_per_block: u16,
    inodes_per_group:  u32,
    inode_size:        u16,
    inode_table:       u32,
}

static mut FS: Option<Ext2State> = None;
static mut CWD_INO: u32 = 2;
static mut CWD_PATH: [u8; 256] = [0u8; 256];

unsafe fn read_block(fs: &Ext2State, block_nr: u32, buf_phys: u64) -> bool {
    let lba = block_nr as u64 * fs.sectors_per_block as u64;
    (fs.read_fn)(lba, fs.sectors_per_block, buf_phys) == 0
}

unsafe fn read_inode(fs: &Ext2State, ino: u32, buf_phys: u64) -> bool {
    let ipg = fs.inodes_per_group;
    let idx = if ipg > 0 { (ino - 1) % ipg } else { return false; };
    let off_bytes = idx as u64 * fs.inode_size as u64;
    let bs = fs.block_size as u64;
    let (off_block, off_in_block) = if bs > 0 {
        ((off_bytes / bs) as u32, (off_bytes % bs) as usize)
    } else {
        return false;
    };
    if !read_block(fs, fs.inode_table + off_block, buf_phys) { return false; }
    if off_in_block != 0 {
        let virt = phys_to_virt(buf_phys) as *mut u8;
        core::ptr::copy(virt.add(off_in_block), virt, fs.inode_size as usize);
    }
    true
}

fn build_abs_path(cwd: &[u8], new_path: &[u8], out: &mut [u8; 256]) -> usize {
    let mut tmp = [0u8; 512];
    let mut tmp_len = 0usize;

    let is_absolute = new_path.first() == Some(&b'/');
    let base: &[u8] = if is_absolute { b"/" } else { cwd };
    for &b in base {
        if tmp_len < 511 { unsafe { *tmp.get_unchecked_mut(tmp_len) = b; } tmp_len += 1; }
    }
    let last = if tmp_len > 0 { unsafe { *tmp.get_unchecked(tmp_len - 1) } } else { 0 };
    if last != b'/' {
        if tmp_len < 511 { unsafe { *tmp.get_unchecked_mut(tmp_len) = b'/'; } tmp_len += 1; }
    }
    for &b in new_path {
        if tmp_len < 511 { unsafe { *tmp.get_unchecked_mut(tmp_len) = b; } tmp_len += 1; }
    }

    unsafe { *out.get_unchecked_mut(0) = b'/'; }
    let mut out_len = 1usize;
    let mut i = 0usize;
    while i < tmp_len {
        if unsafe { *tmp.get_unchecked(i) } == b'/' { i += 1; continue; }
        let seg_start = i;
        while i < tmp_len && unsafe { *tmp.get_unchecked(i) } != b'/' { i += 1; }
        let seg = unsafe { core::slice::from_raw_parts(tmp.as_ptr().add(seg_start), i - seg_start) };
        if seg == b"." {
        } else if seg == b".." {
            if out_len > 1 {
                while out_len > 1 && unsafe { *out.get_unchecked(out_len - 1) } != b'/' {
                    out_len -= 1;
                }
                if out_len > 1 { out_len -= 1; }
            }
        } else {
            if out_len > 1 && out_len < 255 {
                unsafe { *out.get_unchecked_mut(out_len) = b'/'; }
                out_len += 1;
            }
            for &b in seg {
                if out_len < 255 { unsafe { *out.get_unchecked_mut(out_len) = b; } out_len += 1; }
            }
        }
    }
    if out_len == 0 { unsafe { *out.get_unchecked_mut(0) = b'/'; } out_len = 1; }
    unsafe { *out.get_unchecked_mut(out_len) = 0; }
    out_len
}

unsafe fn kputs(s: &[u8]) {
    let mut tmp = [0u8; 256];
    let n = s.len().min(255);
    tmp[..n].copy_from_slice(&s[..n]);
    tmp[n] = 0;
    printk(tmp.as_ptr());
}

unsafe fn lookup_path(fs: &Ext2State, path: *const u8, inode_buf: u64, start_ino: u32) -> Option<u32> {
    let mut path_len = 0usize;
    while *path.add(path_len) != 0 && path_len < 256 { path_len += 1; }
    let path_bytes = core::slice::from_raw_parts(path, path_len);

    let is_abs = path_bytes.first() == Some(&b'/');
    let mut cur_ino: u32 = if is_abs { 2 } else { start_ino };

    let parts_start = if is_abs { &path_bytes[1..] } else { path_bytes };
    if parts_start.is_empty() { return Some(cur_ino); }

    let mut remaining = parts_start;
    loop {
        let seg_end = remaining.iter().position(|&b| b == b'/').unwrap_or(remaining.len());
        let seg = &remaining[..seg_end];
        if seg.is_empty() { break; }

        if !read_inode(fs, cur_ino, inode_buf) { return None; }
        let inode = &*(phys_to_virt(inode_buf) as *const Inode);
        if inode.i_mode & 0xF000 != 0x4000 { return None; }

        let mut found_ino: Option<u32> = None;
        'outer: for bi in 0..12u32 {
            let blk = { let b = inode.i_block; b[bi as usize] };
            if blk == 0 { break; }
            let dir_buf = alloc_phys_frame();
            if !read_block(fs, blk, dir_buf) { break; }
            let virt = phys_to_virt(dir_buf) as *const u8;
            let mut off = 0usize;
            while off < fs.block_size as usize {
                let ent_ino  = *(virt.add(off) as *const u32);
                let rec_len  = *(virt.add(off + 4) as *const u16) as usize;
                let name_len = *(virt.add(off + 6) as *const u8) as usize;
                if ent_ino != 0 && name_len == seg.len() {
                    let name = core::slice::from_raw_parts(virt.add(off + 8), name_len);
                    if name == seg {
                        found_ino = Some(ent_ino);
                        break 'outer;
                    }
                }
                if rec_len == 0 { break; }
                off += rec_len;
            }
        }

        cur_ino = found_ino?;
        if seg_end >= remaining.len() { break; }
        remaining = &remaining[seg_end + 1..];
    }
    Some(cur_ino)
}

#[no_mangle]
pub extern "C" fn ext2_ls(path: *const u8) {
    let fs = unsafe { match FS.as_ref() { Some(f) => f, None => return } };
    let inode_buf = unsafe { alloc_phys_frame() };
    let start_ino = unsafe { CWD_INO };
    let target_ino = unsafe { lookup_path(fs, path, inode_buf, start_ino) };
    let ino = match target_ino { Some(i) => i, None => {
        unsafe { printk(b"ext2: path not found\0".as_ptr()); }
        return;
    }};

    unsafe {
        if !read_inode(fs, ino, inode_buf) {
            printk(b"ext2: failed to read inode\0".as_ptr());
            return;
        }
        let inode = &*(phys_to_virt(inode_buf) as *const Inode);
        if inode.i_mode & 0xF000 != 0x4000 {
            printk(b"ext2: not a directory\0".as_ptr());
            return;
        }
        let dir_buf = alloc_phys_frame();
        for bi in 0..12u32 {
            let blk = { let b = inode.i_block; b[bi as usize] };
            if blk == 0 { break; }
            if !read_block(fs, blk, dir_buf) { break; }
            let virt = phys_to_virt(dir_buf) as *const u8;
            let mut off = 0usize;
            while off < fs.block_size as usize {
                let ent_ino  = *(virt.add(off) as *const u32);
                let rec_len  = *(virt.add(off + 4) as *const u16) as usize;
                let name_len = *(virt.add(off + 6) as *const u8) as usize;
                let ftype    = *(virt.add(off + 7) as *const u8);
                if ent_ino != 0 && name_len > 0 {
                    let prefix: &[u8] = if ftype == 2 { b"dir  " } else { b"file " };
                    kputs(prefix);
                    let name = core::slice::from_raw_parts(virt.add(off + 8), name_len);
                    kputs(name);
                    printk(b"\n\0".as_ptr());
                }
                if rec_len == 0 { break; }
                off += rec_len;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn ext2_cat(path: *const u8, out_buf: *mut u8, max_len: usize) -> i64 {
    let fs = unsafe { match FS.as_ref() { Some(f) => f, None => return -1 } };
    let inode_buf = unsafe { alloc_phys_frame() };
    let start_ino = unsafe { CWD_INO };
    let target_ino = unsafe { lookup_path(fs, path, inode_buf, start_ino) };
    let ino = match target_ino { Some(i) => i, None => return -1 };

    unsafe {
        if !read_inode(fs, ino, inode_buf) { return -1; }
        let inode = &*(phys_to_virt(inode_buf) as *const Inode);
        if inode.i_mode & 0xF000 == 0x4000 { return -1; }

        let file_size = { inode.i_size } as usize;
        let mut written = 0usize;
        let data_buf = alloc_phys_frame();

        for bi in 0..12usize {
            if written >= file_size || written >= max_len { break; }
            let blk = { let b = inode.i_block; b[bi] };
            if blk == 0 { break; }
            if !read_block(fs, blk, data_buf) { return -1; }

            let chunk = (file_size - written).min(fs.block_size as usize).min(max_len - written);
            let src = phys_to_virt(data_buf) as *const u8;
            core::ptr::copy_nonoverlapping(src, out_buf.add(written), chunk);
            written += chunk;
        }
        written as i64
    }
}

#[no_mangle]
pub extern "C" fn ext2_cd(path: *const u8) -> i32 {
    let fs = unsafe { match FS.as_ref() { Some(f) => f, None => return -1 } };
    let inode_buf = unsafe { alloc_phys_frame() };
    let start_ino = unsafe { CWD_INO };
    let target_ino = unsafe { lookup_path(fs, path, inode_buf, start_ino) };
    let ino = match target_ino {
        Some(i) => i,
        None => {
            unsafe { printk(b"cd: not found\0".as_ptr()); }
            return -1;
        }
    };
    unsafe {
        if !read_inode(fs, ino, inode_buf) {
            printk(b"cd: failed to read inode\0".as_ptr());
            return -1;
        }
        let inode = &*(phys_to_virt(inode_buf) as *const Inode);
        if inode.i_mode & 0xF000 != 0x4000 {
            printk(b"cd: not a directory\0".as_ptr());
            return -1;
        }
        CWD_INO = ino;
        let path_len = { let mut l = 0usize; while *path.add(l) != 0 { l += 1; } l };
        let path_bytes = core::slice::from_raw_parts(path, path_len);
        let cwd_len = { let mut l = 0usize; while l < 255 && *CWD_PATH.get_unchecked(l) != 0 { l += 1; } l };
        let mut cwd_snap = [0u8; 256];
        core::ptr::copy_nonoverlapping(CWD_PATH.as_ptr(), cwd_snap.as_mut_ptr(), cwd_len);
        let cwd_slice = core::slice::from_raw_parts(cwd_snap.as_ptr(), cwd_len);
        let out_ptr = CWD_PATH.as_mut_ptr();
        let mut out_arr = [0u8; 256];
        build_abs_path(cwd_slice, path_bytes, &mut out_arr);
        core::ptr::copy_nonoverlapping(out_arr.as_ptr(), out_ptr, 256);
    }
    0
}

#[no_mangle]
pub extern "C" fn ext2_cwd(buf: *mut u8, max: usize) -> usize {
    if buf.is_null() || max == 0 { return 0; }
    unsafe {
        let len = { let mut l = 0usize; while l < 255 && *CWD_PATH.get_unchecked(l) != 0 { l += 1; } l };
        let copy_len = len.min(max - 1);
        core::ptr::copy_nonoverlapping(CWD_PATH.as_ptr(), buf, copy_len);
        *buf.add(copy_len) = 0;
        copy_len
    }
}

#[no_mangle]
pub static module_name: [u8; 5] = *b"ext2\0";

#[no_mangle]
pub extern "C" fn module_init() -> i32 {
    let fn_ptr = unsafe { ksym_lookup(b"ahci_read_sectors\0".as_ptr()) };
    if fn_ptr == 0 {
        unsafe { printk(b"ext2: need ahci module (insmod ahci first)\0".as_ptr()); }
        return -1;
    }
    let read_fn: ReadFn = unsafe { core::mem::transmute(fn_ptr) };

    let sb_phys = unsafe { alloc_phys_frame() };
    if read_fn(2, 2, sb_phys) != 0 {
        unsafe { printk(b"ext2: failed to read superblock\0".as_ptr()); }
        return -1;
    }
    let sb: &Superblock = unsafe { &*(phys_to_virt(sb_phys) as *const Superblock) };
    if { sb.s_magic } != 0xEF53 {
        unsafe { printk(b"ext2: bad magic (not ext2?)\0".as_ptr()); }
        return -1;
    }

    let log_bs = { sb.s_log_block_size };
    let block_size: u32 = 1024u32 << log_bs;
    let sectors_per_block = (block_size / 512) as u16;
    let inodes_per_group  = { sb.s_inodes_per_group };
    let inode_size = {
        let s = sb.s_inode_size;
        if s == 0 { 128u16 } else { s }
    };
    let first_data_block = { sb.s_first_data_block };

    let bgdt_block = first_data_block + 1;
    let bgd_phys = unsafe { alloc_phys_frame() };
    let fs_tmp = Ext2State { read_fn, block_size, sectors_per_block, inodes_per_group, inode_size, inode_table: 0 };
    if !unsafe { read_block(&fs_tmp, bgdt_block, bgd_phys) } {
        unsafe { printk(b"ext2: failed to read BGD\0".as_ptr()); }
        return -1;
    }
    let bgd0: &Bgd = unsafe { &*(phys_to_virt(bgd_phys) as *const Bgd) };
    let inode_table = { bgd0.bg_inode_table };

    unsafe {
        FS = Some(Ext2State { read_fn, block_size, sectors_per_block, inodes_per_group, inode_size, inode_table });
        CWD_INO = 2;
        CWD_PATH[0] = b'/';
        CWD_PATH[1] = 0;

        ksym_export(b"ext2_ls\0".as_ptr(),  ext2_ls  as *const () as usize);
        ksym_export(b"ext2_cat\0".as_ptr(), ext2_cat as *const () as usize);
        ksym_export(b"ext2_cd\0".as_ptr(),  ext2_cd  as *const () as usize);
        ksym_export(b"ext2_cwd\0".as_ptr(), ext2_cwd as *const () as usize);
    }

    unsafe { printk(b"ext2: mounted, root dir:\0".as_ptr()); }
    ext2_ls(b"/\0".as_ptr());

    0
}

#[no_mangle]
pub unsafe extern "C" fn module_exit() {
    FS = None;
    CWD_INO = 2;
    CWD_PATH[0] = b'/';
    CWD_PATH[1] = 0;
    printk(b"ext2: unmounted\0".as_ptr());
}
