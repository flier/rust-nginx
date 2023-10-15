#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use std::collections::HashMap;

use ngx_mod::{
    rt::{
        core::{time::MSec, CmdRef, Code, ConfRef, Str},
        http::script::ComplexValueRef,
        native_setter,
    },
    Conf,
};

#[derive(Clone, Default, Conf)]
#[conf(http::main)]
struct MainConf {
    #[directive(args(1))]
    endpoint: Str,
    #[directive(args(1))]
    interval: MSec,
    #[directive(args(1))]
    batch_size: usize,
    #[directive(args(1))]
    batch_count: usize,
    #[directive(args(1))]
    service_name: Str,
}

#[derive(Clone, Default, Conf)]
#[conf(http::main, http::server, http::location)]
struct LocConf<'a> {
    #[directive(args(1), set = complex_value)]
    trace: Option<&'a ComplexValueRef>,
    #[directive(args(1), set = enum_values, values = propagation::TYPES)]
    trace_ctx: propagation::Type,
    #[directive(args(1), set = complex_value)]
    span_name: Option<&'a ComplexValueRef>,
    #[directive(args(1), set = add_span_attr)]
    span_attrs: HashMap<Str, &'a ComplexValueRef>,
}

mod propagation {
    use ngx_mod::rt::ngx_enum_values;

    bitflags::bitflags! {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct Type: usize {
            const EXTRACT = 0x0001;
            const INJECT = 0x0002;
        }
    }

    ngx_enum_values! {
        pub enum TYPES {
            "ignore" => Type::empty().bits(),
            "extract" => Type::EXTRACT.bits(),
            "inject" => Type::INJECT.bits(),
            "propagate" => Type::EXTRACT.bits() | Type::INJECT.bits()
        }
    }
}

#[native_setter(log = cf)]
fn add_span_attr(cf: &ConfRef, cmd: &CmdRef, conf: &mut LocConf) -> Result<(), Code> {
    Ok(())
}
