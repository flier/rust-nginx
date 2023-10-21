use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Path, Token,
};

use crate::util::find_ngx_rt;

#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub types: Punctuated<Type, Token![|]>,
}

impl Parse for Scope {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Punctuated::parse_separated_nonempty(input).map(|types| Scope { types })
    }
}

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

impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use Type::*;

        let p: Path = input.parse()?;
        let name = quote! { #p }.to_string().to_lowercase();

        if name.starts_with("http") {
            cfg_if! {
                if #[cfg(feature = "http")] {
                    match name.as_str() {
                        "http" | "http :: main" => Ok(HttpMain),
                        "http :: srv" | "http :: server" => Ok(HttpServer),
                        "http :: loc" | "http :: location" => Ok(HttpLocation),
                        "http :: ups" | "http :: upstream" => Ok(HttpUpstream),
                        "http :: sif" | "http :: srv_if" | "http :: server :: if" => Ok(HttpServerIf),
                        "http :: lif" | "http :: loc_if" | "http :: location :: if" => Ok(HttpLocationIf),
                        "http :: lmt" | "http :: limit" | "http :: limit_except" => Ok(HttpLimitExcept),
                        _ => Err(Error::new(p.span(), "unknown `http` directive type"))
                    }
                } else {
                    abort!(p.span(), "`http` support is disabled");
                }
            }
        } else if name.starts_with("stream") {
            cfg_if! {
                if #[cfg(feature = "stream")] {
                    match name.as_str() {
                        "stream" | "stream :: main" => Ok(StreamMain),
                        "stream :: srv" | "stream :: server" => Ok(StreamServer),
                        "stream :: ups" | "stream :: upstream" => Ok(StreamUpstream),
                        _ => Err(Error::new(p.span(), "unknown `stream` directive type")),
                    }
                } else {
                    Err(Error::new(p.span(), "`stream` support is disabled"));
                }
            }
        } else if name.starts_with("mail") {
            cfg_if! {
                if #[cfg(feature = "mail")] {
                    match name.as_str() {
                        "mail" | "mail :: main" => Ok(MailMain),
                        "mail :: srv" | "mail :: server" => Ok(MailServer),
                        _ => Err(Error::new(p.span(), "unknown `mail` directive type")),
                    }
                } else {
                    Err(Error::new(p.span(), "`mail` support is disabled"))
                }
            }
        } else if name == "main" {
            Ok(Main)
        } else if name == "any" {
            Ok(Any)
        } else if name == "direct" {
            Ok(Direct)
        } else if name == "event" {
            cfg_if! {
                if #[cfg(feature = "event")] {
                    Ok(Event)
                } else {
                    Err(Error::new(p.span(), "`event` support is disabled"));
                }
            }
        } else {
            Err(Error::new(p.span(), "unknown directive type"))
        }
    }
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
