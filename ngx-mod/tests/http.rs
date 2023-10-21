use ngx_mod::{http, rt::core::ModuleType, Module, ModuleMetadata};

#[derive(Module)]
#[module(type = http)]
struct M;

impl Module for M {}

impl http::Module for M {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
    type LocConf = ();
}

#[test]
fn module_metadata() {
    assert_eq!(M::module().ty(), ModuleType::Http);
    assert_eq!(M::commands().len(), 0);
}
