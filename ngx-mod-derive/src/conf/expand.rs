use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Ident};

use crate::{
    extract,
    util::{find_ngx_mod, find_ngx_rt},
};

use super::{Directive, FieldArgs, StructArgs};

pub fn expand(input: DeriveInput) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = input;
    let (args, _) = extract::args::<StructArgs, _>(attrs, "conf");
    let struct_args = args.unwrap_or_default();
    let struct_name: &Ident = &ident;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let directives = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = data
    {
        named
            .into_iter()
            .filter_map(|f| {
                let syn::Field {
                    attrs, ident, ty, ..
                } = f;
                let (args, _) = extract::args::<FieldArgs, _>(attrs, "directive");

                args.map(|args| Directive {
                    struct_args: &struct_args,
                    struct_name,
                    args,
                    name: ident.expect("name"),
                    ty,
                })
            })
            .collect::<Vec<_>>()
    } else {
        abort!(
            ident.span(),
            "conf attribute can only be applied to structure with named fields"
        )
    };

    let n = directives.len();
    let ngx_rt = find_ngx_rt();
    let ngx_mod = find_ngx_mod();

    quote! {
        impl #impl_generics #ngx_mod ::UnsafeConf for #struct_name #ty_generics #where_clause {
            type T = [#ngx_rt ::ffi::ngx_command_t; #n];

            const COMMANDS: [#ngx_rt ::ffi::ngx_command_t; #n] = [
                #( #directives ),*
            ];
        }
    }
}
