use ngx_mod::{http, Module};

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

#[cfg(feature = "static-link")]
#[cfg(test)]
mod tests {
    use ngx_mod::{rt::core::ModuleType, ModuleMetadata};

    use super::*;

    #[test]
    fn module_metadata() {
        assert_eq!(M::module().ty(), ModuleType::Http);
        assert_eq!(M::commands().len(), 0);
    }
}
