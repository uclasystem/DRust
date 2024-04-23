use std::{
    alloc::{Allocator, Layout},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    sync::Arc,
    thread,
};

use crate::{
    dprintln,
    drust_std::alloc::{ddeallocate, ddrop, LOCAL_ALLOCATOR, REF_MAP},
};

use super::*;
use dashmap::mapref::entry::Entry;

// TODO!(add way to erase the panic no local copy for dref and dmutref)
// Just check if the hashmap contains the ptr is enough
pub struct DRef<'a, T: DRust + Sized> {
    pub(crate) orig: &'a T,
    pub(crate) copy: Option<*const T>,
}

impl<'a, T: DRust + Sized> Deref for DRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self.copy {
            Some(x) => unsafe { &(*x) },
            None => {
                panic!("No local copy, call local_copy first");
            }
        }
    }
}

impl<'a, T: DRust + Sized> Drop for DRef<'a, T> {
    fn drop(&mut self) {
        if self.copy.is_none() {
            return;
        }
        let orig_addr = self.orig as *const T as usize;
        let current_addr = *(self.copy.as_ref().unwrap());
        if orig_addr == current_addr as usize {
            return;
        }
        if current_place(current_addr as usize) != Destination::Local {
            panic!("Dropping a remote copy of a DRef");
        }
        let ref_map = unsafe { Arc::clone(REF_MAP.as_ref().unwrap()) };
        match ref_map.entry(orig_addr) {
            Entry::Occupied(mut entry) => {
                let (ptr, count) = entry.get_mut();
                *count -= 1;
                if *count == 0 {
                    unsafe {
                        LOCAL_ALLOCATOR.deallocate(
                            NonNull::new_unchecked((*ptr) as *mut u8),
                            Layout::new::<T>(),
                        )
                    };
                    entry.remove();
                }
            }
            Entry::Vacant(_) => {
                panic!("No entry in ref map");
            }
        };
    }
}

impl<'a, T: DRust + Sized> DRef<'a, T> {
    pub fn local_copy(&mut self) {
        match self.copy.as_ref() {
            Some(ptr) => {
                if current_place(*ptr as usize) != Destination::Local {
                    panic!("Already have a remote copy");
                }
                return;
            }
            None => {}
        };
        let orig_addr = self.orig as *const T as usize;
        match current_place(orig_addr) {
            Destination::Local => {
                self.copy = Some(self.orig as *const T);
            }
            Destination::Remote(_server_idx) => {
                let ref_map = unsafe { Arc::clone(REF_MAP.as_ref().unwrap()) };
                match ref_map.entry(orig_addr) {
                    Entry::Occupied(mut entry) => {
                        let (ptr, count) = entry.get_mut();
                        *count += 1;
                        self.copy = Some(*ptr as *const T);
                    }
                    Entry::Vacant(entry) => {
                        let ptr = unsafe {
                            LOCAL_ALLOCATOR
                                .allocate(Layout::new::<T>())
                                .unwrap()
                                .as_mut_ptr() as *mut T
                        };
                        entry.insert((ptr as usize, 1));
                        unsafe {
                            drust_read_sync(
                                ptr as usize - LOCAL_HEAP_START,
                                orig_addr - GLOBAL_HEAP_START,
                                mem::size_of::<T>(),
                                thread::current().id().as_u64().get() as usize,
                            );
                        }
                        self.copy = Some(ptr);
                    }
                };
            }
        };
    }
}
