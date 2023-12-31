use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

mod conf;
mod extract;
mod module;
mod util;

#[proc_macro_error]
#[proc_macro_derive(Module, attributes(module))]
pub fn derive_module(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);

    let expanded = module::expand(input);

    expanded.into()
}

#[proc_macro_error]
#[proc_macro_derive(Conf, attributes(conf, directive))]
pub fn derive_conf(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);

    let expanded = conf::expand(input);

    expanded.into()
}
