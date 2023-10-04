use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use structmeta::{NameValue, StructMeta};
use syn::{
    parse_quote_spanned,
    spanned::Spanned,
    Expr, ExprCall, ExprLet,
    FnArg::{self, Receiver},
    GenericArgument, Ident, ItemFn, Pat, PathArguments, ReturnType, Signature, Stmt, Type,
    TypePath, TypeReference,
};

#[derive(Clone, Debug, StructMeta)]
pub struct Args {
    name: Option<NameValue<Ident>>,
    log_err: Option<NameValue<Expr>>,
}

pub fn expand(args: Args, f: ItemFn) -> TokenStream {
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

    let handler: ExprCall = parse_quote_spanned! { ident.span() =>
        handler( #( #unsafe_params ),* )
    };

    let (result, result_ty) = if matches!(output, ReturnType::Default) {
        (Expr::Call(handler), ReturnType::Default)
    } else {
        (
            if is_result(&output) {
                if let Some(log_err) = args.log_err.as_ref().map(|arg| &arg.value) {
                    parse_quote_spanned! { output.span() =>
                        match #handler {
                            Ok(_) => { ::ngx_mod::rt::ffi::NGX_OK as isize }
                            Err(err) => {
                                #log_err (err.to_string().as_str());

                                ::ngx_mod::rt::ffi::NGX_ERROR as isize
                            }
                        }
                    }
                } else {
                    parse_quote_spanned! { output.span() =>
                        match #handler {
                            Ok(_) => { ::ngx_mod::rt::ffi::NGX_OK as isize }
                            Err(_) => { ::ngx_mod::rt::ffi::NGX_ERROR as isize }
                        }
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
        )
    };

    quote! {
        #( #attrs )*
        #[no_mangle]
        #vis unsafe extern "C" fn #name #ty_generics ( #( #unsafe_args ),* ) #result_ty #where_clause {
            fn handler #ty_generics ( #inputs ) #output #where_clause #block

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
