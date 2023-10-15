use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

use crate::util::find_ngx_rt;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Any,
    Main,
    Direct,
    #[cfg(feature = "event")]
    Event,
    #[cfg(feature = "http")]
    HttpMain,
    #[cfg(feature = "http")]
    HttpServer,
    #[cfg(feature = "http")]
    HttpLocation,
    #[cfg(feature = "http")]
    HttpUpstream,
    #[cfg(feature = "http")]
    HttpServerIf,
    #[cfg(feature = "http")]
    HttpLocationIf,
    #[cfg(feature = "http")]
    HttpLimitExcept,
    #[cfg(feature = "stream")]
    StreamMain,
    #[cfg(feature = "stream")]
    StreamServer,
    #[cfg(feature = "stream")]
    StreamUpstream,
    #[cfg(feature = "mail")]
    MailMain,
    #[cfg(feature = "mail")]
    MailServer,
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Type::*;

        let ngx_rt = find_ngx_rt();

        tokens.append_all(match self {
            Main => quote! { #ngx_rt ::ffi::NGX_CONF_MAIN },
            Any => quote! { #ngx_rt ::ffi::NGX_CONF_ANY },
            Direct => quote! { #ngx_rt ::ffi::NGX_CONF_DIRECT },
            Event => quote! { #ngx_rt ::ffi::NGX_CONF_EVENT },
            #[cfg(feature = "http")]
            HttpMain => quote! { #ngx_rt ::ffi::NGX_HTTP_MAIN_CONF },
            #[cfg(feature = "http")]
            HttpServer => quote! { #ngx_rt ::ffi::NGX_HTTP_SRV_CONF },
            #[cfg(feature = "http")]
            HttpLocation => quote! { #ngx_rt ::ffi::NGX_HTTP_LOC_CONF },
            #[cfg(feature = "http")]
            HttpUpstream => quote! { #ngx_rt ::ffi::NGX_HTTP_UPS_CONF },
            #[cfg(feature = "http")]
            HttpServerIf => quote! { #ngx_rt ::ffi::NGX_HTTP_SIF_CONF },
            #[cfg(feature = "http")]
            HttpLocationIf => quote! { #ngx_rt ::ffi::NGX_HTTP_LIF_CONF },
            #[cfg(feature = "http")]
            HttpLimitExcept => quote! { #ngx_rt ::ffi::NGX_HTTP_LMT_CONF },
            #[cfg(feature = "stream")]
            StreamMain => quote! { #ngx_rt ::ffi::NGX_STREAM_MAIN_CONF },
            #[cfg(feature = "stream")]
            StreamServer => quote! { #ngx_rt ::ffi::NGX_STREAM_SRV_CONF },
            #[cfg(feature = "stream")]
            StreamUpstream => quote! { #ngx_rt ::ffi::NGX_STREAM_UPS_CONF },
            #[cfg(feature = "mail")]
            MailMain => quote! { #ngx_rt ::ffi::NGX_MAIL_MAIN_CONF },
            #[cfg(feature = "mail")]
            MailServer => quote! { #ngx_rt ::ffi::NGX_MAIL_SRV_CONF },
        })
    }
}
