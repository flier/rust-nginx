use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

mod callback;
mod handler;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn native_handler(attr: TokenStream, handler: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as handler::Args);
    let handler = parse_macro_input!(handler as syn::ItemFn);

    let expanded = handler::expand(args, handler);

    expanded.into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn native_callback(attr: TokenStream, handler: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as callback::Args);
    let handler = parse_macro_input!(handler as syn::ItemType);

    let expanded = callback::expand(args, handler);

    expanded.into()
}
