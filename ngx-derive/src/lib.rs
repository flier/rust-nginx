use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

#[cfg(feature = "native_callback")]
mod callback;
#[cfg(feature = "native_handler")]
mod handler;
#[cfg(feature = "module")]
mod module;

#[cfg(feature = "module")]
#[proc_macro_error]
#[proc_macro_derive(Module, attributes(module))]
pub fn derive_module(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);

    let expanded = module::expand(input);

    expanded.into()
}

#[cfg(feature = "native_handler")]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn native_handler(attr: TokenStream, handler: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as handler::Args);
    let handler = parse_macro_input!(handler as syn::ItemFn);

    let expanded = handler::expand(args, handler);

    expanded.into()
}

#[cfg(feature = "native_callback")]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn native_callback(attr: TokenStream, handler: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as callback::Args);
    let handler = parse_macro_input!(handler as syn::ItemType);

    let expanded = callback::expand(args, handler);

    expanded.into()
}
