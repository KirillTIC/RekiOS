extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Clone, Copy, PartialEq,Debug)]
pub enum ModuleState {
    Live,
    Failed,
}
pub struct LoadedModule {
    pub name: String,
    pub state: ModuleState,
    exit_fn: unsafe extern "C" fn(),
    _code: Vec<u8>,
}
// TODO
