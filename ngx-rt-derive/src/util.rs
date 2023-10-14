use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::abort;
use syn::{parse_quote, Ident, Path};

pub fn find_crate_name() -> Path {
    match crate_name("ngx-rt") {
        Ok(FoundCrate::Itself) => {
            parse_quote! {
                crate
            }
        }
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());

            parse_quote! {
                #ident
            }
        }
        Err(_) => match crate_name("ngx-mod") {
            Ok(FoundCrate::Itself) => {
                parse_quote! {
                    crate :: rt
                }
            }
            Ok(FoundCrate::Name(name)) => {
                let ident = Ident::new(&name, Span::call_site());

                parse_quote! {
                    #ident :: rt
                }
            }
            Err(err) => {
                abort!(
                    "`ngx-rt` or `ngx-mod` should present in `Cargo.toml`, {}",
                    err
                )
            }
        },
    }
}
