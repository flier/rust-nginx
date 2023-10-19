use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote_spanned};
use structmeta::{NameValue, StructMeta};
use syn::{
    parse_quote_spanned, spanned::Spanned, AngleBracketedGenericArguments, BareFnArg, Expr,
    ExprCall, ExprLet, FnArg, GenericArgument, Ident, ItemType, PathArguments, ReturnType, Stmt,
    Type, TypeBareFn, TypePath, TypeReference, TypeTuple,
};

use crate::util::find_ngx_rt;

#[derive(Clone, Debug, StructMeta)]
pub struct Args {
    name: Option<NameValue<Ident>>,
    log: Option<NameValue<Expr>>,
}

pub fn expand(args: Args, item: ItemType) -> TokenStream {
    let span = item.span();
    let ItemType {
        attrs,
        vis,
        ident,
        generics,
        ty,
        ..
    } = item;
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let TypeBareFn { inputs, output, .. } = if let Type::BareFn(f) = ty.as_ref() {
        f
    } else {
        abort! { span, "only bare fn supported" }
    };

    let (unsafe_args, unsafe_params) = inputs
        .iter()
        .enumerate()
        .map(|(i, arg @ BareFnArg { attrs, name, ty })| {
            let arg_name = name
                .clone()
                .map_or_else(|| format_ident!("arg{}", i), |(name, _)| name);
            let (fn_arg, convert) = to_native_ty(&arg_name, ty).unzip();

            (
                fn_arg.map_or_else(
                    || {
                        parse_quote_spanned! { arg.span() =>
                            #( #attrs )*
                            #arg_name : #ty
                        }
                    },
                    |mut arg| {
                        if let FnArg::Typed(ref mut pt) = arg {
                            pt.attrs.extend(pt.attrs.clone());
                        }
                        arg
                    },
                ),
                (
                    convert.map(|expr| {
                        parse_quote_spanned! { arg.span() =>
                            #expr ;
                        }
                    }),
                    arg_name,
                ),
            )
        })
        .unzip::<FnArg, (Option<Stmt>, Ident), Vec<_>, Vec<_>>();
    let (unsafe_conversions, unsafe_params) = unsafe_params
        .into_iter()
        .unzip::<Option<Stmt>, Ident, Vec<_>, Vec<_>>();

    let call: ExprCall = parse_quote_spanned! { span =>
        self.0(
            #( #unsafe_params ,)*
        )
    };

    let name = args.name.as_ref().map_or(&ident, |arg| &arg.value);
    let ngx_rt = find_ngx_rt();

    let (result, result_ty) = if matches!(output, ReturnType::Default) {
        (Expr::Call(call), ReturnType::Default)
    } else {
        let result = if let Some((ok, err)) = extract_result_types(output) {
            let mut result = if let Some(log) = args.log.as_ref().map(|arg| &arg.value) {
                parse_quote_spanned! { output.span() =>
                    crate::AsResult::ok(#call).map_err(|err| {
                        #ngx_rt ::core::Logger::emerg (
                            #log,
                            format!(concat!("call `{}` failed, {}", stringify!(#name), err)));

                        err
                    })
                }
            } else {
                parse_quote_spanned! { output.span() =>
                    crate::AsResult::ok(#call)
                }
            };

            if is_empty_tuple(ok) {
                result = parse_quote_spanned! { ok.span() =>
                    #result .map(|_| ())
                }
            } else {
                result = parse_quote_spanned! { ok.span() =>
                    #result .map( ::std::convert::From::from )
                }
            }

            if is_empty_tuple(err) {
                result = parse_quote_spanned! { ok.span() =>
                    #result .map_err(|_| ())
                }
            } else {
                result = parse_quote_spanned! { ok.span() =>
                    #result .map_err( ::std::convert::From::from )
                }
            }

            result
        } else {
            parse_quote_spanned! { output.span() =>
                #call.into()
            }
        };
        let result_ty = parse_quote_spanned! { output.span() =>
            -> crate::ffi::ngx_int_t
        };

        (result, result_ty)
    };

    quote_spanned! { span =>
        #( #attrs )*
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq)]
        #vis struct #name (
            pub unsafe extern "C" fn(
                #( #unsafe_args ),*
            ) #result_ty
        );

        impl #ngx_rt ::NativeCallback for #name {
            type CType = unsafe extern "C" fn( #( #unsafe_args ),* ) #result_ty;
        }

        impl #name {
            pub fn call #ty_generics (&self, #inputs ) #output #where_clause {
                #( #unsafe_conversions )*

                unsafe {
                    #result
                }
            }
        }
    }
}

fn is_empty_tuple(ty: &Type) -> bool {
    if let Type::Tuple(TypeTuple { elems, .. }) = ty {
        elems.is_empty()
    } else {
        false
    }
}

fn extract_result_types(output: &ReturnType) -> Option<(&Type, &Type)> {
    match output {
        ReturnType::Type(_, ref ty) => match ty.as_ref() {
            Type::Path(TypePath { qself, path }) if qself.is_none() => path
                .segments
                .last()
                .filter(|seg| seg.ident == "Result")
                .and_then(|seg| match seg.arguments {
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        ref args,
                        ..
                    }) if args.len() == 2 => {
                        let mut iter = args.iter();

                        iter.next().zip(iter.next()).and_then(|types| match types {
                            (GenericArgument::Type(ok), GenericArgument::Type(err)) => {
                                Some((ok, err))
                            }
                            _ => None,
                        })
                    }
                    _ => None,
                }),
            _ => None,
        },
        _ => None,
    }
}

fn to_native_ty(arg_name: &Ident, ty: &Type) -> Option<(FnArg, ExprLet)> {
    match ty {
        Type::Reference(TypeReference {
            mutability, elem, ..
        }) => Some(if is_foreign_ty(elem.as_ref()) {
            cast_as_ptr(arg_name, elem)
        } else {
            cast_from_ref(arg_name, mutability.is_some(), elem)
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
                            cast_as_raw(arg_name, mutability, elem)
                        } else {
                            cast_from_option(arg_name, mutability, elem)
                        }
                    }),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn is_foreign_ty(ty: &Type) -> bool {
    matches!(ty, Type::Path(TypePath { qself, path })
        if qself.is_none()
            && path.segments.last().map_or(false, |seg| seg.ident.to_string().ends_with("Ref")))
}

fn cast_as_ptr(arg_name: &Ident, elem: &Type) -> (FnArg, ExprLet) {
    let raw_ty = parse_quote_spanned! { elem.span() =>
        #arg_name : * mut <#elem as crate::foreign_types::ForeignTypeRef>::CType
    };

    let convert = parse_quote_spanned! { elem.span() =>
        let #arg_name = <#elem as crate::foreign_types::ForeignTypeRef>::as_ptr( #arg_name )
    };

    (raw_ty, convert)
}

fn cast_as_raw(arg_name: &Ident, mutability: bool, elem: &Type) -> (FnArg, ExprLet) {
    let raw_ty = parse_quote_spanned! { elem.span() =>
        #arg_name : * mut <#elem as crate::foreign_types::ForeignTypeRef>::CType
    };

    let convert = if mutability {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = <#elem as crate::AsRawRef>::as_raw (#arg_name)
        }
    } else {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = <#elem as crate::AsRawMut>::as_raw_mut (#arg_name)
        }
    };

    (raw_ty, convert)
}

fn cast_from_ref(arg_name: &Ident, mutability: bool, elem: &Type) -> (FnArg, ExprLet) {
    let raw_ty = parse_quote_spanned! { elem.span() =>
        #arg_name : * mut ::std::ffi::c_void
    };

    let convert = if mutability {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = #arg_name as *mut #elem as *mut ::std::ffi::c_void
        }
    } else {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = #arg_name as *const #elem as *mut #elem as *mut ::std::ffi::c_void
        }
    };

    (raw_ty, convert)
}

fn cast_from_option(arg_name: &Ident, mutability: bool, elem: &Type) -> (FnArg, ExprLet) {
    let raw_ty = parse_quote_spanned! { elem.span() =>
        #arg_name : * mut ::std::ffi::c_void
    };

    let convert = if mutability {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = #arg_name.map_or_else(ptr::null_mut, |v| v as *mut #elem as *mut ::std::ffi::c_void)
        }
    } else {
        parse_quote_spanned! { elem.span() =>
            let #arg_name = #arg_name.map_or_else(ptr::null_mut, |v| v as *const #elem as *mut #elem as *mut ::std::ffi::c_void)
        }
    };

    (raw_ty, convert)
}
