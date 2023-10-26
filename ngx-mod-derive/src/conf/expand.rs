use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse_quote, Block, Data, DataStruct, DeriveInput, ExprStruct, Fields, FieldsNamed, Ident,
    ItemImpl,
};

use crate::{conf::r#struct::DefaultValue, extract, util::find_ngx_rt};

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

    let (field_names, directives) = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = data
    {
        (
            named
                .iter()
                .flat_map(|f| f.ident.clone())
                .collect::<Vec<_>>(),
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
                .collect::<Vec<_>>(),
        )
    } else {
        abort!(
            ident.span(),
            "conf attribute can only be applied to structure with named fields"
        )
    };

    let ngx_rt = find_ngx_rt();

    let impl_default: Option<ItemImpl> = struct_args.default_value().map(|v| {
        let block: Block = match v {
            DefaultValue::Unset => {
                let s: ExprStruct = parse_quote!{ #struct_name {
                    #( #field_names : #ngx_rt ::core::conf::unset(), )*
                } };
                parse_quote!{ {
                    #s
                } }
            }
            DefaultValue::Zeroed => {
                parse_quote!{ {
                    unsafe { ::std::mem::zeroed() }
                } }
            }
        };

        parse_quote! {
            impl #impl_generics ::std::default::Default for #struct_name #ty_generics #where_clause {
                fn default() -> Self #block
            }
        }
    });

    let n = directives.len();

    let impl_unsafe_conf: ItemImpl = parse_quote! {
        impl #impl_generics #ngx_rt ::core::UnsafeConf for #struct_name #ty_generics #where_clause {
            type Commands = [#ngx_rt ::ffi::ngx_command_t; #n];

            const COMMANDS: Self::Commands = [
                #( #directives ),*
            ];
        }
    };

    let impl_conf_ext: ItemImpl = parse_quote! {
        impl #impl_generics #ngx_rt ::core::ConfExt for #struct_name #ty_generics #where_clause {
            fn commands() -> #ngx_rt ::core::Cmds<'static> {
                #ngx_rt ::core::Cmds::from( & <Self as #ngx_rt ::core::UnsafeConf>::COMMANDS[.. #n - 1])
            }
        }
    };

    quote! {
        #impl_default
        #impl_unsafe_conf
        #impl_conf_ext
    }
}
