use ngx_mod::{rt::ffi, Module};

#[derive(Module)]
struct M;

impl ngx_mod::Module for M {}

#[no_mangle]
static mut ngx_m_commands: [ffi::ngx_command_t; 0] = [];

fn main() {}
