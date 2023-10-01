use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod handler;
mod module;

#[proc_macro_error]
#[proc_macro_derive(Module, attributes(module))]
pub fn derive_module(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let expanded = module::expand(input);

    expanded.into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn native_handler(attr: TokenStream, handler: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as handler::Args);
    let handler = parse_macro_input!(handler as syn::ItemFn);

    let expanded = handler::expand(args, handler);

    expanded.into()
}
