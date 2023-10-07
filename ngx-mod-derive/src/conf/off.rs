use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

use super::Type;

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

impl Offset {
    pub fn for_conf(conf_types: &[Type]) -> Option<Offset> {
        cfg_if! {
            if #[cfg(feature = "http")] {
                if conf_types.contains(&Type::HttpLocation) || conf_types.contains(&Type::HttpLocationIf) ||conf_types.contains(&Type::HttpLimitExcept)  {
                    return Some(Offset::HttpLocation)
                } else if conf_types.contains(&Type::HttpServer) || conf_types.contains(&Type::HttpServerIf) ||conf_types.contains(&Type::HttpUpstream) {
                    return Some(Offset::HttpServer)
                } else if conf_types.contains(&Type::HttpMain) {
                    return Some(Offset::HttpMain)
                }
            }
        }

        cfg_if! {
            if #[cfg(feature = "stream")] {
                if conf_types.contains(&Type::StreamServer) || conf_types.contains(&Type::StreamUpstream) {
                    return Some(Offset::StreamServer)
                } else if conf_types.contains(&Type::StreamMain) {
                    return Some(Offset::StreamMain)
                }
            }
        }

        cfg_if! {
            if #[cfg(feature = "mail")] {
                if conf_types.contains(&Type::MailSever) {
                    return Some(Offset::MailSever)
                } else if conf_types.contains(&Type::MailMain) {
                    return Some(Offset::MailMain)
                }
            }
        }

        None
    }
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
