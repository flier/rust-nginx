use case::CaseExt;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Ident};

use crate::extract;

use super::{Directive, FieldArgs, StructArgs};

pub fn expand(input: DeriveInput) -> TokenStream {
    let DeriveInput {
        attrs, ident, data, ..
    } = input;
    let (args, _) = extract::args::<StructArgs, _>(attrs, "conf");
    let struct_args = args.unwrap_or_default();
    let struct_name: &Ident = &ident;

    let base_name = {
        let mut s = struct_args
            .name
            .as_ref()
            .map(|n| n.value.to_string())
            .unwrap_or_else(|| ident.to_string())
            .to_snake();

        if !s.starts_with("ngx_") {
            s = format!("ngx_{}", s);
        }

        s
    };
    let cmds_name = {
        let mut s = base_name.clone();

        if !s.ends_with("_cmds") {
            s += "_cmds";
        }

        Ident::new(&s, Span::call_site())
    };

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

    quote! {
        #[no_mangle]
        static mut #cmds_name : [::ngx_mod::rt::ffi::ngx_command_t; #n] = [
            #( #directives ),*
        ];
    }
}
