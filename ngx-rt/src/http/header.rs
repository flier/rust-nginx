use std::ptr;

use foreign_types::ForeignTypeRef;

use crate::{
    core::{hash, list, ListRef, Str},
    ffi,
};

#[repr(transparent)]
pub struct Header<'a>(&'a hash::TableEltRef);

impl<'a> From<&'a hash::TableEltRef> for Header<'a> {
    fn from(p: &'a hash::TableEltRef) -> Self {
        Header(p)
    }
}

impl<'a> Header<'a> {
    pub fn key(&self) -> Option<Str> {
        self.0.key()
    }

    pub fn value(&self) -> Option<Str> {
        self.0.value()
    }

    pub fn lowcase_key(&self) -> Option<&str> {
        self.0.lowcase_key()
    }
}

pub struct Headers<'a>(&'a mut ListRef<<hash::TableEltRef as ForeignTypeRef>::CType>);

impl<'a> Headers<'a> {
    pub fn add<K, V>(&mut self, key: K, value: V) -> Option<Header>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let elt = {
            let key = key.as_ref();
            let value = value.as_ref();
            let lowcase_key = key.to_ascii_lowercase();

            let key = self.0.pool().strdup(key)?;
            let value = self.0.pool().strdup(value)?;
            let lowcase_key = self.0.pool().strdup(lowcase_key)?;

            ffi::ngx_table_elt_t {
                hash: 0,
                key: key.into(),
                value: value.into(),
                lowcase_key: lowcase_key.as_ptr() as *mut _,
                next: ptr::null_mut(),
            }
        };

        self.0
            .push(elt)
            .map(|elt| Header(unsafe { hash::TableEltRef::from_ptr(elt as *mut _) }))
    }
}

impl<'a> From<&'a mut ListRef<<hash::TableEltRef as ForeignTypeRef>::CType>> for Headers<'a> {
    fn from(p: &'a mut ListRef<<hash::TableEltRef as ForeignTypeRef>::CType>) -> Self {
        Headers(p)
    }
}

impl<'a> IntoIterator for Headers<'a> {
    type Item = Header<'a>;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.0.into_iter())
    }
}

pub struct Iter<'a>(list::Iter<'a, <hash::TableEltRef as ForeignTypeRef>::CType>);

impl<'a> Iterator for Iter<'a> {
    type Item = Header<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|p| unsafe { Header(hash::TableEltRef::from_ptr(p as *const _ as *mut _)) })
    }
}
