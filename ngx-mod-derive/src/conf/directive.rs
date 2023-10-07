use case::CaseExt;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Ident, Type, TypePath};

use super::{FieldArgs, Offset, Set, StructArgs};

pub struct Directive<'a> {
    pub struct_args: &'a StructArgs,
    pub struct_name: &'a Ident,
    pub args: FieldArgs,
    pub name: Ident,
    pub ty: Type,
}

impl<'a> ToTokens for Directive<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let struct_name = self.struct_name;
        let field_name = &self.name;
        let name = self.name();
        let conf_ty = self.struct_args.conf_type();
        let conf_off = self
            .args
            .conf_offset()
            .or_else(|| Offset::for_conf(&conf_ty))
            .map_or_else(|| quote! { 0usize }, |off: Offset| quote! { #off });
        let args = conf_ty
            .into_iter()
            .map(|ty| quote! { #ty })
            .chain(self.args.args().into_iter().map(|args| quote! { #args }));
        let set = self.set();

        tokens.append_all(quote! {
            ::ngx_mod::rt::ffi::ngx_command_t {
                name: ::ngx_mod::rt::ngx_str!( #name ),
                type_: ( #( #args )|* ) as usize,
                set: Some( #set ),
                conf: #conf_off as usize,
                offset: ::ngx_mod::memoffset::offset_of!( #struct_name , #field_name ) as usize,
                post: ::std::ptr::null_mut(),
            }
        })
    }
}

impl<'a> Directive<'a> {
    fn name(&self) -> String {
        self.args
            .name
            .as_ref()
            .map(|n| n.value.to_string())
            .unwrap_or_else(|| self.name.to_string())
            .to_snake()
    }

    fn set(&self) -> Set {
        if let Some(p) = self.args.set.as_ref().map(|p| &p.value) {
            p.get_ident()
                .and_then(|i| i.to_string().to_lowercase().as_str().parse().ok())
                .unwrap_or_else(|| Set::Setter(p.clone()))
        } else {
            match self.ty {
                Type::Path(TypePath {
                    ref qself,
                    ref path,
                }) if qself.is_none() => {
                    if let Some(ident) = path.get_ident() {
                        match ident.to_string().as_str() {
                            "bool" => Set::Flag,
                            "isize" => Set::Number,
                            _ => abort! { self.ty, "unexpected field type: {:?}", path },
                        }
                    } else {
                        abort! { self.ty, "unexpected field type: {:?}", path }
                    }
                }
                _ => abort! { self.ty, "unknown directive type: {:?}", self.ty },
            }
        }
    }
}
