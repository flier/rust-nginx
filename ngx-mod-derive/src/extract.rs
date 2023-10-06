use proc_macro_error::abort;
use syn::{parse::Parse, spanned::Spanned, Attribute};

const WELL_KNOWN_ATTRS: &[&str] = &["allow", "deny", "doc", "cfg"];

pub fn args<T, I>(attrs: I, name: &str) -> (Option<T>, Vec<Attribute>)
where
    T: Default + Parse + merge::Merge,
    I: IntoIterator<Item = Attribute>,
{
    let (args, attrs): (Vec<_>, Vec<_>) = attrs.into_iter().partition(|f| f.path().is_ident(name));

    let args = if args.is_empty() {
        None
    } else {
        Some(
            args.into_iter()
                .map(|attr| match attr.parse_args::<T>() {
                    Ok(arg) => arg,
                    Err(err) => abort!(attr.span(), "fail to parse args, {}", err),
                })
                .fold(T::default(), |mut args, arg| {
                    args.merge(arg);
                    args
                }),
        )
    };
    let attrs = attrs
        .into_iter()
        .filter(|attr| {
            WELL_KNOWN_ATTRS
                .iter()
                .any(|name| attr.path().is_ident(name))
        })
        .collect();

    (args, attrs)
}
