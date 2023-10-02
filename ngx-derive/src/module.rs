use case::CaseExt;
use merge::Merge;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use structmeta::{NameValue, StructMeta};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    Attribute, Ident, ItemImpl, ItemStatic,
};

#[derive(Clone, Debug, Default, Merge, StructMeta)]
struct Args {
    #[struct_meta(name = "type")]
    ty: Option<NameValue<Type>>,
    name: Option<NameValue<Ident>>,
}

impl Args {
    pub fn extract<I: IntoIterator<Item = Attribute>>(attrs: I) -> (Self, Vec<Attribute>) {
        let (args, attrs): (Vec<_>, Vec<_>) =
            attrs.into_iter().partition(|f| f.path().is_ident("module"));

        let args = args
            .into_iter()
            .map(|attr| match attr.parse_args::<Args>() {
                Ok(arg) => arg,
                Err(err) => abort!(attr.span(), "fail to parse args, {}", err),
            })
            .fold(Args::default(), |mut args, arg| {
                args.merge(arg);
                args
            });
        let attrs = attrs
            .into_iter()
            .filter(|attr| {
                WELL_KNOWN_ATTRS
                    .iter()
                    .any(|name| attr.path().is_ident(name))
            })
            .collect();

        (args, attrs)
    }
}

#[derive(Clone, Debug)]
enum Type {
    Core(kw::core),
    Conf(kw::conf),
    Event(kw::event),
    Http(kw::http),
    Mail(kw::mail),
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
            input.parse().map(Type::Event)
        } else if lookahead.peek(kw::http) {
            input.parse().map(Type::Http)
        } else if lookahead.peek(kw::mail) {
            input.parse().map(Type::Mail)
        } else if lookahead.peek(kw::stream) {
            input.parse().map(Type::Stream)
        } else {
            Err(lookahead.error())
        }
    }
}

const WELL_KNOWN_ATTRS: &[&str] = &["allow", "deny", "doc", "cfg"];

pub fn expand(input: syn::DeriveInput) -> TokenStream {
    let (args, _) = Args::extract(input.attrs);

    let ident: &Ident = &input.ident;
    let mod_name = args
        .name
        .as_ref()
        .map(|n| n.value.to_string())
        .unwrap_or_else(|| {
            let mut s = input.ident.to_string();

            if !s.starts_with("Ngx") {
                s = format!("Ngx{}", s);
            }

            if !s.ends_with("Module") {
                s += "Module";
            }

            s
        })
        .to_snake();
    let ngx_module_name = Ident::new(mod_name.as_str(), Span::call_site());
    let ngx_module_ctx_name = format_ident!("{}_ctx", &mod_name);

    let ngx_module: ItemStatic = parse_quote! {
        #[no_mangle]
        pub static mut #ngx_module_name: ::ngx_mod::ffi::ngx_module_t = ::ngx_mod::ffi::ngx_module_t {
            ctx_index: ::ngx_mod::UNSET_INDEX,
            index: ::ngx_mod::UNSET_INDEX,
            name: ::std::ptr::null_mut(),
            spare0: 0,
            spare1: 0,
            version: ::ngx_mod::ffi::nginx_version as ::ngx_mod::ffi::ngx_uint_t,
            signature: ::ngx_mod::ffi::NGX_RS_MODULE_SIGNATURE.as_ptr() as *const ::std::ffi::c_char,

            ctx: & #ngx_module_ctx_name as *const _ as *mut _,
            commands: unsafe { &ngx_http_upstream_custom_commands[0] as *const _ as *mut _ },
            type_: ::ngx_mod::ffi::NGX_HTTP_MODULE as ::ngx_mod::ffi::ngx_uint_t,

            init_master: Some(<#ident as ::ngx_mod::UnsafeModule>::init_master),
            init_module: Some(<#ident as ::ngx_mod::UnsafeModule>::init_module),
            init_process: Some(<#ident as ::ngx_mod::UnsafeModule>::init_process),
            init_thread: Some(<#ident as ::ngx_mod::UnsafeModule>::init_thread),
            exit_thread: Some(<#ident as ::ngx_mod::UnsafeModule>::exit_thread),
            exit_process: Some(<#ident as ::ngx_mod::UnsafeModule>::exit_process),
            exit_master: Some(<#ident as ::ngx_mod::UnsafeModule>::exit_master),

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

    let impl_module: ItemImpl = parse_quote! {
        impl #impl_generics ::ngx_mod::ModuleMetadata for #ident #ty_generics #where_clause {
            fn module() -> &'static ::ngx_mod::rt::core::ModuleRef {
                unsafe { ::ngx_mod::rt::core::ModuleRef::from_ptr(&mut #ngx_module_name as *mut _) }
            }
        }
    };

    let ngx_module_ctx: Option<ItemStatic> = args.ty.as_ref().and_then(|ty|match ty.value {
        Type::Core(_) => Some(parse_quote! {
            #[no_mangle]
            static #ngx_module_ctx_name: ::ngx_mod::ffi::ngx_core_module_t = ::ngx_mod::ffi::ngx_core_module_t {
                name: ::ngx_mod::rt::ngx_str!( #mod_name ),
                create_conf: Some(<#ident as ::ngx_mod::core::UnsafeModule>::create_conf),
                init_conf: Some(<#ident as ::ngx_mod::core::UnsafeModule>::init_conf),
            };
        }),
        Type::Http(_) => Some(parse_quote! {
            #[no_mangle]
            static #ngx_module_ctx_name: ::ngx_mod::ffi::ngx_http_module_t = ::ngx_mod::ffi::ngx_http_module_t {
                preconfiguration: Some(<#ident as ::ngx_mod::http::UnsafeModule>::preconfiguration),
                postconfiguration: Some(<#ident as ::ngx_mod::http::UnsafeModule>::postconfiguration),
                create_main_conf: Some(<#ident as ::ngx_mod::http::UnsafeModule>::create_main_conf),
                init_main_conf: Some(<#ident as ::ngx_mod::http::UnsafeModule>::init_main_conf),
                create_srv_conf: Some(<#ident as ::ngx_mod::http::UnsafeModule>::create_srv_conf),
                merge_srv_conf: Some(<#ident as ::ngx_mod::http::UnsafeModule>::merge_srv_conf),
                create_loc_conf: Some(<#ident as ::ngx_mod::http::UnsafeModule>::create_loc_conf),
                merge_loc_conf: Some(<#ident as ::ngx_mod::http::UnsafeModule>::merge_loc_conf),
            };
        }),
        _ => None,
    });

    let ngx_modules: ItemStatic = parse_quote! {
        #[no_mangle]
        pub static mut ngx_modules: [*const ::ngx_mod::ffi::ngx_module_t; 2] = [
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

        #impl_module

        #ngx_module_ctx

        #ngx_modules
        #ngx_module_names
        #ngx_module_order
    }
}
