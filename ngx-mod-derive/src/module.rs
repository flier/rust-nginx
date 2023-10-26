use case::CaseExt;
use cfg_if::cfg_if;
use merge::Merge;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use structmeta::{NameValue, StructMeta};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Error, Ident, ItemImpl, ItemStatic,
};

use crate::{
    extract,
    util::{find_ngx_mod, find_ngx_rt},
};

#[derive(Clone, Debug, Default, Merge, StructMeta)]
struct Args {
    #[struct_meta(name = "type")]
    ty: Option<NameValue<Type>>,
    name: Option<NameValue<Ident>>,
}

#[derive(Clone, Debug)]
enum Type {
    Core(kw::core),
    Conf(kw::conf),
    #[cfg(feature = "event")]
    Event(kw::event),
    #[cfg(feature = "http")]
    Http(kw::http),
    #[cfg(feature = "mail")]
    Mail(kw::mail),
    #[cfg(feature = "stream")]
    Stream(kw::stream),
}

mod kw {
    syn::custom_keyword!(core);
    syn::custom_keyword!(conf);
    syn::custom_keyword!(event);
    syn::custom_keyword!(http);
    syn::custom_keyword!(mail);
    syn::custom_keyword!(stream);
}

impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::core) {
            input.parse().map(Type::Core)
        } else if lookahead.peek(kw::conf) {
            input.parse().map(Type::Conf)
        } else if lookahead.peek(kw::event) {
            cfg_if! {
                if #[cfg(feature = "event")] {
                    input.parse().map(Type::Event)
                } else {
                    Err(Error::new(Span::call_site(), "`event` feature is disabled"))
                }
            }
        } else if lookahead.peek(kw::http) {
            cfg_if! {
                if #[cfg(feature = "http")] {
                    input.parse().map(Type::Http)
                } else {
                    Err(Error::new(Span::call_site(), "`http` feature is disabled"))
                }
            }
        } else if lookahead.peek(kw::mail) {
            cfg_if! {
                if #[cfg(feature = "mail")] {
                    input.parse().map(Type::Mail)
                } else {
                    Err(Error::new(Span::call_site(), "`mail` feature is disabled"))
                }
            }
        } else if lookahead.peek(kw::stream) {
            cfg_if! {
                if #[cfg(feature = "stream")] {
                    input.parse().map(Type::Stream)
                } else {
                    Err(Error::new(Span::call_site(), "`stream` feature is disabled"))
                }
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ngx_rt = find_ngx_rt();

        tokens.append_all(match self {
            Self::Core(_) => quote! { #ngx_rt ::ffi::NGX_CORE_MODULE },
            Self::Conf(_) => quote! { #ngx_rt ::ffi::NGX_CONF_MODULE },
            #[cfg(feature = "event")]
            Self::Event(_) => quote! { #ngx_rt ::ffi::NGX_EVENT_MODULE },
            #[cfg(feature = "http")]
            Self::Http(_) => quote! { #ngx_rt ::ffi::NGX_HTTP_MODULE },
            #[cfg(feature = "mail")]
            Self::Mail(_) => quote! { #ngx_rt ::ffi::NGX_MAIL_MODULE },
            #[cfg(feature = "stream")]
            Self::Stream(_) => quote! { #ngx_rt ::ffi::NGX_STREAM_MODULE },
        })
    }
}

pub fn expand(input: syn::DeriveInput) -> TokenStream {
    let (args, _) = extract::args::<Args, _>(input.attrs, "module");
    let args = args.unwrap_or_default();

    let ident: &Ident = &input.ident;
    let base_name = {
        let mut s = args
            .name
            .as_ref()
            .map(|n| n.value.to_string())
            .unwrap_or_else(|| input.ident.to_string())
            .to_snake();

        if !s.starts_with("ngx_") {
            s = format!("ngx_{}", s);
        }

        s
    };
    let mod_name = {
        let mut s = base_name.clone();

        if !s.ends_with("_module") {
            s += "_module";
        }

        s
    };
    let ngx_module_name = Ident::new(mod_name.as_str(), Span::call_site());
    let ngx_module_ctx_name = format_ident!("{}_ctx", &mod_name);
    let ngx_module_cmds_name = format_ident!("{}_commands", &mod_name);

    let module_ty = args.ty.as_ref().map_or_else(
        || Type::Http(parse_quote! { http }),
        |arg| arg.value.clone(),
    );

    let ngx_rt = find_ngx_rt();
    let ngx_mod = find_ngx_mod();

    let ngx_module: ItemStatic = parse_quote! {
        #[no_mangle]
        pub static mut #ngx_module_name: #ngx_rt ::ffi::ngx_module_t = #ngx_rt ::ffi::ngx_module_t {
            ctx_index: #ngx_mod ::UNSET_INDEX,
            index: #ngx_mod ::UNSET_INDEX,
            name: ::std::ptr::null_mut(),
            spare0: 0,
            spare1: 0,
            version: #ngx_rt ::ffi::nginx_version as #ngx_rt ::ffi::ngx_uint_t,
            signature: #ngx_rt ::ffi::NGX_RS_MODULE_SIGNATURE.as_ptr() as *const ::std::ffi::c_char,

            ctx: & #ngx_module_ctx_name as *const _ as *mut _,
            commands: unsafe { & #ngx_module_cmds_name [0] as *const _ as *mut _ },
            type_: #module_ty as #ngx_rt ::ffi::ngx_uint_t,

            init_master: Some(<#ident as #ngx_mod ::UnsafeModule>::init_master),
            init_module: Some(<#ident as #ngx_mod ::UnsafeModule>::init_module),
            init_process: Some(<#ident as #ngx_mod ::UnsafeModule>::init_process),
            init_thread: Some(<#ident as #ngx_mod ::UnsafeModule>::init_thread),
            exit_thread: Some(<#ident as #ngx_mod ::UnsafeModule>::exit_thread),
            exit_process: Some(<#ident as #ngx_mod ::UnsafeModule>::exit_process),
            exit_master: Some(<#ident as #ngx_mod ::UnsafeModule>::exit_master),

            spare_hook0: 0,
            spare_hook1: 0,
            spare_hook2: 0,
            spare_hook3: 0,
            spare_hook4: 0,
            spare_hook5: 0,
            spare_hook6: 0,
            spare_hook7: 0,
        };
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let impl_module_metadata: ItemImpl = parse_quote! {
        impl #impl_generics #ngx_mod ::ModuleMetadata for #ident #ty_generics #where_clause {
            fn module() -> &'static #ngx_rt ::core::ModuleRef {
                unsafe { #ngx_rt ::foreign_types::ForeignTypeRef::from_ptr(&mut #ngx_module_name as *mut _) }
            }

            fn commands() -> #ngx_rt ::core::Cmds<'static> {
                #ngx_rt ::core::Cmds::from(unsafe { & #ngx_module_cmds_name [..#ngx_module_cmds_name.len() - 1] })
            }
        }
    };

    let impl_deref: ItemImpl = parse_quote! {
        impl #impl_generics ::std::ops::Deref for #ident #ty_generics #where_clause {
            type Target = #ngx_rt ::core::ModuleRef;

            fn deref(&self) -> &Self::Target {
                unsafe { #ngx_rt ::foreign_types::ForeignTypeRef::from_ptr(&mut #ngx_module_name as *mut _) }
            }
        }
    };

    let impl_as_ref: ItemImpl = parse_quote! {
        impl #impl_generics ::std::convert::AsRef<#ngx_rt ::core::ModuleRef> for #ident #ty_generics #where_clause {
            fn as_ref(&self) -> &#ngx_rt ::core::ModuleRef {
                unsafe { #ngx_rt ::foreign_types::ForeignTypeRef::from_ptr(&mut #ngx_module_name as *mut _) }
            }
        }
    };

    let (ngx_module_ctx, ngx_module_cmds): (Option<ItemStatic>, Option<ItemStatic>) =
        match module_ty {
            Type::Core(_) => (
                Some(parse_quote! {
                    #[no_mangle]
                    static #ngx_module_ctx_name: #ngx_rt ::ffi::ngx_core_module_t = #ngx_rt ::ffi::ngx_core_module_t {
                        name: #ngx_rt ::ngx_str!( #mod_name ),
                        create_conf: Some(<#ident as #ngx_mod ::core::UnsafeModule>::create_conf),
                        init_conf: Some(<#ident as #ngx_mod ::core::UnsafeModule>::init_conf),
                    };
                }),
                Some(parse_quote! {
                    #[no_mangle]
                    static mut #ngx_module_cmds_name: [#ngx_rt ::ffi::ngx_command_t;
                        <<#ident as #ngx_mod ::core::Module>::Conf as #ngx_rt ::core::UnsafeConf>::COMMANDS.len() +
                    1] = unsafe {
                        #ngx_mod ::const_concat!(
                            <<#ident as #ngx_mod ::core::Module> :: Conf as #ngx_rt ::core::UnsafeConf>::COMMANDS,
                            [ #ngx_rt ::ngx_command!() ]
                        )
                    };
                }),
            ),
            Type::Http(_) => (
                Some(parse_quote! {
                    #[no_mangle]
                    static #ngx_module_ctx_name: #ngx_rt ::ffi::ngx_http_module_t = #ngx_rt ::ffi::ngx_http_module_t {
                        preconfiguration: Some(<#ident as #ngx_mod ::http::UnsafeModule>::preconfiguration),
                        postconfiguration: Some(<#ident as #ngx_mod ::http::UnsafeModule>::postconfiguration),
                        create_main_conf: Some(<#ident as #ngx_mod ::http::UnsafeModule>::create_main_conf),
                        init_main_conf: Some(<#ident as #ngx_mod ::http::UnsafeModule>::init_main_conf),
                        create_srv_conf: Some(<#ident as #ngx_mod ::http::UnsafeModule>::create_srv_conf),
                        merge_srv_conf: Some(<#ident as #ngx_mod ::http::UnsafeModule>::merge_srv_conf),
                        create_loc_conf: Some(<#ident as #ngx_mod ::http::UnsafeModule>::create_loc_conf),
                        merge_loc_conf: Some(<#ident as #ngx_mod ::http::UnsafeModule>::merge_loc_conf),
                    };
                }),
                Some(parse_quote! {
                    #[no_mangle]
                    static mut #ngx_module_cmds_name: [#ngx_rt ::ffi::ngx_command_t;
                        <<#ident as #ngx_mod ::http::Module> :: MainConf as #ngx_rt ::core::UnsafeConf>::COMMANDS.len() +
                        <<#ident as #ngx_mod ::http::Module> :: SrvConf as #ngx_rt ::core::UnsafeConf>::COMMANDS.len() +
                        <<#ident as #ngx_mod ::http::Module> :: LocConf as #ngx_rt ::core::UnsafeConf>::COMMANDS.len() +
                    1] = unsafe {
                        #ngx_mod ::const_concat!(
                            <<#ident as #ngx_mod ::http::Module> :: MainConf as #ngx_rt ::core::UnsafeConf>::COMMANDS,
                            <<#ident as #ngx_mod ::http::Module> :: SrvConf as #ngx_rt ::core::UnsafeConf>::COMMANDS,
                            <<#ident as #ngx_mod ::http::Module> :: LocConf as #ngx_rt ::core::UnsafeConf>::COMMANDS,
                            [ #ngx_rt ::ngx_command!() ]
                        )
                    };
                }),
            ),
            Type::Stream(_) => (
                Some(parse_quote! {
                    #[no_mangle]
                    static #ngx_module_ctx_name: #ngx_rt ::ffi::ngx_stream_module_t = #ngx_rt ::ffi::ngx_stream_module_t {
                        preconfiguration: Some(<#ident as #ngx_mod ::stream::UnsafeModule>::preconfiguration),
                        postconfiguration: Some(<#ident as #ngx_mod ::stream::UnsafeModule>::postconfiguration),
                        create_main_conf: Some(<#ident as #ngx_mod ::stream::UnsafeModule>::create_main_conf),
                        init_main_conf: Some(<#ident as #ngx_mod ::stream::UnsafeModule>::init_main_conf),
                        create_srv_conf: Some(<#ident as #ngx_mod ::stream::UnsafeModule>::create_srv_conf),
                        merge_srv_conf: Some(<#ident as #ngx_mod ::stream::UnsafeModule>::merge_srv_conf),
                    };
                }),
                Some(parse_quote! {
                    #[no_mangle]
                    static mut #ngx_module_cmds_name: [#ngx_rt ::ffi::ngx_command_t;
                        <<#ident as #ngx_mod ::stream::Module> :: MainConf as #ngx_rt ::core::UnsafeConf>::COMMANDS.len() +
                        <<#ident as #ngx_mod ::stream::Module> :: SrvConf as #ngx_rt ::core::UnsafeConf>::COMMANDS.len() +
                    1] = unsafe {
                        #ngx_mod ::const_concat!(
                            <<#ident as #ngx_mod ::stream::Module> :: MainConf as #ngx_rt ::core::UnsafeConf>::COMMANDS,
                            <<#ident as #ngx_mod ::stream::Module> :: SrvConf as #ngx_rt ::core::UnsafeConf>::COMMANDS,
                            [ #ngx_rt ::ngx_command!() ]
                        )
                    };
                }),
            ),
            _ => (None, None),
        };

    let ngx_modules: ItemStatic = parse_quote! {
        #[no_mangle]
        pub static mut ngx_modules: [*const #ngx_rt ::ffi::ngx_module_t; 2] = [
            unsafe { & #ngx_module_name as *const _ },
            ::std::ptr::null(),
        ];
    };

    let ngx_module_names: ItemStatic = parse_quote! {
        #[no_mangle]
        pub static mut ngx_module_names: [*const ::std::ffi::c_char; 2] = [
            concat!(stringify!(#ngx_module_name), "\0").as_ptr() as *const _,
            ::std::ptr::null(),
        ];
    };

    let ngx_module_order: ItemStatic = parse_quote! {
        #[no_mangle]
        pub static mut ngx_module_order: [*const ::std::ffi::c_char; 1] = [::std::ptr::null()];
    };

    quote! {
        #ngx_module

        #impl_module_metadata
        #impl_deref
        #impl_as_ref

        #ngx_module_ctx
        #ngx_module_cmds

        #ngx_modules
        #ngx_module_names
        #ngx_module_order
    }
}
