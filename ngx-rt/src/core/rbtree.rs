use std::mem::{zeroed, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use foreign_types::{foreign_type, ForeignTypeRef};
use ngx_rt_derive::native_callback;

use crate::{callback, ffi, never_drop, property, AsRawMut, AsRawRef};

pub type Key = usize;

foreign_type! {
    pub unsafe type Tree: Send {
        type CType = ffi::ngx_rbtree_t;

        fn drop = never_drop::<ffi::ngx_rbtree_t>;
    }
}

impl TreeRef {
    property! {
        root: &NodeRef;
        sentinel: &NodeRef;
    }

    callback! {
        insert: InsertFn;
    }

    pub fn is_empty(&self) -> bool {
        unsafe {
            let r = self.as_raw();

            r.root == r.sentinel
        }
    }

    pub fn nodes(&self) -> Iter {
        let node = self.root().min(self.sentinel());

        Iter { tree: self, node }
    }

    pub fn insert_node(&mut self, node: &mut NodeRef) {
        unsafe {
            ffi::ngx_rbtree_insert(self.as_ptr(), node.as_ptr());
        }
    }

    pub fn delete_node(&mut self, node: &mut NodeRef) {
        unsafe {
            ffi::ngx_rbtree_delete(self.as_ptr(), node.as_ptr());
        }
    }

    pub fn next_node(&self, node: &NodeRef) -> Option<&NodeRef> {
        unsafe {
            let next = ffi::ngx_rbtree_next(self.as_ptr(), node.as_ptr());

            NonNull::new(next).map(|p| NodeRef::from_ptr(p.as_ptr()))
        }
    }
}

pub fn init<'a>(
    tree: &'a mut MaybeUninit<<TreeRef as ForeignTypeRef>::CType>,
    sentinel: &'a mut NodeRef,
    insert: RawInsertFn,
) -> &'a mut TreeRef {
    sentinel.set_black();

    unsafe {
        TreeRef::from_ptr_mut(tree.write(ffi::ngx_rbtree_t {
            root: sentinel.as_ptr(),
            sentinel: sentinel.as_ptr(),
            insert: Some(insert),
        }) as *mut _)
    }
}

pub type RawInsertFn = unsafe extern "C" fn(
    root: *mut ffi::ngx_rbtree_node_t,
    node: *mut ffi::ngx_rbtree_node_t,
    sentinel: *mut ffi::ngx_rbtree_node_t,
);

#[native_callback]
pub type InsertFn = fn(root: &mut NodeRef, node: &NodeRef, sentinel: &NodeRef);

impl<'a> IntoIterator for &'a TreeRef {
    type Item = &'a NodeRef;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes()
    }
}

pub struct Iter<'a> {
    tree: &'a TreeRef,
    node: Option<&'a NodeRef>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a NodeRef;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node.take() {
            self.node = self.tree.next_node(node);
            Some(node)
        } else {
            None
        }
    }
}

foreign_type! {
    pub unsafe type Node: Send {
        type CType = ffi::ngx_rbtree_node_t;

        fn drop = never_drop::<ffi::ngx_rbtree_node_t>;
    }
}

impl NodeRef {
    pub const RED: u8 = 1;
    pub const BLACK: u8 = 0;

    property! {
        key: Key { get; set; };
        left as &NodeRef;
        right as &NodeRef;
        parent as &NodeRef;
    }

    pub fn set_red(&mut self) {
        unsafe {
            self.as_raw_mut().color = Self::RED;
        }
    }

    pub fn set_black(&mut self) {
        unsafe {
            self.as_raw_mut().color = Self::BLACK;
        }
    }

    pub fn is_red(&self) -> bool {
        unsafe { self.as_raw().color != 0 }
    }

    pub fn is_black(&self) -> bool {
        !self.is_red()
    }

    pub fn min(&self, sentinel: &NodeRef) -> Option<&NodeRef> {
        unsafe {
            let mut node = self.as_ptr();
            let sentinel = sentinel.as_ptr();

            while !node.is_null() {
                if let Some(next) = node.as_ref().map(|p| p.left) {
                    if next == sentinel {
                        break;
                    }

                    node = next;
                }
            }

            NonNull::new(node).map(|p| NodeRef::from_ptr(p.as_ptr()))
        }
    }
}

pub fn sentinel() -> Sentinel {
    Sentinel::default()
}

#[repr(transparent)]
pub struct Sentinel(ffi::ngx_rbtree_node_t);

impl Default for Sentinel {
    fn default() -> Self {
        let mut node = unsafe { zeroed::<ffi::ngx_rbtree_node_t>() };

        node.color = NodeRef::BLACK;

        Self(node)
    }
}

impl Deref for Sentinel {
    type Target = NodeRef;

    fn deref(&self) -> &Self::Target {
        unsafe { NodeRef::from_ptr(&self.0 as *const _ as *mut _) }
    }
}

impl DerefMut for Sentinel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { NodeRef::from_ptr_mut(&mut self.0 as *mut _) }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{zeroed, MaybeUninit};

    use crate::native_handler;

    use super::*;

    #[test]
    fn rbtree() {
        let mut t = MaybeUninit::uninit();
        let mut s = sentinel();

        let tree = init(&mut t, &mut s, nbtree_insert_value);

        assert!(tree.is_empty());
        assert!(tree.nodes().collect::<Vec<_>>().is_empty());
        assert!(tree.sentinel().is_black());

        let mut n = unsafe { zeroed() };
        let node = unsafe { NodeRef::from_ptr_mut(&mut n as *mut _) };

        tree.insert_node(node);

        assert!(!tree.is_empty());

        let nodes = tree.nodes().collect::<Vec<_>>();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes.first().unwrap().as_ptr(), node.as_ptr());

        tree.delete_node(node);

        assert!(tree.is_empty());
    }

    #[native_handler(name = nbtree_insert_value)]
    fn insert_value(root: &mut NodeRef, node: &NodeRef, sentinel: &NodeRef) {
        assert_ne!(root.as_ptr(), node.as_ptr());
        assert_ne!(root.as_ptr(), sentinel.as_ptr());
        assert_ne!(node.as_ptr(), sentinel.as_ptr());
    }
}
