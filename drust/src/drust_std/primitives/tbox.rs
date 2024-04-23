use std::{
    alloc::{Allocator, Layout},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    thread,
};

use crate::{dprintln, drust_std::alloc::*};

use super::*;
use dbox::*;
use dmut::*;
use dref::*;

pub struct TBox<T: DRust + Sized> {
    pub(crate) data: Option<Box<T, &'static good_memory_allocator::SpinLockedAllocator>>,
}

impl<'a, T: DRust + Sized> TBox<T> {
    pub fn as_remote(mut self) -> DBox<T> {
        let original_data = mem::replace(&mut self.data, None);
        let raw_1 = Box::into_raw(original_data.unwrap());
        DBox {
            data: Some(unsafe { Box::from_raw_in(raw_1, &LOCAL_ALLOCATOR) }),
            copy: ptr::null_mut(),
            copy_exists: false,
        }
    }

    pub fn migrate(&mut self, dst: Destination) {
        let raw_1 = ptr::addr_of!(**(self.data.as_ref().unwrap()));
        let src = current_place(raw_1 as usize);

        match dst {
            Destination::Local => {
                match src {
                    Destination::Local => {
                        // println!("Already in destination");
                    }
                    Destination::Remote(_server_idx) => {
                        let ptr = unsafe {
                            LOCAL_ALLOCATOR
                                .allocate(Layout::new::<T>())
                                .unwrap()
                                .as_mut_ptr() as *mut T
                        };
                        let x = unsafe { Box::from_raw_in(ptr, &LOCAL_ALLOCATOR) };
                        unsafe {
                            drust_read_sync(
                                ptr as usize - LOCAL_HEAP_START,
                                raw_1 as usize - GLOBAL_HEAP_START,
                                mem::size_of::<T>(),
                                thread::current().id().as_u64().get() as usize,
                            );
                        }

                        ddeallocate(
                            unsafe { NonNull::new_unchecked(raw_1 as *mut u8) },
                            Layout::new::<T>(),
                            _server_idx,
                        );

                        let original_data = mem::replace(&mut self.data, Some(x));
                        let _ = Box::into_raw(original_data.unwrap());
                        self.data.as_mut().unwrap().migrate(dst);
                    }
                }
            }
            Destination::Remote(_server_idx) => {
                match src {
                    Destination::Local => {
                        self.data.as_mut().unwrap().migrate(dst);
                        let ptr = dallocate(Layout::new::<T>(), _server_idx)
                            .unwrap()
                            .as_mut_ptr() as *mut T;
                        let x = unsafe { Box::from_raw_in(ptr, get_remote_allocator(_server_idx)) };
                        unsafe {
                            drust_write_sync(
                                raw_1 as usize - LOCAL_HEAP_START,
                                ptr as usize - GLOBAL_HEAP_START,
                                mem::size_of::<T>(),
                                thread::current().id().as_u64().get() as usize,
                            );
                        }
                        // wait_sync();
                        unsafe {
                            LOCAL_ALLOCATOR.deallocate(
                                NonNull::new_unchecked(raw_1 as *mut u8),
                                Layout::new::<T>(),
                            )
                        };
                        let original_data = mem::replace(&mut self.data, Some(x));
                        let _ = Box::into_raw(original_data.unwrap());
                    }
                    Destination::Remote(_server_idx) => {
                        println!("Not support Remote to remote migration now");
                    }
                }
            }
        };
    }

    pub fn get_ref(&'a self) -> DRef<'a, T> {
        DRef {
            orig: &(**self.data.as_ref().unwrap()),
            copy: None,
        }
    }

    pub fn get_mut_ref(&'a mut self) -> DMutRef<'a, T> {
        DMutRef {
            orig: &mut (**self.data.as_mut().unwrap()),
            copy: None,
        }
    }
}

// TODO!(add a way to also copy tbox values to local when deref dmutref)
// Current only support get dref and dereference it.

impl<T: DRust + Sized> Deref for TBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        let raw_1 = ptr::addr_of!(**(self.data.as_ref().unwrap()));
        if current_place(raw_1 as usize) != Destination::Local {
            panic!("Not in local, get dref and dereference it first");
        }
        &(**self.data.as_ref().unwrap())
    }
}

impl<T: DRust + Sized> DerefMut for TBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        let raw_1 = ptr::addr_of!(**(self.data.as_ref().unwrap()));
        if current_place(raw_1 as usize) != Destination::Local {
            panic!("Not in local, get dmutref and dereference it first");
        }
        &mut (**self.data.as_mut().unwrap())
    }
}
