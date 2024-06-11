use std::{
    alloc::{Allocator, Layout}, intrinsics, ops::{Deref, DerefMut}, ptr::{self, NonNull}, sync::Arc, thread
};

use crate::{
    dassert, dprintln, drust_std::alloc::{ddeallocate, ddrop, LOCAL_ALLOCATOR, REF_MAP}
};

use super::*;
use dashmap::mapref::entry::Entry;
use dmut::*;
use dref::*;
use tbox::*;

pub struct DBox<T: DRust + Sized> {
    // pub data: Box<T, System>,
    pub(crate) data: Option<Box<T, &'static good_memory_allocator::SpinLockedAllocator>>,
    pub(crate) copy: *mut T,
    pub(crate) copy_exists: bool,
}

unsafe impl<T: DRust + Sized + Send> Send for DBox<T> {}
unsafe impl<T: DRust + Sized + Sync> Sync for DBox<T> {}

impl<T: DRust + Sized + Clone> Clone for DBox<T> {
    fn clone(&self) -> Self {
        let raw_1 = ptr::addr_of!(**(self.data.as_ref().unwrap()));
        // println!("raw_1: {:x}", raw_1 as usize);
        let src = current_place(raw_1 as usize);
        match src {
            Destination::Local => {
                let new_data = self.data.as_ref().unwrap().clone();
                DBox {
                    data: Some(new_data),
                    copy: ptr::null_mut(),
                    copy_exists: false,
                }
            }
            Destination::Remote(_server_idx) => {
                // TODO! (add a better way to clone remote data, currently do not support remote clone complicated type)
                let ptr = unsafe {
                    LOCAL_ALLOCATOR
                        .allocate(Layout::new::<T>())
                        .unwrap()
                        .as_mut_ptr() as *mut T
                };
                // if SETTING == 2 && mem::size_of::<T>() > 512 * 1024 {
                //     for i in 0..mem::size_of::<T>() / (512 * 1024) {
                //         unsafe { drust_read_sync(ptr as usize - LOCAL_HEAP_START + i * 512 * 1024, raw_1 as usize - GLOBAL_HEAP_START + i * 512 * 1024, 512 * 1024, thread::current().id().as_u64().get() as usize); }
                //     }
                //     if mem::size_of::<T>() % (512 * 1024) != 0 {
                //         unsafe { drust_read_sync(ptr as usize - LOCAL_HEAP_START + mem::size_of::<T>() / (512 * 1024) * 512 * 1024, raw_1 as usize - GLOBAL_HEAP_START + mem::size_of::<T>() / (512 * 1024) * 512 * 1024, mem::size_of::<T>() % (512 * 1024), thread::current().id().as_u64().get() as usize); }
                //     }
                // } else {
                unsafe {
                    drust_read_sync(
                        ptr as usize - LOCAL_HEAP_START,
                        raw_1 as usize - GLOBAL_HEAP_START,
                        mem::size_of::<T>(),
                        thread::current().id().as_u64().get() as usize,
                    );
                }
                // }
                let x = unsafe { Box::from_raw_in(ptr, &LOCAL_ALLOCATOR) };
                DBox {
                    data: Some(x),
                    copy: ptr::null_mut(),
                    copy_exists: false,
                }
            }
        }
    }
}

impl<T: DRust + Sized> Deref for DBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self.get_place() {
            Destination::Local => &(**self.data.as_ref().unwrap()),
            Destination::Remote(_) => {
                if !self.copy_exists {
                    self.local_copy();
                }
                unsafe { &*self.copy }
            }
        }
    }
}

impl<T: DRust + Sized> DerefMut for DBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.migrate_to_local();
        &mut (**self.data.as_mut().unwrap())
    }
}

impl<T: DRust + Sized> Drop for DBox<T> {
    fn drop(&mut self) {
        self.drop_copy();
        if self.data.is_none() {
            return;
        }
        let type_id = self.data.as_ref().unwrap().typeid();
        let raw_orig = consume_original_data(&mut self.data);
        let des = current_place(raw_orig as usize);
        match des {
            Destination::Local => {
                let local_data = unsafe { Box::from_raw_in(raw_orig, &LOCAL_ALLOCATOR) };
                drop(local_data);
            }
            Destination::Remote(_server_idx) => {
                dprintln!("T type: {}", std::any::type_name::<T>());
                ddrop(
                    unsafe { NonNull::new_unchecked(raw_orig as *mut u8) },
                    type_id as usize,
                    _server_idx,
                );
            }
        }
    }
}

impl<'a, T: DRust + Sized> DBox<T> {
    pub unsafe fn into_raw(mut self) -> *mut T{
        if self.data.is_none() {
            return ptr::null_mut();
        }
        let raw_orig = consume_original_data(&mut self.data);
        raw_orig
    }

    pub unsafe fn into_raw_addr(mut self) -> usize {
        if self.data.is_none() {
            return 0;
        }
        let raw_orig = consume_original_data(&mut self.data);
        raw_orig as usize
    }

    pub unsafe fn from_raw(raw: *mut T) -> Self {
        dassert!(raw as usize >= GLOBAL_HEAP_START, "Invalid address!");
        DBox {
            data: Some(Box::from_raw_in(raw, &LOCAL_ALLOCATOR)),
            copy: ptr::null_mut(),
            copy_exists: false,
        }
    }

    pub unsafe fn from_raw_box(
        raw: Box<T, &'static good_memory_allocator::SpinLockedAllocator>,
    ) -> Self {
        DBox { 
            data: Some(raw),
            copy: ptr::null_mut(),
            copy_exists: false,
        }
    }

    pub fn new(contents: T) -> Self {
        let ptr = unsafe {
            LOCAL_ALLOCATOR
                .allocate(Layout::new::<T>())
                .unwrap()
                .as_mut_ptr() as *mut T
        };
        let mut x = unsafe { Box::from_raw_in(ptr, &LOCAL_ALLOCATOR) };
        unsafe {
            // copy_mem(raw_1 as usize, raw_2 as usize, mem::size_of::<T>());
            ptr::write_volatile(x.as_mut() as *mut T, contents);
        }
        DBox {
            data: Some(x),
            copy: ptr::null_mut(),
            copy_exists: false,
        }
    }

    pub fn box_new(contents: Box<T>) -> Self {
        let ptr = unsafe {
            LOCAL_ALLOCATOR
                .allocate(Layout::new::<T>())
                .unwrap()
                .as_mut_ptr() as *mut T
        };
        let x = unsafe { Box::from_raw_in(ptr, &LOCAL_ALLOCATOR) };
        let remote = DBox { 
            data: Some(x),
            copy: ptr::null_mut(),
            copy_exists: false,
        };
        let raw_1 = ptr::addr_of!(*contents);
        let raw_2 = ptr::addr_of!(**(remote.data.as_ref().unwrap()));
        unsafe {
            // copy_mem(raw_1 as usize, raw_2 as usize, mem::size_of::<T>());
            intrinsics::volatile_copy_nonoverlapping_memory(raw_2 as *mut T, raw_1 as *const T, 1);
        }
        let (addr, alloc) = Box::into_raw_with_allocator(contents);
        unsafe {
            alloc.deallocate(NonNull::new_unchecked(addr as *mut u8), Layout::new::<T>());
        }
        remote
    }

    // pub fn box_new_on(contents: Box<T>, server_id: usize) -> Self {
    //     if server_id == unsafe{SERVER_INDEX} {
    //         return DBox::box_new(contents);
    //     }
    //     let local_ptr = unsafe{LOCAL_ALLOCATOR.allocate(Layout::new::<T>()).unwrap().as_mut_ptr() as *mut T};
    //     let remote_ptr = dallocate(Layout::new::<T>(), server_id).unwrap().as_mut_ptr() as *mut T;
    //     let raw_1 = ptr::addr_of!(*contents);
    //     unsafe{
    //         copy_mem(raw_1 as usize, local_ptr as usize, mem::size_of::<T>());
    //         drust_write_sync(local_ptr as usize - LOCAL_HEAP_START, remote_ptr as usize - GLOBAL_HEAP_START, mem::size_of::<T>(), thread::current().id().as_u64().get() as usize);
    //         let (addr, alloc) = Box::into_raw_with_allocator(contents);
    //         alloc.deallocate(NonNull::new_unchecked(addr as *mut u8), Layout::new::<T>());
    //         LOCAL_ALLOCATOR.deallocate(NonNull::new_unchecked(local_ptr as *mut u8), Layout::new::<T>());
    //     }
    //     let x = unsafe{Box::from_raw_in(remote_ptr, get_remote_allocator(server_id))};
    //     DBox {
    //         data: Some(x)
    //     }
    // }

    pub fn get_place(&self) -> Destination {
        let raw_1 = ptr::addr_of!(**(self.data.as_ref().unwrap())) as usize;
        current_place(raw_1)
    }

    pub fn get_addr(&self) -> usize {
        let raw_1 = ptr::addr_of!(**(self.data.as_ref().unwrap())) as usize;
        raw_1
    }

    pub fn as_local(mut self) -> TBox<T> {
        let raw_1 = ptr::addr_of!(**(self.data.as_ref().unwrap()));
        match current_place(raw_1 as usize) {
            Destination::Local => TBox {
                data: mem::replace(&mut self.data, None),
            },
            Destination::Remote(_server_idx) => {
                self.drop_copy();
                let ptr = unsafe {
                    LOCAL_ALLOCATOR
                        .allocate(Layout::new::<T>())
                        .unwrap()
                        .as_mut_ptr() as *mut T
                };
                let x = unsafe { Box::from_raw_in(ptr, &LOCAL_ALLOCATOR) };
                let mut local = TBox { data: Some(x) };
                let raw_2 = ptr::addr_of!(**(local.data.as_ref().unwrap()));
                unsafe {
                    drust_read_sync(
                        raw_2 as usize - LOCAL_HEAP_START,
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
                consume_original_data(&mut self.data);
                local.data.as_mut().unwrap().migrate(Destination::Local);
                local
            }
        }
    }

    pub fn migrate_to_local(&mut self) {
        let raw_1 = ptr::addr_of!(**(self.data.as_ref().unwrap()));
        let src = current_place(raw_1 as usize);
        match src {
            Destination::Local => {
                // println!("Already in destination");
            }
            Destination::Remote(_server_idx) => {
                self.drop_copy();
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
                self.data.as_mut().unwrap().migrate(Destination::Local);
            }
        };
    }

    fn drop_copy(&mut self) {
        if self.copy_exists {
            let ref_map = unsafe { Arc::clone(REF_MAP.as_ref().unwrap()) };
            let orig_addr = self.data.as_ref().unwrap().as_ref() as *const T as usize;
            match ref_map.entry(orig_addr) {
                Entry::Occupied(mut entry) => {
                    let (ptr, count) = entry.get_mut();
                    *count -= 1;
                    assert!(*count == 0, "Owner should be the last reference!");
                    unsafe {
                        assert!(current_place(*ptr as usize) == Destination::Local, "Copy should be local!");
                        LOCAL_ALLOCATOR.deallocate(
                            NonNull::new_unchecked(*ptr as *mut u8),
                            Layout::new::<T>(),
                        );
                    }
                    entry.remove();
                }
                Entry::Vacant(_) => {
                    panic!("Copy should be in ref_map!");
                }
            }
            self.copy_exists = false;
            self.copy = ptr::null_mut();
        }
    }

    fn local_copy(&self) {
        
        let orig_addr = self.data.as_ref().unwrap().as_ref() as *const T as usize;
        if current_place(orig_addr) == Destination::Local {
            return;
        }
        if self.copy_exists {
            assert!(current_place(self.copy as usize) == Destination::Local, "Copy should be local!");
            return;
        }
        let ref_map = unsafe { Arc::clone(REF_MAP.as_ref().unwrap()) };
        match ref_map.entry(orig_addr) {
            Entry::Occupied(mut entry) => {
                let (ptr, count) = entry.get_mut();
                *count += 1;
                // self.copy = Some(*ptr as *const T);
                unsafe{
                    ptr::write_volatile(&self.copy as *const *mut T as *mut *mut T, *ptr as *mut T);
                    ptr::write_volatile(&self.copy_exists as *const bool as *mut bool, true);
                }
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
                unsafe {
                    ptr::write_volatile(&self.copy as *const *mut T as *mut *mut T, ptr);
                    ptr::write_volatile(&self.copy_exists as *const bool as *mut bool, true);
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

    // pub fn get_mut_ref(&'a mut self) -> DMutRef<'a, T> {
    //     DMutRef {
    //         orig: &mut (**self.data.as_mut().unwrap()),
    //         copy: None,
    //     }
    // }
    pub fn get_mut_ref(&'a mut self) -> DMut<'a, T> {
        if self.copy_exists {
            self.migrate_to_local();
        }
        let data_addr: *const Option<Box<T, &good_memory_allocator::SpinLockedAllocator<20, 8>>> = ptr::addr_of!(self.data);
        let combination: usize = ((unsafe{SERVER_INDEX} << 58) | (data_addr as usize));
        DMut {
            orig: &mut (**self.data.as_mut().unwrap()),
            copy: ptr::null_mut(),
            owner: combination,
            copy_exists: false,
        }
    }
}
