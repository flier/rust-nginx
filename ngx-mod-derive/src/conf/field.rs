use cfg_if::cfg_if;
use merge::Merge;
use proc_macro_error::abort;
use quote::quote;
use structmeta::{Flag, NameArgs, NameValue, StructMeta};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Expr, ExprLit, ExprRange, Lit, LitInt, LitStr, Path, RangeLimits, Token,
};

use super::{Args, Offset};

#[derive(Clone, Debug, Default, Merge, StructMeta)]
pub struct FieldArgs {
    pub conf: Option<NameValue<Path>>,
    pub name: Option<NameValue<LitStr>>,
    pub args: Option<NameArgs<Vec<Arg>>>,
    #[merge(strategy = merge_flag)]
    pub block: Flag,
    #[merge(strategy = merge_flag)]
    pub flag: Flag,
    pub set: Option<NameValue<Path>>,
}

fn merge_flag(left: &mut Flag, right: Flag) {
    if left.span.is_none() {
        left.span = right.span;
    }
}

impl FieldArgs {
    pub fn conf_offset(&self) -> Option<Offset> {
        use Offset::*;

        self.conf.as_ref().map(|arg| &arg.value).map(|p| {
            let name = quote! { #p }.to_string().to_lowercase();

            if name.starts_with("http") {
                cfg_if! {
                    if #[cfg(feature = "http")] {
                        match name.as_str() {
                            "http" | "http :: main" => HttpMain,
                            "http :: srv" | "http :: server" => HttpServer,
                            "http :: loc" | "http :: location" => HttpLocation,
                            _ => abort!(p.span(), "unknown directive type")
                        }
                    } else {
                        abort!(p.span(), "`http` support is disabled");
                    }
                }
            } else if name.starts_with("stream") {
                cfg_if! {
                    if #[cfg(feature = "stream")] {
                        match name.as_str() {
                            "stream" | "stream :: main" => StreamMain,
                            "stream :: srv" | "stream :: server" => StreamServer,
                            _ => abort!(p.span(), "unknown directive type"),
                        }
                    } else {
                        abort!(p.span(), "`stream` support is disabled");
                    }
                }
            } else if name.starts_with("mail") {
                cfg_if! {
                    if #[cfg(feature = "mail")] {
                        match name.as_str() {
                            "mail" | "mail :: main" => MailMain,
                            "mail :: srv" | "mail :: server" => MailServer,
                            _ => abort!(p.span(), "unknown directive type"),
                        }
                    } else {
                        abort!(p.span(), "`mail` support is disabled");
                    }
                }
            } else {
                abort!(p.span(), "unknown directive type")
            }
        })
    }

    pub fn args(&self) -> Vec<Args> {
        let mut args = self.args.as_ref().map_or_else(
            || vec![Args::None],
            |arg| {
                arg.args
                    .iter()
                    .flat_map(|lit| lit.parse_args())
                    .collect::<Vec<_>>()
            },
        );

        if self.block.span.is_some() {
            args.push(Args::Block);
        }

        if self.flag.span.is_some() {
            args.push(Args::Flag);
        }

        args
    }
}

#[derive(Clone, Debug)]
pub enum Arg {
    Int(LitInt),
    Range(ExprRange),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek2(Token![..]) || input.peek2(Token![..=]) {
            input.parse().map(Arg::Range)
        } else {
            input.parse().map(Arg::Int)
        }
    }
}

impl Arg {
    fn parse_args(&self) -> Vec<Args> {
        match self {
            Arg::Int(lit) => lit.parse_args(),
            Arg::Range(r) => r.parse_args(),
        }
    }
}

const MAX_ARGS: usize = 8;

trait ParseArgs {
    fn parse_args(&self) -> Vec<Args>;
}

impl ParseArgs for LitInt {
    fn parse_args(&self) -> Vec<Args> {
        vec![self.parse_int().into()]
    }
}

impl ParseArgs for ExprRange {
    fn parse_args(&self) -> Vec<Args> {
        match (self.start.as_ref(), self.limits, self.end.as_ref()) {
            (Some(start), RangeLimits::HalfOpen(_), Some(end)) => {
                let start = start.parse_int();
                let end = end.parse_int();

                (start..end).map(|n| n.into()).collect()
            }
            (Some(start), RangeLimits::Closed(_), Some(end)) => {
                let start = start.as_ref().parse_int();
                let end = end.as_ref().parse_int();

                (start..=end).map(|n| n.into()).collect()
            }
            (Some(start), RangeLimits::HalfOpen(_), None) => {
                let start = start.as_ref().parse_int();

                (start..MAX_ARGS).map(|n| n.into()).collect()
            }
            (None, RangeLimits::HalfOpen(_), Some(end)) => {
                let end = end.as_ref().parse_int();

                (0..end).map(|n| n.into()).collect()
            }
            (None, RangeLimits::Closed(_), Some(end)) => {
                let end = end.as_ref().parse_int();

                (0..=end).map(|n| n.into()).collect()
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
