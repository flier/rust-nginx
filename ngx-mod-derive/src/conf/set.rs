use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Expr, Path, Type};

use crate::util::find_ngx_rt;

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
    #[cfg(feature = "http")]
    ComplexValue,
    #[cfg(feature = "http")]
    HttpTypes,
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
            #[cfg(feature = "http")]
            "complex_value" => ComplexValue,
            #[cfg(feature = "http")]
            "types" | "http_types" => HttpTypes,
            _ => return Err(()),
        })
    }
}

impl ToTokens for Set {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Set::*;

        let ngx_rt = find_ngx_rt();

        tokens.append_all(match self {
            Flag => quote! { #ngx_rt ::ffi::ngx_conf_set_flag_slot },
            Str => quote! { #ngx_rt ::ffi::ngx_conf_set_str_slot },
            StrArray => quote! { #ngx_rt ::ffi::ngx_conf_set_str_array_slot },
            KeyValue => quote! { #ngx_rt ::ffi::ngx_conf_set_keyval_slot },
            Number => quote! { #ngx_rt ::ffi::ngx_conf_set_num_slot },
            Size => quote! { #ngx_rt ::ffi::ngx_conf_set_size_slot },
            Offset => quote! { #ngx_rt ::ffi::ngx_conf_set_off_slot },
            MSec => quote! { #ngx_rt ::ffi::ngx_conf_set_msec_slot },
            Seconds => quote! { #ngx_rt ::ffi::ngx_conf_set_sec_slot },
            Buffers => quote! { #ngx_rt ::ffi::ngx_conf_set_bufs_slot },
            Enum => quote! { #ngx_rt ::ffi::ngx_conf_set_enum_slot },
            BitMask => quote! { #ngx_rt ::ffi::ngx_conf_set_bitmask_slot },
            Path => quote! { #ngx_rt ::ffi::ngx_conf_set_path_slot },
            Access => quote! { #ngx_rt ::ffi::ngx_conf_set_access_slot },
            #[cfg(feature = "http")]
            ComplexValue => quote! { #ngx_rt ::ffi::ngx_http_set_complex_value_slot },
            #[cfg(feature = "http")]
            HttpTypes => quote! { #ngx_rt ::ffi::ngx_http_types_slot },
            Setter(path) => quote! { #path },
        })
    }
}

impl Set {
    pub fn assert_eq_size(&self, ty: &Type) -> Option<Expr> {
        use Set::*;

        let ngx_rt = find_ngx_rt();
        let assert_eq_size: syn::Path = parse_quote! {
            #ngx_rt ::static_assertions::assert_eq_size
        };

        match self {
            Flag => Some(parse_quote! { #assert_eq_size!( #ty, #ngx_rt ::ffi::ngx_flag_t ) }),
            Str => Some(parse_quote! { #assert_eq_size!( #ty, #ngx_rt ::ffi::ngx_str_t ) }),
            StrArray | KeyValue => {
                Some(parse_quote! { #assert_eq_size!( #ty, * mut #ngx_rt ::ffi::ngx_array_t ) })
            }
            Number => Some(parse_quote! { #assert_eq_size!( #ty, #ngx_rt ::ffi::ngx_int_t ) }),
            Size => Some(parse_quote! { #assert_eq_size!( #ty, usize ) }),
            Offset => Some(parse_quote! { #assert_eq_size!( #ty, #ngx_rt ::ffi::off_t ) }),
            MSec => Some(parse_quote! { #assert_eq_size!( #ty, #ngx_rt ::ffi::ngx_msec_t ) }),
            Seconds => Some(parse_quote! { #assert_eq_size!( #ty, #ngx_rt ::ffi::time_t ) }),
            Buffers => Some(parse_quote! { #assert_eq_size!( #ty, #ngx_rt ::ffi::ngx_bufs_t ) }),
            Enum | BitMask | Access => {
                Some(parse_quote! { #assert_eq_size!( #ty, #ngx_rt ::ffi::ngx_uint_t ) })
            }
            Path => Some(parse_quote! { #assert_eq_size!( #ty, * mut #ngx_rt ::ffi::ngx_path_t ) }),
            #[cfg(feature = "http")]
            ComplexValue => Some(
                parse_quote! { #assert_eq_size!( #ty, * mut #ngx_rt ::ffi::ngx_http_complex_value_t ) },
            ),
            #[cfg(feature = "http")]
            HttpTypes => {
                Some(parse_quote! { #assert_eq_size!( #ty, * mut #ngx_rt ::ffi::ngx_array_t ) })
            }
            Setter(_) => None,
        }
    }
}
