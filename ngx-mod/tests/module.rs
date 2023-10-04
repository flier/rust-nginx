#![cfg(feature = "static-link")]

use ngx_mod::{
    http,
    rt::{core::Type, ffi, ngx_command},
    Module, ModuleMetadata,
};

#[derive(Module)]
#[module(name = foobar, type = http)]
struct M;

impl Module for M {}

impl http::Module for M {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
    type LocConf = ();
}

#[no_mangle]
static mut ngx_foobar_module_commands: [ffi::ngx_command_t; 1] = [ngx_command!()];

#[test]
fn module() {
    assert_eq!(M::module().ty(), Type::Http);
}
