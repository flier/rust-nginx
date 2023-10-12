use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use structmeta::{NameArgs, NameValue, StructMeta};
use syn::{
    parse_quote, parse_quote_spanned,
    spanned::Spanned,
    Expr, ExprCall, ExprLet,
    FnArg::{self, Receiver},
    GenericArgument, Ident, ItemFn, LitBool, Pat, PathArguments, ReturnType, Signature, Stmt, Type,
    TypePath, TypeReference,
};

#[derive(Clone, Debug, StructMeta)]
pub struct Args {
    name: Option<NameValue<Ident>>,
    log: Option<NameValue<Expr>>,
    embedded: Option<NameArgs<Option<LitBool>>>,
}

impl Args {
    pub fn log(&self) -> Option<&Expr> {
        self.log.as_ref().map(|arg| &arg.value)
    }

    pub fn embedded(&self) -> bool {
        self.embedded
            .as_ref()
            .map_or(!cfg!(debug_assertions), |arg| {
                arg.args.as_ref().map_or(true, |b| b.value)
            })
    }
}

pub enum Style {
    Handler,
    Setter,
}

pub fn expand(args: Args, f: ItemFn, style: Style) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = f;
    let Signature {
        ident,
        generics,
        inputs,
        output,
        ..
    } = sig;
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let name = args.name.as_ref().map_or(&ident, |arg| &arg.value);
    let (unsafe_args, unsafe_params) = inputs
        .iter()
        .flat_map(|arg: &syn::FnArg| match arg {
            syn::FnArg::Typed(ty) => Some(ty),
            Receiver(_) => None,
        })
        .map(|pt| {
            if let Pat::Ident(pi) = pt.pat.as_ref() {
                let arg_name = &pi.ident;
                let ty = pt.ty.as_ref();
                let (fn_arg, convert) = from_native_ty(arg_name, ty).unzip();

                (
                    fn_arg.map_or_else(
                        || FnArg::Typed(pt.clone()),
                        |mut arg| {
                            if let FnArg::Typed(ref mut pt) = arg {
                                pt.attrs.extend(pt.attrs.clone());
                            }
                            arg
                        },
                    ),
                    (
                        convert.map(|expr| {
                            parse_quote_spanned! { pt.span() =>
                                #expr ;
                            }
                        }),
                        arg_name,
                    ),
                )
            } else {
                abort!(
                    pt.span(),
                    "only support ident pattern in function argument, {:?}",
                    pt
                )
            }
        })
        .unzip::<FnArg, (Option<Stmt>, &Ident), Vec<_>, Vec<_>>();
    let (unsafe_conversions, unsafe_params) = unsafe_params
        .into_iter()
        .unzip::<Option<Stmt>, &Ident, Vec<_>, Vec<_>>();

    let handler: ExprCall = if args.embedded() {
        parse_quote_spanned! { ident.span() =>
            handler ( #( #unsafe_params ),* )
        }
    } else {
        parse_quote_spanned! { ident.span() =>
            #ident ( #( #unsafe_params ),* )
        }
    };

    let (result, result_ty) = if matches!(output, ReturnType::Default) {
        (Expr::Call(handler), ReturnType::Default)
    } else {
        match style {
            Style::Handler => (
                if is_result(&output) {
                    if let Some(log) = args.log() {
                        parse_quote_spanned! { output.span() =>
                            match #handler {
                                Ok(ok) => { ::ngx_mod::rt::RawOk::<::ngx_mod::rt::ffi::ngx_int_t>::raw_ok(ok) }
                                Err(err) => {
                                    ::ngx_mod::rt::core::Logger::emerg (
                                        #log,
                                        format!("call `{}` failed, {}", stringify!(#ident), err) );

                                    ::ngx_mod::rt::RawErr::<::ngx_mod::rt::ffi::ngx_int_t>::raw_err(())
                                }
                            }
                        }
                    } else {
                        parse_quote_spanned! { output.span() =>
                            ::ngx_mod::rt::RawResult::<::ngx_mod::rt::ffi::ngx_int_t>::raw_result(#handler)
                        }
                    }
                } else {
                    parse_quote_spanned! { output.span() =>
                        isize::from(#handler)
                    }
                },
                parse_quote_spanned! { output.span() =>
                    -> ::ngx_mod::rt::ffi::ngx_int_t
                },
            ),
            Style::Setter => (
                if is_result(&output) {
                    if let Some(log) = args.log() {
                        parse_quote_spanned! { output.span() =>
                            match #handler {
                                Ok(ok) => { ::ngx_mod::rt::RawOk::<*mut ::std::ffi::c_char>::raw_ok(ok) }
                                Err(err) => {
                                    // let log = ::std::convert::AsRef::<::ngx_mod::rt::core::LogRef>::as_ref( #log );

                                    ::ngx_mod::rt::core::Logger::emerg (
                                        #log,
                                        format!("call `{}` failed, {}", stringify!(#ident), err) );

                                    ::ngx_mod::rt::RawErr::<*mut ::std::ffi::c_char>::raw_err(())
                                }
                            }
                        }
                    } else {
                        parse_quote_spanned! { output.span() =>
                            ::ngx_mod::rt::RawResult::<*mut ::std::ffi::c_char>::raw_result(#handler)
                        }
                    }
                } else {
                    parse_quote_spanned! { output.span() =>
                        #handler as *mut _
                    }
                },
                parse_quote_spanned! { output.span() =>
                    -> *mut ::std::ffi::c_char
                },
            ),
        }
    };

    let (safe_handler, unsafe_handler): (Option<ItemFn>, Option<ItemFn>) = if args.embedded() {
        (
            None,
            Some(parse_quote! {
                fn handler #ty_generics ( #inputs ) #output #where_clause #block
            }),
        )
    } else {
        (
            Some(parse_quote! {
                #vis fn #ident #ty_generics ( #inputs ) #output #where_clause #block
            }),
            None,
        )
    };

    quote! {
        #safe_handler

        #( #attrs )*
        #[no_mangle]
        #vis unsafe extern "C" fn #name #ty_generics ( #( #unsafe_args ),* ) #result_ty #where_clause {
            #unsafe_handler

            #( #unsafe_conversions )*

            #result
        }
    }
}

fn from_native_ty(arg_name: &Ident, ty: &Type) -> Option<(FnArg, ExprLet)> {
    match ty {
        Type::Reference(TypeReference {
            mutability, elem, ..
        }) => Some(if is_foreign_ty(elem.as_ref()) {
            cast_from_ptr(arg_name, mutability.is_some(), elem)
        } else {
            cast_as_ref(arg_name, mutability.is_some(), elem)
        }),
        Type::Path(TypePath { qself, path }) if qself.is_none() => match path.segments.last() {
            Some(seg) if seg.ident == "Option" => match seg.arguments {
                PathArguments::AngleBracketed(ref args) if args.args.len() == 1 => args
                    .args
                    .first()
                    .and_then(|arg| {
                        if let GenericArgument::Type(Type::Reference(TypeReference {
                            mutability,
                            elem,
                            ..
                        })) = arg
                        {
                            Some((mutability.is_some(), elem))
                        } else {
                            None
                        }
                    })
                    .map(|(mutability, elem)| {
                        if is_foreign_ty(elem.as_ref()) {
                            cast_from_raw(arg_name, mutability, elem)
                        } else {
                            cast_as_option(arg_name, mutability, elem)
                        }
                    }),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn is_result(output: &ReturnType) -> bool {
    matches!(output, ReturnType::Type(_, ref ty)
        if matches!(ty.as_ref(), Type::Path(TypePath { qself, path })
            if qself.is_none() && path.segments.last().as_ref().map_or(false, |s| s.ident == "Result" )))
}

fn is_foreign_ty(ty: &Type) -> bool {
    matches!(ty, Type::Path(TypePath { qself, path })
        if qself.is_none()
            && path.segments.last().map_or(false, |seg| seg.ident.to_string().ends_with("Ref")))
}

fn cast_from_ptr(arg_name: &Ident, mutability: bool, elem: &Type) -> (FnArg, ExprLet) {
    let raw_ty = parse_quote_spanned! { elem.span() =>
        #arg_name : * mut <#elem as ::ngx_mod::rt::foreign_types::ForeignTypeRef>::CType
    };

    let convert = if mutability {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = <#elem as ::ngx_mod::rt::foreign_types::ForeignTypeRef>::from_ptr_mut (#arg_name)
        }
    } else {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = <#elem as ::ngx_mod::rt::foreign_types::ForeignTypeRef>::from_ptr (#arg_name)
        }
    };

    (raw_ty, convert)
}

fn cast_from_raw(arg_name: &Ident, mutability: bool, elem: &Type) -> (FnArg, ExprLet) {
    let raw_ty = parse_quote_spanned! { elem.span() =>
        #arg_name : * mut <#elem as ::ngx_mod::rt::foreign_types::ForeignTypeRef>::CType
    };

    let convert = if mutability {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = <#elem as ::ngx_mod::rt::FromRawMut>::from_raw_mut (#arg_name)
        }
    } else {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = <#elem as ::ngx_mod::rt::FromRawRef>::from_raw (#arg_name)
        }
    };

    (raw_ty, convert)
}

fn cast_as_ref(arg_name: &Ident, mutability: bool, elem: &Type) -> (FnArg, ExprLet) {
    let raw_ty = parse_quote_spanned! { elem.span() =>
        #arg_name : * mut ::std::ffi::c_void
    };

    let convert = if mutability {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = #arg_name.cast::< #elem >().as_mut().expect(stringify!(#arg_name))
        }
    } else {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = #arg_name.cast::< #elem >().as_ref().expect(stringify!(#arg_name))
        }
    };

    (raw_ty, convert)
}

fn cast_as_option(arg_name: &Ident, mutability: bool, elem: &Type) -> (FnArg, ExprLet) {
    let raw_ty = parse_quote_spanned! { elem.span() =>
        #arg_name : * mut ::std::ffi::c_void
    };

    let convert = if mutability {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = #arg_name.cast::< #elem >().as_mut()
        }
    } else {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = #arg_name.cast::< #elem >().as_ref()
        }
    };

    (raw_ty, convert)
}
