use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

#[derive(Clone, Debug, PartialEq)]
pub enum Offset {
    #[cfg(feature = "http")]
    HttpMain,
    #[cfg(feature = "http")]
    HttpServer,
    #[cfg(feature = "http")]
    HttpLocation,
    #[cfg(feature = "stream")]
    StreamMain,
    #[cfg(feature = "stream")]
    StreamServer,
    #[cfg(feature = "mail")]
    MailMain,
    #[cfg(feature = "mail")]
    MailServer,
}

impl ToTokens for Offset {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Offset::*;

        tokens.append_all(match self {
            #[cfg(feature = "http")]
            HttpMain => quote! { ::ngx_mod::rt::ffi::NGX_RS_HTTP_MAIN_CONF_OFFSET },
            #[cfg(feature = "http")]
            HttpServer => quote! { ::ngx_mod::rt::ffi::NGX_RS_HTTP_SRV_CONF_OFFSET },
            #[cfg(feature = "http")]
            HttpLocation => quote! { ::ngx_mod::rt::ffi::NGX_RS_HTTP_LOC_CONF_OFFSET },
            #[cfg(feature = "stream")]
            StreamMain => quote! { ::ngx_mod::rt::ffi::NGX_RS_STREAM_MAIN_CONF_OFFSET },
            #[cfg(feature = "stream")]
            StreamServer => quote! { ::ngx_mod::rt::ffi::NGX_RS_STREAM_SRV_CONF_OFFSET },
            #[cfg(feature = "mail")]
            MailMain => quote! { ::ngx_mod::rt::ffi::NGX_RS_MAIL_MAIN_CONF_OFFSET },
            #[cfg(feature = "mail")]
            MailServer => quote! { ::ngx_mod::rt::ffi::NGX_RS_MAIL_SRV_CONF_OFFSET },
        })
    }
}
