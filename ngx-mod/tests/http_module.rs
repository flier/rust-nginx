#![cfg(feature = "static-link")]

use ngx_mod::{http, rt::core::Type, Module, ModuleMetadata};

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

#[test]
fn http_module() {
    assert_eq!(M::module().ty(), Type::Http);
}
