extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use bootloader_api::info::MemoryRegionKind;
use core::arch::x86_64::__cpuid;
use core::str;
use x86_64::instructions::port::Port;

pub enum CommandResult {
    Output(String),
    Clear,
    Spawned,
    None,
}

pub fn read(command: &String) -> CommandResult {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let cmd = parts[0];
    let args = &parts[1..];

    match cmd {
        "help" => CommandResult::Output(format!(
            "\x02Commands: help, echo, clear, fetch, calc, reboot, halt (Only QEMU/Bochs), lsmod, insmod, rmmod, ls, cat, cd, pwd"
        )),
        "echo" => CommandResult::Output(args.join(" ")),
        "clear" => CommandResult::Clear,
        "fetch" => CommandResult::Output(fetch()),
        "calc" => {
            let expr = args.join(" ");
            crate::spawn_calc(&expr);
            CommandResult::Spawned
        }
        "reboot" => reboot(),
        "halt" => halt(),
        "lsmod" => lsmod(),
        "insmod" => insmod(args),
        "rmmod"  => rmmod(args),
        "ls"     => ls_cmd(args),
        "cat"    => cat_cmd(args),
        "cd"     => cd_cmd(args),
        "pwd"    => pwd_cmd(),
        _ => CommandResult::Output(format!("\x03Unkown command: {}", cmd)),
    }
}

pub fn reboot() -> ! {
    unsafe {
        let mut port: Port<u8> = Port::new(0x64);
        port.write(0xFE);
    }
    loop {}
}
pub fn halt() -> ! {
    unsafe {
        let mut port: Port<u16> = Port::new(0x604);
        port.write(0x2000);
    }
    loop {}
}
fn get_cpu_name() -> [u8; 48] {
    let mut name = [0u8; 48];
    let r1 = __cpuid(0x80000002);
    let r2 = __cpuid(0x80000003);
    let r3 = __cpuid(0x80000004);

    let chunks = [
        r1.eax, r1.ebx, r1.ecx, r1.edx, r2.eax, r2.ebx, r2.ecx, r2.edx, r3.eax, r3.ebx, r3.ecx,
        r3.edx,
    ];

    for (i, &chunk) in chunks.iter().enumerate() {
        let bytes = chunk.to_le_bytes();
        name[i * 4..i * 4 + 4].copy_from_slice(&bytes);
    }
    name
}
fn get_total_ram() -> u64 {
    crate::MEMORY_REGIONS
        .get()
        .map(|regions| {
            regions
                .iter()
                .filter(|r| r.kind == MemoryRegionKind::Usable)
                .map(|r| r.end - r.start)
                .sum()
        })
        .unwrap_or(0)
}
pub fn fetch() -> String {
    let ram = get_total_ram() / 1024 / 1024;
    let name_bytes = get_cpu_name();
    let cpu_name = str::from_utf8(&name_bytes)
        .unwrap_or("Unknown")
        .trim_matches(|c: char| c == '\0' || c.is_whitespace());
    let logo = r#"
          __
  ___  _//   \
_/   \/__|_   \
/  __//_/==\_  | ___
/ | / /|// == \ \  /
|  | |\|| //_\ | |_/
\  \ \\ / \_/| || \
\___/\\| _  ///___\
\__|\_\=//_// _\_|
   \___\_____/
     \____/
    "#;

    format!(
        "\n\x03{} \n \x01▼\x02▼\x03▼\x04▼\x05▼\x06▼\x0B▼\x0C▼\x0E▼\x0F▼\x01▼\x02▼\x03▼\x04▼\x05▼\x06▼\x0B▼\x0C▼\x0E▼\x0F▼ \n \x05┌OS: RekiOS \n └OS version: 0.0.a \n \x0C┌Shell: RekiSh \n └Shell vesrion: 0.p.a \n \x06┌CPU: {} \n └RAM: {}MiB Usage (with UEFI) \n \x01▲\x02▲\x03▲\x04▲\x05▲\x06▲\x0B▲\x0C▲\x0E▲\x0F▲\x01▲\x02▲\x03▲\x04▲\x05▲\x06▲\x0B▲\x0C▲\x0E▲\x0F▲ \n",
        logo, cpu_name, ram
    )
}
fn lsmod() -> CommandResult {
    let mods = crate::module::loader::list();
    if mods.is_empty() {
        return CommandResult::Output(format!("\x03(No modules loaded)"));
    }
    let mut out = format!("\x02{:<24} {}\x01\n", "Name", "State");
    for (name, state) in &mods {
        let st = match state {
            crate::module::loader::ModuleState::Live => "\x02live\x01",
            crate::module::loader::ModuleState::Failed => "\x03failed\x01",
        };
        out += &format!("   {:<24} {}\n", name, st);
    }
    CommandResult::Output(out)
}
fn insmod(args: &[&str]) -> CommandResult {
    let Some(name) = args.first() else {
        return CommandResult::Output(format!("\x03insmod <name>"));
    };
    let data: &[u8] = match *name {
        "hello" => include_bytes!("../../kernel_modules/hello/hello.km"),
        "ahci"  => include_bytes!("../../kernel_modules/ahci/ahci.km"),
        "ext2"  => include_bytes!("../../kernel_modules/ext2/ext2.km"),
        _ => return CommandResult::Output(format!("\x03insmod: unknown module '{}'", name)),
    };
    match crate::module::loader::insmod(data) {
        Ok(()) => CommandResult::Output(format!("\x02insmod: loaded '{}'", name)),
        Err(e) => CommandResult::Output(format!("\x03insmod: {}", e)),
    }
}
fn ls_cmd(args: &[&str]) -> CommandResult {
    let path = args.first().copied().unwrap_or(".");
    let fn_ptr = crate::module::symbols::lookup("ext2_ls").unwrap_or(0);
    if fn_ptr == 0 {
        return CommandResult::Output(format!("\x03ext2 not loaded (insmod ext2)"));
    }
    let f: extern "C" fn(*const u8) = unsafe { core::mem::transmute(fn_ptr) };
    let mut buf = alloc::vec![0u8; path.len() + 1];
    buf[..path.len()].copy_from_slice(path.as_bytes());
    f(buf.as_ptr());
    CommandResult::None
}
fn cat_cmd(args: &[&str]) -> CommandResult {
    let Some(path) = args.first() else {
        return CommandResult::Output(format!("\x03cat <path>"));
    };
    let fn_ptr = crate::module::symbols::lookup("ext2_cat").unwrap_or(0);
    if fn_ptr == 0 {
        return CommandResult::Output(format!("\x03ext2 not loaded (insmod ext2)"));
    }
    let f: extern "C" fn(*const u8, *mut u8, usize) -> i64 =
        unsafe { core::mem::transmute(fn_ptr) };
    let mut path_buf = alloc::vec![0u8; path.len() + 1];
    path_buf[..path.len()].copy_from_slice(path.as_bytes());
    let mut out = alloc::vec![0u8; 65536];
    let n = f(path_buf.as_ptr(), out.as_mut_ptr(), out.len());
    if n < 0 {
        return CommandResult::Output(format!("\x03cat: file not found or read error"));
    }
    match String::from_utf8(out[..n as usize].to_vec()) {
        Ok(s)  => CommandResult::Output(s),
        Err(_) => CommandResult::Output(format!("\x03cat: binary file ({} bytes)", n)),
    }
}
fn cd_cmd(args: &[&str]) -> CommandResult {
    let path = args.first().copied().unwrap_or("/");
    let fn_ptr = crate::module::symbols::lookup("ext2_cd").unwrap_or(0);
    if fn_ptr == 0 {
        return CommandResult::Output(format!("\x03ext2 not loaded (insmod ext2)"));
    }
    let f: extern "C" fn(*const u8) -> i32 = unsafe { core::mem::transmute(fn_ptr) };
    let mut buf = alloc::vec![0u8; path.len() + 1];
    buf[..path.len()].copy_from_slice(path.as_bytes());
    if f(buf.as_ptr()) != 0 {
        return CommandResult::Output(format!("\x03cd: {}: no such directory", path));
    }
    CommandResult::None
}
fn pwd_cmd() -> CommandResult {
    let fn_ptr = crate::module::symbols::lookup("ext2_cwd").unwrap_or(0);
    if fn_ptr == 0 {
        return CommandResult::Output(format!("\x03ext2 not loaded (insmod ext2)"));
    }
    let f: extern "C" fn(*mut u8, usize) -> usize = unsafe { core::mem::transmute(fn_ptr) };
    let mut buf = alloc::vec![0u8; 256];
    let n = f(buf.as_mut_ptr(), 256);
    match core::str::from_utf8(&buf[..n]) {
        Ok(s) => CommandResult::Output(format!("{}", s)),
        Err(_) => CommandResult::Output(format!("\x03pwd: encoding error")),
    }
}
fn rmmod(args: &[&str]) -> CommandResult {
    let Some(name) = args.first() else {
        return CommandResult::Output(format!("\x03rmmod <name>"));
    };
    match crate::module::loader::unload(name) {
        Ok(()) => CommandResult::Output(format!("\x02Unloaded: {}", name)),
        Err(msg) => CommandResult::Output(format!("\x03{}", msg)),
    }
}
