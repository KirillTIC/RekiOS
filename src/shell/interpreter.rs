extern crate alloc;
use alloc::format;
use alloc::string::String;

pub fn read(command: &String) -> Option<String> {
    Some(format!("UNKNOWN COMMAND: {}", command))
}
