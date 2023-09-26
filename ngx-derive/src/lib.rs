use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod module;

#[proc_macro_error]
#[proc_macro_derive(Module, attributes(module))]
pub fn derive_module(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let expanded = module::expand(input);

    expanded.into()
}
