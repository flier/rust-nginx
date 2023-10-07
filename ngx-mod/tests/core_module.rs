#![cfg(feature = "static-link")]

use ngx_mod::{core, rt::core::Type, Module, ModuleMetadata};

#[derive(Module)]
#[module(name = foobar, type = core)]
struct M;

impl Module for M {}

impl core::Module for M {
    type Error = ();
    type Conf = ();
}

#[test]
fn core_module() {
    assert_eq!(M::module().ty(), Type::Http);
}
