use alloc::collections::{btree_set::Iter, BTreeSet};
use core::ops::Deref;

use crate::util::owner::Owner;

pub struct SharedSet<T: Ord + Clone>(BTreeSet<T>);

impl<T: Ord + Clone> SharedSet<T> {
    #[inline]
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    #[inline]
    pub fn iter(&self) -> Iter<T> {
        self.0.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn insert<T: Ord + Clone, O: Owner<SharedSet<T>>>(
    owner: O,
    value: T,
) -> Option<SharedSetHandle<T, O>> {
    if owner.with(|set| set.0.insert(value.clone()))? {
        Some(SharedSetHandle { owner, value })
    } else {
        None
    }
}

pub struct SharedSetHandle<T: Ord + Clone, O: Owner<SharedSet<T>>> {
    owner: O,
    value: T,
}

impl<T: Ord + Clone, O: Owner<SharedSet<T>>> SharedSetHandle<T, O> {
    #[inline]
    pub fn owner(&self) -> &O {
        &self.owner
    }
}

impl<T: Ord + Clone, O: Owner<SharedSet<T>>> Deref for SharedSetHandle<T, O> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Ord + Clone, O: Owner<SharedSet<T>>> Drop for SharedSetHandle<T, O> {
    #[inline]
    fn drop(&mut self) {
        self.owner.with(|set| set.0.remove(&self.value));
    }
}
