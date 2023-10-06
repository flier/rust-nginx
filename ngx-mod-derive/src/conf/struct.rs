use cfg_if::cfg_if;
use merge::Merge;
use proc_macro_error::abort;
use quote::quote;
use structmeta::{NameValue, StructMeta};
use syn::{spanned::Spanned, Ident, Path};

use super::Type;

#[derive(Clone, Debug, Default, Merge, StructMeta)]
pub struct StructArgs {
    #[struct_meta(unnamed)]
    #[merge(strategy = merge::vec::append)]
    pub ty: Vec<Path>,
    pub name: Option<NameValue<Ident>>,
}

impl StructArgs {
    pub fn conf_type(&self) -> Vec<Type> {
        use Type::*;

        if self.ty.is_empty() {
            vec![Any]
        } else {
            self.ty.iter().map(|p| {
                let name = quote! { #p }.to_string().to_lowercase();

                if name.starts_with("http") {
                    cfg_if! {
                        if #[cfg(feature = "http")] {
                            match name.as_str() {
                                "http" | "http :: main" => HttpMain,
                                "http :: srv" | "http :: server" => HttpServer,
                                "http :: loc" | "http :: location" => HttpLocation,
                                "http :: ups" | "http :: upstream" => HttpUpstream,
                                "http :: sif" | "http :: srv_if" | "http :: server :: if" => HttpServerIf,
                                "http :: lif" | "http :: loc_if" | "http :: location :: if" => HttpLocationIf,
                                "http :: lmt" | "http :: limit" | "http :: limit_except" => HttpLimitExcept,
                                _ => abort!(p.span(), "unknown directive type")
                            }
                        } else {
                            abort!(p.span(), "`http` support is disabled");
                        }
                    }
                } else if name.starts_with("stream") {
                    cfg_if! {
                        if #[cfg(feature = "stream")] {
                            match name.as_str() {
                                "stream" | "stream :: main" => StreamMain,
                                "stream :: srv" | "stream :: server" => StreamServer,
                                "stream :: ups" | "stream :: upstream" => StreamUpstream,
                                _ => abort!(p.span(), "unknown directive type"),
                            }
                        } else {
                            abort!(p.span(), "`stream` support is disabled");
                        }
                    }
                } else if name.starts_with("mail") {
                    cfg_if! {
                        if #[cfg(feature = "mail")] {
                            match name.as_str() {
                                "mail" | "mail :: main" => MailMain,
                                "mail :: srv" | "mail :: server" => MailServer,
                                _ => abort!(p.span(), "unknown directive type"),
                            }
                        } else {
                            abort!(p.span(), "`mail` support is disabled");
                        }
                    }
                } else if name == "main" {
                    Main
                } else if name == "any" {
                    Any
                } else if name == "direct" {
                    Direct
                } else if name == "event" {
                    cfg_if! {
                        if #[cfg(feature = "event")] {
                            Event
                        } else {
                            abort!(p.span(), "`event` support is disabled");
                        }
                    }
                } else {
                    abort!(p.span(), "unknown directive type");
                }
            }).collect()
        }
    }
}
