use merge::Merge;
use proc_macro_error::abort;
use structmeta::{NameValue, StructMeta};
use syn::Ident;

use super::Scope;

#[derive(Clone, Debug, Default, Merge, StructMeta)]
pub struct StructArgs {
    #[struct_meta(unnamed)]
    pub scope: Option<Scope>,
    pub name: Option<NameValue<Ident>>,
    pub default: Option<NameValue<Ident>>,
}

impl StructArgs {
    pub fn default_value(&self) -> Option<DefaultValue> {
        self.default.as_ref().map(|arg| &arg.value).map(|v| {
            if v == "unset" {
                DefaultValue::Unset
            } else if v == "zeroed" {
                DefaultValue::Zeroed
            } else {
                abort! {v.span(), "unknown default value, should be `unset` or `zeroed`"}
            }
        })
    }
}

pub enum DefaultValue {
    Unset,
    Zeroed,
}
