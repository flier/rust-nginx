use merge::Merge;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use structmeta::{Flag, NameArgs, NameValue, StructMeta};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    DeriveInput, Expr, ExprLit, ExprRange, Ident, Lit, LitInt, Path, RangeLimits, Token,
};

use crate::{extract, ffi};

#[derive(Clone, Debug, Default, Merge, StructMeta)]
struct Args {
    name: Option<NameValue<Ident>>,
    args: Option<NameArgs<Vec<IntOrRange>>>,
    #[struct_meta(name = "type")]
    ty: Option<NameValue<Path>>,
    #[merge(strategy = merge_flag)]
    block: Flag,
    #[merge(strategy = merge_flag)]
    flag: Flag,
}

fn merge_flag(left: &mut Flag, right: Flag) {
    if left.span.is_none() {
        left.span = right.span;
    }
}

#[derive(Clone, Debug)]
enum IntOrRange {
    Int(LitInt),
    Range(ExprRange),
}

impl IntOrRange {
    fn parse_args(&self) -> u32 {
        match self {
            IntOrRange::Int(lit) => lit.parse_args(),
            IntOrRange::Range(r) => r.parse_args(),
        }
    }
}

trait ParseArgs {
    fn parse_args(&self) -> u32;
}

impl ParseArgs for LitInt {
    fn parse_args(&self) -> u32 {
        self.parse_int().parse_args()
    }
}

impl ParseArgs for usize {
    fn parse_args(&self) -> u32 {
        if let Some(n) = ARGS.get(*self) {
            *n
        } else {
            abort_call_site!("`args` should be a number from 0 to 7")
        }
    }
}

impl ParseArgs for ExprRange {
    fn parse_args(&self) -> u32 {
        match (self.start.as_ref(), self.limits, self.end.as_ref()) {
            (Some(start), RangeLimits::HalfOpen(_), Some(end)) => {
                let start = start.parse_int();
                let end = end.parse_int();

                (start..end)
                    .map(|n| n.parse_args())
                    .fold(0, |acc, n| acc | n)
            }
            (Some(start), RangeLimits::Closed(_), Some(end)) => {
                let start = start.as_ref().parse_int();
                let end = end.as_ref().parse_int();

                (start..=end)
                    .map(|n| n.parse_args())
                    .fold(0, |acc, n| acc | n)
            }
            (Some(start), RangeLimits::HalfOpen(_), None) => {
                let start = start.as_ref().parse_int();

                (start..ffi::NGX_CONF_MAX_ARGS as usize)
                    .map(|n| n.parse_args())
                    .fold(0, |acc, n| acc | n)
            }
            (None, RangeLimits::HalfOpen(_), Some(end)) => {
                let end = end.as_ref().parse_int();

                (0..end).map(|n| n.parse_args()).fold(0, |acc, n| acc | n)
            }
            (None, RangeLimits::Closed(_), Some(end)) => {
                let end = end.as_ref().parse_int();

                (0..=end).map(|n| n.parse_args()).fold(0, |acc, n| acc | n)
            }
            _ => abort!(self.span(), "expect a range of integer"),
        }
    }
}

trait ParseInt {
    fn parse_int(&self) -> usize;
}

impl ParseInt for LitInt {
    fn parse_int(&self) -> usize {
        if let Ok(n) = self.base10_parse() {
            n
        } else {
            abort!(self.span(), "`args` should be a 10 based integer literal")
        }
    }
}

impl ParseInt for Expr {
    fn parse_int(&self) -> usize {
        if let Expr::Lit(ExprLit {
            lit: Lit::Int(lit), ..
        }) = self
        {
            lit.parse_int()
        } else {
            abort!(self.span(), "expect an integer literal")
        }
    }
}

impl Parse for IntOrRange {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek2(Token![..]) || input.peek2(Token![..=]) {
            input.parse().map(IntOrRange::Range)
        } else {
            input.parse().map(IntOrRange::Int)
        }
    }
}

const ARGS: [u32; 8] = [
    ffi::NGX_CONF_NOARGS,
    ffi::NGX_CONF_TAKE1,
    ffi::NGX_CONF_TAKE2,
    ffi::NGX_CONF_TAKE3,
    ffi::NGX_CONF_TAKE4,
    ffi::NGX_CONF_TAKE5,
    ffi::NGX_CONF_TAKE6,
    ffi::NGX_CONF_TAKE7,
];

impl Args {
    pub fn ty(&self) -> usize {
        let mut n = 0;

        n |= self.args.as_ref().map_or(ffi::NGX_CONF_NOARGS, |arg| {
            arg.args.iter().fold(0, |acc, lit| acc | lit.parse_args())
        });

        if self.block.span.is_some() {
            n |= ffi::NGX_CONF_BLOCK;
        }
        if self.flag.span.is_some() {
            n |= ffi::NGX_CONF_FLAG;
        }

        n as usize
    }
}

pub fn expand(input: DeriveInput) -> TokenStream {
    let (args, _) = extract::args::<Args, _>(input.attrs, "conf");

    quote! {}
}
