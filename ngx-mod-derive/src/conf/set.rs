use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum Set {
    Flag,
    Str,
    StrArray,
    KeyValue,
    Number,
    Size,
    Offset,
    MSec,
    Seconds,
    Buffers,
    Enum,
    BitMask,
    Path,
    Access,
    ComplexValue,
    Setter(Path),
}

impl FromStr for Set {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Set::*;

        Ok(match s {
            "flag" => Flag,
            "str" => Str,
            "[str]" | "str_array" => StrArray,
            "kv" | "keyval" => KeyValue,
            "int" | "num" => Number,
            "size" => Size,
            "off" => Offset,
            "msec" => MSec,
            "sec" => Seconds,
            "bufs" => Buffers,
            "enum_values" => Enum,
            "bitmask" => BitMask,
            "path" => Path,
            "access" => Access,
            "complex_value" => ComplexValue,
            _ => return Err(()),
        })
    }
}

impl ToTokens for Set {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Set::*;

        tokens.append_all(match self {
            Flag => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_flag_slot },
            Str => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_str_slot },
            StrArray => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_str_array_slot },
            KeyValue => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_keyval_slot },
            Number => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_num_slot },
            Size => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_size_slot },
            Offset => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_off_slot },
            MSec => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_msec_slot },
            Seconds => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_sec_slot },
            Buffers => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_bufs_slot },
            Enum => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_enum_slot },
            BitMask => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_bitmask_slot },
            Path => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_path_slot },
            Access => quote! { ::ngx_mod::rt::ffi::ngx_conf_set_access_slot },
            ComplexValue => quote! { ::ngx_mod::rt::ffi::ngx_http_set_complex_value_slot },
            Setter(path) => quote! { #path },
        })
    }
}
