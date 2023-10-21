use ngx_mod::{rt::core::ModuleType, stream, Module, ModuleMetadata};

#[derive(Module)]
#[module(type = stream)]
struct M;

impl Module for M {}

impl stream::Module for M {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
}

#[test]
fn core_module() {
    assert_eq!(M::module().ty(), ModuleType::Stream);
    assert_eq!(M::commands().len(), 0);
}
