use ngx_mod::{stream, Module};

#[derive(Module)]
#[module(type = stream)]
struct M;

impl Module for M {}

impl stream::Module for M {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
}

#[cfg(feature = "static-link")]
#[cfg(test)]
mod tests {
    use ngx_mod::{rt::core::Type, ModuleMetadata};

    use super::*;

    #[test]
    fn core_module() {
        assert_eq!(M::module().ty(), Type::Stream);
        assert_eq!(M::commands().len(), 0);
    }
}
