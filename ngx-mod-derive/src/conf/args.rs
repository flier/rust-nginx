use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens, TokenStreamExt};

#[derive(Clone, Debug, PartialEq)]
pub enum Args {
    NoArgs = 0,
    Take1,
    Take2,
    Take3,
    Take4,
    Take5,
    Take6,
    Take7,
    Block,
    Flag,
}

impl From<usize> for Args {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::NoArgs,
            1 => Self::Take1,
            2 => Self::Take2,
            3 => Self::Take3,
            4 => Self::Take4,
            5 => Self::Take5,
            6 => Self::Take6,
            7 => Self::Take7,
            _ => abort!("`args` should be 0..=7, got: {}", value),
        }
    }
}

impl ToTokens for Args {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Args::*;

        tokens.append_all(match self {
            NoArgs => quote! { ::ngx_mod::rt::ffi::NGX_CONF_NOARGS },
            Take1 => quote! { ::ngx_mod::rt::ffi::NGX_CONF_TAKE1 },
            Take2 => quote! { ::ngx_mod::rt::ffi::NGX_CONF_TAKE2 },
            Take3 => quote! { ::ngx_mod::rt::ffi::NGX_CONF_TAKE3 },
            Take4 => quote! { ::ngx_mod::rt::ffi::NGX_CONF_TAKE4 },
            Take5 => quote! { ::ngx_mod::rt::ffi::NGX_CONF_TAKE5 },
            Take6 => quote! { ::ngx_mod::rt::ffi::NGX_CONF_TAKE6 },
            Take7 => quote! { ::ngx_mod::rt::ffi::NGX_CONF_TAKE7 },
            Block => quote! { ::ngx_mod::rt::ffi::NGX_CONF_BLOCK },
            Flag => quote! { ::ngx_mod::rt::ffi::NGX_CONF_FLAG },
        })
    }
}
