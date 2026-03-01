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
    None,
}

pub fn read(command: &String) -> CommandResult {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let cmd = parts[0];
    let args = &parts[1..];

    match cmd {
        "help" => CommandResult::Output(format!(
            "\x02Commands: help, echo, clear, fetch, reboot, halt (Only QEMU/Bochs)"
        )),
        "echo" => CommandResult::Output(args.join(" ")),
        "clear" => CommandResult::Clear,
        "fetch" => CommandResult::Output(fetch()),
        "reboot" => reboot(),
        "halt" => halt(),
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
pub fn get_cpu_name() -> [u8; 48] {
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
pub fn get_total_ram() -> u64 {
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

    format!(
        "\n \x02OS: RekiOS \n \x02OS version: 0.0.a \n \x03Shell: RekiSh \n \x03Shell vesrion: 0.p.a \n \x04CPU: {} \n \x04RAM: {}MiB Usage (with UEFI)\n",
        cpu_name, ram
    )
}
