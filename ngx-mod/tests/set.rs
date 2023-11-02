use std::fs;

use ngx_mod::{
    rt::core::{conf, ArrayRef, Bufs, KeyValue, MSec, Sec, Str},
    Conf,
};

#[derive(Clone, Debug, Conf)]
#[conf(default = unset)]
pub struct Conf<'a> {
    #[directive(args(1), set = flag)]
    pub flag: conf::Flag,
    #[directive(args(1), set = str)]
    pub str: Str,
    #[directive(args(1), set = str_array)]
    pub str_array: Option<&'a ArrayRef<Str>>,
    #[directive(args(1), set = keyval)]
    pub keyval: Option<&'a ArrayRef<KeyValue>>,
    #[directive(args(1), set = num)]
    pub num: isize,
    #[directive(args(1), set = size)]
    pub size: usize,
    #[directive(args(1), set = off)]
    pub off: i64,
    #[directive(args(1), set = msec)]
    pub msec: MSec,
    #[directive(args(1), set = sec)]
    pub sec: Sec,
    #[directive(args(1), set = bufs)]
    pub bufs: Bufs,
    #[directive(args(1), set = enum_values, values = propagation::TYPES)]
    pub ctx: propagation::Type,
    #[directive(args(1), set = bitmask, values = propagation::TYPES)]
    pub bitmask: propagation::Type,
}

mod propagation {
    use std::mem;

    use ngx_mod::rt::{core::Unset, ngx_enum_values};

    bitflags::bitflags! {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct Type: usize {
            const EXTRACT = 0x0001;
            const INJECT = 0x0002;
        }
    }

    impl Unset for Type {
        const UNSET: Self = unsafe { mem::transmute(usize::MAX) };

        fn is_unset(&self) -> bool {
            *self == Self::UNSET
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
