#![cfg(feature = "static-link")]

use ngx_mod::{rt::core::Type, stream, Module, ModuleMetadata};

#[derive(Module)]
#[module(name = foobar, type = stream)]
struct M;

impl Module for M {}

impl stream::Module for M {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
}

#[test]
fn core_module() {
    assert_eq!(M::module().ty(), Type::Http);
}
