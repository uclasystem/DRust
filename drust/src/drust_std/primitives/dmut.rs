use std::{
    alloc::{Allocator, Layout},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    thread,
};

use crate::{
    dprintln,
    drust_std::alloc::{ddeallocate, ddrop, LOCAL_ALLOCATOR},
};

use super::*;
use tbox::*;

pub struct DMutRef<'a, T: DRust + Sized> {
    pub(crate) orig: &'a mut T,
    pub(crate) copy: Option<*mut T>,
}

impl<'a, T: DRust + Sized> Deref for DMutRef<'a, T> {
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

impl<'a, T: DRust + Sized> DerefMut for DMutRef<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.local_copy();
        unsafe { &mut (*(self.copy.unwrap())) }
    }
}

impl<'a, T: DRust + Sized> Drop for DMutRef<'a, T> {
    fn drop(&mut self) {
        if self.copy.is_none() {
            return;
        }
        let orig_addr = self.orig as *const T as usize;
        let ptr = self.copy.unwrap() as usize;
        if current_place(ptr) != Destination::Local {
            panic!("Dropping a remote copy of a DRef");
        }
        unsafe {
            drust_write_sync(
                ptr - LOCAL_HEAP_START,
                orig_addr - GLOBAL_HEAP_START,
                mem::size_of::<T>(),
                thread::current().id().as_u64().get() as usize,
            );
            LOCAL_ALLOCATOR.deallocate(NonNull::new_unchecked(ptr as *mut u8), Layout::new::<T>());
        }
    }
}

impl<'a, T: DRust + Sized> DMutRef<'a, T> {
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
                self.copy = Some(self.orig as *mut T);
            }
            Destination::Remote(_server_idx) => {
                let ptr = unsafe {
                    LOCAL_ALLOCATOR
                        .allocate(Layout::new::<T>())
                        .unwrap()
                        .as_mut_ptr() as *mut T
                };
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
}
