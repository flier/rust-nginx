use foreign_types::ForeignTypeRef;

use crate::core::{hash, list, Str};

#[repr(transparent)]
pub struct Header<'a>(&'a hash::TableEltRef);

impl<'a> From<&'a hash::TableEltRef> for Header<'a> {
    fn from(p: &'a hash::TableEltRef) -> Self {
        Header(p)
    }
}

impl<'a> Header<'a> {
    pub fn key(&self) -> Option<&Str> {
        self.0.key()
    }

    pub fn value(&self) -> Option<&Str> {
        self.0.value()
    }

    pub fn lowcase_key(&self) -> Option<&str> {
        self.0.lowcase_key()
    }
}

pub struct Headers<'a>(pub(crate) list::Iter<'a, <hash::TableEltRef as ForeignTypeRef>::CType>);

impl<'a> Iterator for Headers<'a> {
    type Item = Header<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|p| unsafe { Header(hash::TableEltRef::from_ptr(p as *const _ as *mut _)) })
    }
}
