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
        rt::{
            core::{ModuleType, Str},
            FromRawRef,
        },
        ModuleMetadata,
    };

    use super::*;

    #[test]
    fn module_metadata() {
        assert_eq!(M::module().ty(), ModuleType::Core);
        assert_eq!(M::commands().len(), 0);
    }

    #[test]
    fn module_ref() {
        assert_eq!(M.as_ref().ty(), ModuleType::Core);
        assert_eq!(M.ty(), ModuleType::Core);
        assert_eq!(M.commands().len(), 0);
    }

    #[test]
    fn module_ctx() {
        assert_eq!(
            unsafe {
                Str::from_raw(&ngx_m_module_ctx.name as *const _ as *mut _)
                    .unwrap()
                    .to_str()
                    .unwrap()
            },
            "ngx_m_module"
        );
    }
}
