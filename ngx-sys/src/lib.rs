#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

cfg_if::cfg_if! {
    if #[cfg(feature = "gen")] {
        include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    } else {
        mod bindings;

        pub use self::bindings::*;
    }
}
