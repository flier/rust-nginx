use std::ptr;

use foreign_types::ForeignTypeRef;

use crate::{
    core::{hash, list, ListRef},
    ffi,
};

pub type Header = hash::TableEltRef;

pub struct Headers<'a>(&'a mut ListRef<<hash::TableEltRef as ForeignTypeRef>::CType>);

impl<'a> Headers<'a> {
    pub fn add<K, V>(&mut self, key: K, value: V) -> Option<&Header>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let elt = {
            let key = key.as_ref();
            let value = value.as_ref();
            let lowcase_key = key.to_ascii_lowercase();
            let hash = hash::key(key);

            let key = self.0.pool().strdup(key)?;
            let value = self.0.pool().strdup(value)?;
            let lowcase_key = self.0.pool().strdup(lowcase_key)?;

            ffi::ngx_table_elt_t {
                hash,
                key: key.into(),
                value: value.into(),
                lowcase_key: lowcase_key.as_ptr() as *mut _,
                next: ptr::null_mut(),
            }
        };

        self.0
            .push(elt)
            .map(|elt| unsafe { hash::TableEltRef::from_ptr(elt as *mut _) })
    }

    pub fn set<K, V>(&mut self, key: K, value: V) -> Option<&Header>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key = key.as_ref();
        let value = value.as_ref();

        if self.contains(key) {
            let v = self.0.pool().strdup(value)?;

            let h = self.find_mut(key).expect("header");

            h.set_value(v);

            Some(h)
        } else {
            self.add(key, value)
        }
    }

    pub fn contains<Q>(&self, key: Q) -> bool
    where
        Q: AsRef<str>,
    {
        self.find(key).is_some()
    }

    pub fn find<Q>(&self, key: Q) -> Option<&Header>
    where
        Q: AsRef<str>,
    {
        let key = key.as_ref();
        let hash = hash::key(key);

        self.iter().find(|h| {
            h.hash() == hash
                && h.key().map_or(false, |k| k.len() == key.len())
                && h.lowcase_key().map_or(false, |k| k == key)
        })
    }

    pub fn find_mut<Q>(&mut self, key: Q) -> Option<&mut Header>
    where
        Q: AsRef<str>,
    {
        let key = key.as_ref();
        let hash = hash::key(key);

        self.iter_mut().find(|h| {
            h.hash() == hash
                && h.key().map_or(false, |k| k.len() == key.len())
                && h.lowcase_key().map_or(false, |k| k == key)
        })
    }

    pub fn iter(&self) -> Iter {
        Iter(self.0.iter())
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut(self.0.iter())
    }
}

impl<'a> From<&'a mut ListRef<<hash::TableEltRef as ForeignTypeRef>::CType>> for Headers<'a> {
    fn from(p: &'a mut ListRef<<hash::TableEltRef as ForeignTypeRef>::CType>) -> Self {
        Headers(p)
    }
}

impl<'a> IntoIterator for Headers<'a> {
    type Item = &'a Header;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.0.into_iter())
    }
}

pub struct Iter<'a>(list::Iter<'a, <hash::TableEltRef as ForeignTypeRef>::CType>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Header;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|p| unsafe { hash::TableEltRef::from_ptr(p as *const _ as *mut _) })
    }
}

pub struct IterMut<'a>(list::Iter<'a, <hash::TableEltRef as ForeignTypeRef>::CType>);

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Header;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|p| unsafe { hash::TableEltRef::from_ptr_mut(p as *const _ as *mut _) })
    }
}
