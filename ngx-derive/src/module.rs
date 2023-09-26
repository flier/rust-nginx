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
    Attribute, Ident, ItemStatic, LitStr,
};

use ngx_rt::core::ModuleType;

#[derive(Clone, Debug, Default, Merge, StructMeta)]
struct Args {
    ty: Option<NameValue<Type>>,
    name: Option<NameValue<LitStr>>,
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

    pub fn name(&self) -> Option<String> {
        self.name.as_ref().map(|n| n.value.value())
    }

    pub fn ty(&self) -> ModuleType {
        self.ty
            .as_ref()
            .map_or(ModuleType::Http, |ty| ty.value.clone().into())
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

impl From<Type> for ModuleType {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Core(_) => ModuleType::Core,
            Type::Conf(_) => ModuleType::Conf,
            Type::Event(_) => ModuleType::Event,
            Type::Http(_) => ModuleType::Http,
            Type::Mail(_) => ModuleType::Mail,
            Type::Stream(_) => ModuleType::Stream,
        }
    }
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
    let (args, attrs) = Args::extract(input.attrs);

    let ident: &Ident = &input.ident;
    let name = Ident::new(
        args.name()
            .unwrap_or_else(|| input.ident.to_string())
            .to_snake()
            .as_str(),
        Span::call_site(),
    );
    let ctx_name = format_ident!("{}_ctx", &name);

    let ngx_module: ItemStatic = parse_quote! {
        #[no_mangle]
        pub static mut #name: ::ngx_mod::ffi::ngx_module_t = ::ngx_mod::ffi::ngx_module_t {
            ctx_index: ::ngx_mod::core::UNSET_INDEX,
            index: ::ngx_mod::core::UNSET_INDEX,
            name: ::std::ptr::null_mut(),
            spare0: 0,
            spare1: 0,
            version: ::ngx_mod::ffi::nginx_version as ::ngx_mod::ffi::ngx_uint_t,
            signature: ::ngx_mod::ffi::NGX_RS_MODULE_SIGNATURE.as_ptr() as *const ::std::ffi::c_char,

            ctx: & #ctx_name as *const _ as *mut _,
            commands: unsafe { &ngx_http_upstream_custom_commands[0] as *const _ as *mut _ },
            type_: ::ngx_mod::ffi::NGX_HTTP_MODULE as ::ngx_mod::ffi::ngx_uint_t,

            init_master: Some(<#ident as ::ngx_mod::core::UnsafeModule>::init_master),
            init_module: Some(<#ident as ::ngx_mod::core::UnsafeModule>::init_module),
            init_process: Some(<#ident as ::ngx_mod::core::UnsafeModule>::init_process),
            init_thread: Some(<#ident as ::ngx_mod::core::UnsafeModule>::init_thread),
            exit_thread: Some(<#ident as ::ngx_mod::core::UnsafeModule>::exit_thread),
            exit_process: Some(<#ident as ::ngx_mod::core::UnsafeModule>::exit_process),
            exit_master: Some(<#ident as ::ngx_mod::core::UnsafeModule>::exit_master),

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

    let ngx_module_ctx: Option<ItemStatic> = match args.ty() {
        ModuleType::Http => Some(parse_quote! {
            #[no_mangle]
            static #ctx_name: ::ngx_mod::ffi::ngx_http_module_t = ::ngx_mod::ffi::ngx_http_module_t {
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
    };

    quote! {
        #ngx_module
        #ngx_module_ctx
    }
}
