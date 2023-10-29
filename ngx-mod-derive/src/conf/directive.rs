use case::CaseExt;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Expr, GenericArgument, Ident, Path, Stmt, Type, TypePath};

use crate::util::{find_ngx_mod, find_ngx_rt};

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
        let ngx_rt = find_ngx_rt();
        let ngx_mod = find_ngx_mod();
        let struct_name = self.struct_name;
        let field_name = &self.name;
        let name = self.name();
        let conf_ty = self.struct_args.scope.as_ref().map_or_else(
            || vec![super::Type::Any],
            |scope| scope.types.iter().cloned().collect::<Vec<_>>(),
        );
        let conf_off = self
            .args
            .conf_offset()
            .or_else(|| Offset::for_conf(conf_ty.as_slice()))
            .map_or_else(|| quote! { 0usize }, |off: Offset| quote! { #off });
        let args = conf_ty
            .into_iter()
            .map(|ty| quote! { #ty })
            .chain(self.args.args().into_iter().map(|args| quote! { #args }));
        let set = self.set();
        let assertions: Option<Stmt> = self.assertions().map(|expr| parse_quote! { #expr ; });
        let post = if set == Set::Enum {
            if let Some(p) = self.args.values.as_ref().map(|arg| &arg.value) {
                quote! { #ngx_rt ::core::conf::enum_values( & #p ).as_ptr().cast() }
            } else {
                abort! {
                    self.name.span(), "missing enum values"
                }
            }
        } else {
            quote! { ::std::ptr::null_mut() }
        };

        tokens.append_all(quote! {
            #ngx_rt ::ffi::ngx_command_t {
                name: #ngx_rt ::ngx_str!( #name ),
                type_: ( #( #args )|* ) as usize,
                set: {
                    #assertions

                    Some( #set )
                },
                conf: #conf_off as usize,
                offset: #ngx_mod ::memoffset::offset_of!( #struct_name , #field_name ) as usize,
                post: #post,
            }
        })
    }
}

impl<'a> Directive<'a> {
    fn name(&self) -> String {
        self.args
            .name
            .as_ref()
            .map(|n| n.value.value())
            .unwrap_or_else(|| self.name.to_string().to_snake())
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
                            "usize" => Set::Size,
                            "MSec" => Set::MSec,
                            "Str" => Set::Str,
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

    pub fn assertions(&self) -> Option<Expr> {
        let mut ty = self.ty.clone();

        strip_type_lifetime(&mut ty);

        self.set().assert_eq_size(&ty)
    }
}

fn strip_type_lifetime(ty: &mut Type) {
    match ty {
        Type::Reference(syn::TypeReference {
            ref mut lifetime,
            elem,
            ..
        }) => {
            lifetime.take();

            strip_type_lifetime(elem.as_mut());
        }
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => {
            for s in segments.iter_mut() {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref mut args,
                    ..
                }) = s.arguments
                {
                    *args = args
                        .iter_mut()
                        .flat_map(|arg| match arg {
                            GenericArgument::Lifetime(_) => None,
                            GenericArgument::Type(ty) => {
                                strip_type_lifetime(ty);
                                Some(GenericArgument::Type(ty.clone()))
                            }
                            _ => Some(arg.clone()),
                        })
                        .collect()
                }
            }
        }
        _ => {}
    }
}
