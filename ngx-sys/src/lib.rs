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

/// FIXME: make `ngx_str_t` as opaque type for static variable
///
/// `*mut u8` cannot be shared between threads safely
///
/// help: within `ngx_core_module_t`, the trait `Sync` is not implemented for `*mut u8`
/// note: required because it appears within the type `ngx_str_t`
unsafe impl Sync for ngx_str_t {}
