use ngx_mod::{core, Module};

#[derive(Module)]
#[module(type = core)]
struct M;

impl Module for M {}

impl core::Module for M {
    type Error = ();
    type Conf = ();
}

#[cfg(feature = "static-link")]
#[cfg(test)]
mod tests {
    use ngx_mod::{
        rt::core::{Str, Type},
        ModuleMetadata,
    };

    use super::*;

    #[test]
    fn module_metadata() {
        assert_eq!(M::module().ty(), Type::Core);
        assert_eq!(M::commands().len(), 0);
    }

    #[test]
    fn module_ref() {
        assert_eq!(M.as_ref().ty(), Type::Core);
        assert_eq!(M.ty(), Type::Core);
        assert_eq!(M.commands().len(), 0);
    }

    #[test]
    fn module_ctx() {
        assert_eq!(
            unsafe {
                Str::from_raw(ngx_m_module_ctx.name)
                    .unwrap()
                    .to_str()
                    .unwrap()
            },
            "ngx_m_module"
        );
    }
}
