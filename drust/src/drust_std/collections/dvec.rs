use core::panic;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::{intrinsics, ptr};
use std::ops::{Index, IndexMut};
use dashmap::mapref::entry::Entry;
use serde::{Deserialize, Serialize};
use std::{alloc::Layout, mem, ptr::NonNull, sync::Arc, thread};

use crate::drust_std::comm::*;
use crate::drust_std::{alloc::*, primitives::*};
use crate::{conf::*, dassert, dprintln};

// #[derive(Clone)]
pub struct DVec<T: DRust> {
    pub internal_vec: Option<Vec<T, &'static good_memory_allocator::SpinLockedAllocator>>,
    pub copy: Vec<T, &'static good_memory_allocator::SpinLockedAllocator>,
    pub copy_exists: bool,
}

impl<T:DRust> Default for DVec<T> {
    fn default() -> Self {
        unsafe{
            DVec {
                internal_vec: Some(Vec::new_in(&LOCAL_ALLOCATOR)),
                copy: Vec::new_in(&LOCAL_ALLOCATOR),
                copy_exists: false,
            }
        }
    }
}

impl<T: DRust + Clone> Clone for DVec<T> {
    fn clone(&self) -> Self {
        
        let vec_addr = self.internal_vec.as_ref().unwrap().as_ptr() as usize;
        let vec_ref = match current_place(vec_addr) {
            Destination::Local => {self.internal_vec.as_ref().unwrap()},
            Destination::Remote(_) => {
                self.local_copy();
                self.copy.as_ref()
            }
        };
        let new_dvec = vec_ref.clone();
        unsafe {
            DVec {
                internal_vec: Some(new_dvec),
                copy: Vec::new_in(&LOCAL_ALLOCATOR),
                copy_exists: false,
            }
        }
    }
}

impl<T: DRust> DRust for DVec<T> {
    fn static_typeid() -> u32 {
        (T::static_typeid() << 8) | 1
    }
    fn typeid(&self) -> u32 {
        (T::static_typeid() << 8) | 1
    }
    fn migrate(&mut self, dest: Destination) -> bool {
        false
        // match dest {
        //     Destination::Local => {
        //         self.migrate_to_local();
        //         true
        //     }
        //     Destination::Remote(server_idx) => {
        //         let vec_addr = self.internal_vec.as_ref().unwrap().as_ptr() as usize;
        //         match current_place(vec_addr) {
        //             Destination::Local => {
        //                 let local_vec = self.internal_vec.as_mut().unwrap();
        //                 if local_vec.len() > 0 && local_vec[0].migrate(Destination::Local) {
        //                     for item in local_vec[1..].iter_mut() {
        //                         item.migrate(Destination::Remote(server_idx));
        //                     }
        //                 }
        //                 let allocate_size = local_vec.capacity() * mem::size_of::<T>();
        //                 let buffer = unsafe {
        //                     dallocate(
        //                         Layout::from_size_align_unchecked(
        //                             allocate_size,
        //                             mem::align_of::<T>(),
        //                         ),
        //                         server_idx,
        //                     )
        //                     .unwrap()
        //                     .as_mut_ptr() as *mut T
        //                 };
        //                 unsafe {
        //                     drust_write_large_sync(
        //                         vec_addr - LOCAL_HEAP_START,
        //                         (buffer as usize) - GLOBAL_HEAP_START,
        //                         local_vec.len() * mem::size_of::<T>(),
        //                         thread::current().id().as_u64().get() as usize,
        //                     );
        //                 }
        //                 let remote_vec = unsafe {
        //                     Vec::from_raw_parts_in(
        //                         buffer,
        //                         local_vec.len(),
        //                         local_vec.capacity(),
        //                         get_remote_allocator(server_idx),
        //                     )
        //                 };
        //                 unsafe {
        //                     local_vec.set_len(0);
        //                 }
        //                 self.internal_vec = Some(remote_vec);
        //                 true
        //             }
        //             Destination::Remote(_) => {
        //                 panic!("Not support remote to remote migration!")
        //             }
        //         }
        //     }
        // }
    }
}

impl<T: DRust> Drop for DVec<T> {
    fn drop(&mut self) {
        let typeid = T::static_typeid();
        self.drop_copy();
        if self.internal_vec.is_none() {
            return;
        }
        if (typeid & 0xFF) == 1 || (typeid & 0xFF) == 2 {
            self.migrate_to_local();
        } 
        let ivec_option = mem::replace(&mut self.internal_vec, None);
        match ivec_option {
            Some(ivec) => {
                if ivec.capacity() == 0 {
                    return;
                }
                let raw_addr = ivec.as_ptr() as usize;
                // dprintln!("Dropping DVec at {:x}", raw_addr);
                match current_place(raw_addr) {
                    Destination::Local => {
                        // dprintln!("Dropping DVec at {:x} as local", raw_addr);
                        drop(ivec);
                    }
                    Destination::Remote(server_idx) => {
                        if ivec.capacity() == 0 {
                            return;
                        }
                        dprintln!(
                            "Dropping DVec at {:x} as remote in server {}",
                            raw_addr,
                            server_idx
                        );
                        let (raw_internal_vec, length, capacity) = ivec.into_raw_parts();
                        ddrop_vec(
                            unsafe { NonNull::new_unchecked(raw_internal_vec as *mut u8) },
                            typeid as usize,
                            capacity,
                            length,
                            server_idx,
                        );
                    }
                };
            }
            None => {}
        }
    }
}

impl<T> Index<usize> for DVec<T> 
where 
    T: DRust,
{
    type Output = T;

    fn index(&self, index: usize) -> &T {
        if current_place(self.internal_vec.as_ref().unwrap().as_ptr() as usize) == Destination::Local {
            &self.internal_vec.as_ref().unwrap()[index]
        } else {
            self.local_copy();
            &self.copy[index]
        }
    }
}

impl<T> IndexMut<usize> for DVec<T>
where
    T: DRust,
{
    fn index_mut(&mut self, index: usize) -> &mut T {
        if current_place(self.internal_vec.as_ref().unwrap().as_ptr() as usize) != Destination::Local {
            self.migrate_to_local();
        }
        &mut self.internal_vec.as_mut().unwrap()[index]
    }
}

impl<'a, T: DRust> DVec<T> {

    pub fn new() -> Self {
        unsafe{
            DVec {
                internal_vec: Some(Vec::new_in(&LOCAL_ALLOCATOR)),
                copy: Vec::new_in(&LOCAL_ALLOCATOR),
                copy_exists: false,
            }
        }
    }

    fn drop_copy(&mut self) {
        if self.copy_exists {
            let orig_addr = self.internal_vec.as_ref().unwrap().as_ptr() as usize;
            let empty_copy = unsafe{Vec::new_in(&LOCAL_ALLOCATOR)};  
            let ref_map = unsafe { Arc::clone(REF_MAP.as_ref().unwrap()) };
            match ref_map.entry(orig_addr) {
                Entry::Occupied(mut entry) => {
                    let mut v = mem::replace(&mut self.copy, empty_copy);
                    self.copy_exists = false;
                    dassert!(current_place(v.as_ptr() as usize) == Destination::Local, "Dropping a remote copy of a DVec copy");
                    let (_, count) = entry.get_mut();
                    *count -= 1;
                    dassert!(*count == 0, "owner copy must be the last ref");
                    dprintln!(
                        "Dropping last ref to origin: {:x}, copy addr: {:x}",
                        orig_addr,
                        v.as_ptr() as usize
                    );
                    unsafe {
                        v.set_len(0);
                    }
                    drop(v);
                    entry.remove();
                }
                Entry::Vacant(_) => {
                    panic!("No entry in ref map");
                }
            };
        } else {
            dassert!(self.copy.capacity() == 0, "Capacity must be 0");
        }
    } 


    pub fn len(&self) -> usize {
        self.internal_vec.as_ref().unwrap().len()
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        if len > self.internal_vec.as_ref().unwrap().capacity() {
            panic!("Length exceeds capacity");
        }
        self.internal_vec.as_mut().unwrap().set_len(len);
    }

    pub fn as_ptr(&self) -> *const T {
        self.internal_vec.as_ref().unwrap().as_ptr()
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        let mut vec: Vec<T, &'static good_memory_allocator::SpinLockedAllocator> =
            unsafe { Vec::with_capacity_in(capacity, &LOCAL_ALLOCATOR) };
        DVec {
            internal_vec: Some(vec),
            copy: unsafe {
                Vec::new_in(&LOCAL_ALLOCATOR)
            },
            copy_exists: false,
        }
    }

    pub fn push(&mut self, item: T) {
        if current_place(self.internal_vec.as_ref().unwrap().as_ptr() as usize) != Destination::Local {
            self.migrate_to_local();
        }
        self.internal_vec.as_mut().unwrap().push(item);
    }

    pub fn server_idx(&self) -> usize {
        match current_place(self.internal_vec.as_ref().unwrap().as_ptr() as usize) {
            Destination::Local => unsafe { SERVER_INDEX },
            Destination::Remote(s_id) => s_id,
        }
    }

    pub unsafe fn from_raw(
        raw: Vec<T, &'static good_memory_allocator::SpinLockedAllocator>,
    ) -> Self {
        DVec {
            internal_vec: Some(raw),
            copy: unsafe {
                Vec::new_in(&LOCAL_ALLOCATOR)
            },
            copy_exists: false,
        }
    }

    pub unsafe fn into_raw_parts(mut self) -> (*mut T, usize, usize) {
        let mut ivec = mem::replace(&mut self.internal_vec, None).unwrap();
        ivec.into_raw_parts()
    }

    pub fn from_vec(mut vec: Vec<T>) -> Self {
        let mut rvec: Vec<T, &'static good_memory_allocator::SpinLockedAllocator> =
            unsafe { Vec::with_capacity_in(vec.capacity(), &LOCAL_ALLOCATOR) };
        unsafe {
            // copy_mem(
            //     vec.as_ptr() as usize,
            //     rvec.as_ptr() as usize,
            //     vec.len() * mem::size_of::<T>(),
            // );
            intrinsics::volatile_copy_nonoverlapping_memory(
                rvec.as_mut_ptr(),
                vec.as_ptr(),
                vec.len(),
            );
            rvec.set_len(vec.len());
            vec.set_len(0);
        }

        DVec {
            internal_vec: Some(rvec),
            copy: unsafe {
                Vec::new_in(&LOCAL_ALLOCATOR)
            },
            copy_exists: false,
        }
    }

    pub fn migrate_to_local(&mut self) {
        if self.copy_exists {
            self.drop_copy();
        }
        if self.internal_vec.as_ref().unwrap().capacity() == 0 {
            return;
        }
        let vec_addr = self.internal_vec.as_ref().unwrap().as_ptr() as usize;
        match current_place(vec_addr) {
            Destination::Local => {}
            Destination::Remote(server_idx) => {
                let mut ivec = mem::replace(&mut self.internal_vec, None).unwrap();
                let (raw_ivec, length, capacity) = ivec.into_raw_parts();
                dprintln!(
                    "--------------------------Real Migrating to local--------------------------"
                );
                let mut local_vec: Vec<T, &'static good_memory_allocator::SpinLockedAllocator> =
                    unsafe { Vec::with_capacity_in(capacity, &LOCAL_ALLOCATOR) };
                unsafe {
                    drust_read_large_sync(
                        (local_vec.as_ptr() as *mut T as usize) - LOCAL_HEAP_START,
                        vec_addr - GLOBAL_HEAP_START,
                        length * mem::size_of::<T>(),
                        thread::current().id().as_u64().get() as usize,
                    );
                    local_vec.set_len(length);
                }
                unsafe {
                    ddeallocate(
                        NonNull::new_unchecked(vec_addr as *mut u8),
                        Layout::from_size_align_unchecked(
                            capacity * mem::size_of::<T>(),
                            mem::align_of::<T>(),
                        ),
                        server_idx,
                    );
                }
                if local_vec.len() > 0 && local_vec[0].migrate(Destination::Local) {
                    for item in local_vec[1..].iter_mut() {
                        item.migrate(Destination::Local);
                    }
                }
                self.internal_vec = Some(local_vec);
            }
        }
    }

    pub fn local_copy(&self) {
        if current_place(self.internal_vec.as_ref().unwrap().as_ptr() as usize) == Destination::Local {
            return;
        }
        if self.copy_exists {
            dassert!(
                current_place(self.copy.as_ptr() as usize) == Destination::Local,
                "Already have a remote copy",
            );
            return;
        }
        let raw_orig = self.internal_vec.as_ref().unwrap().as_ptr() as usize;
        let ref_map = unsafe { Arc::clone(REF_MAP.as_ref().unwrap()) };
        match ref_map.entry(raw_orig) {
            Entry::Occupied(mut entry) => {
                dprintln!("Already have an entry in ref map");
                let (ptr, count) = entry.get_mut();
                *count += 1;
                let copy_vec= unsafe {
                    Vec::from_raw_parts_in(
                        *ptr as *mut T,
                        self.internal_vec.as_ref().unwrap().len(),
                        self.internal_vec.as_ref().unwrap().capacity(),
                        &LOCAL_ALLOCATOR,
                    )
                };
                unsafe {
                    let dst_ptr = &self.copy as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as *mut Vec<T, &'static good_memory_allocator::SpinLockedAllocator>;
                    std::ptr::write_volatile(dst_ptr, copy_vec);
                    std::ptr::write_volatile(&self.copy_exists as *const bool as *mut bool, true);
                    dprintln!("Copy addr: {:x}", self.copy.as_ptr() as usize);
                }
                dprintln!(
                    "Copy addr: {:x}, copy_exists: {}",
                    self.copy.as_ptr() as usize,
                    self.copy_exists
                );
            }
            Entry::Vacant(entry) => {
                dprintln!("Creating a new entry in ref map");
                let mut v: Vec<T, &good_memory_allocator::SpinLockedAllocator> =
                    unsafe { Vec::with_capacity_in(self.internal_vec.as_ref().unwrap().capacity(), &LOCAL_ALLOCATOR) };
                let ptr = v.as_ptr();
                dprintln!("Copy addr: {:x}", ptr as usize);
                dprintln!(
                    "thread id: {:x}",
                    thread::current().id().as_u64().get() as usize
                );
                unsafe {
                    drust_read_large_sync(
                        (ptr as usize) - LOCAL_HEAP_START,
                        raw_orig - GLOBAL_HEAP_START,
                        self.internal_vec.as_ref().unwrap().len() * mem::size_of::<T>(),
                        thread::current().id().as_u64().get() as usize,
                    );
                    dprintln!("Copy addr: {:x}", ptr as usize);
                    v.set_len(self.internal_vec.as_ref().unwrap().len());
                }
                dprintln!("first 4 bytes: {:x}", unsafe { *(ptr as *const usize) });
                unsafe {
                    let dst_ptr = (&self.copy) as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as usize as *mut Vec<T, &'static good_memory_allocator::SpinLockedAllocator>;
                    std::ptr::write_volatile(dst_ptr, v);
                    std::ptr::write_volatile(&self.copy_exists as *const bool as usize as *mut bool, true);
                }
                dprintln!(
                    "Copy addr: {:x}, copy_exists: {}",
                    self.copy.as_ptr() as usize,
                    self.copy_exists
                );

                entry.insert((ptr as usize, 1));
            }
        };

    }

    pub fn as_local(mut self) -> Vec<T, &'static good_memory_allocator::SpinLockedAllocator> {
        self.migrate_to_local();
        mem::replace(&mut self.internal_vec, None).unwrap()
    }

    pub fn from_local(vec: Vec<T, &'static good_memory_allocator::SpinLockedAllocator>) -> Self {
        DVec {
            internal_vec: Some(vec),
            copy: unsafe{Vec::new_in(&LOCAL_ALLOCATOR)},
            copy_exists: false,
        }
    }

    pub fn as_local_ref(
        &'a self,
    ) -> &'a Vec<T, &'static good_memory_allocator::SpinLockedAllocator> {
        if self.internal_vec.as_ref().unwrap().capacity() == 0 {
            return self.internal_vec.as_ref().unwrap();
        }
        let vec_addr = self.internal_vec.as_ref().unwrap().as_ptr() as usize;
        match current_place(vec_addr) {
            Destination::Local => {self.internal_vec.as_ref().unwrap()},
            Destination::Remote(_) => {
                self.local_copy();
                self.copy.as_ref()
            }
        }
    }

    pub unsafe fn as_local_mut_ref(
        &'a mut self,
    ) -> &'a mut Vec<T, &'static good_memory_allocator::SpinLockedAllocator> {
        self.migrate_to_local();
        self.internal_vec.as_mut().unwrap()
    }

    pub fn as_dref(&'a self) -> DVecRef<'a, T> {
        let raw_addr = self.internal_vec.as_ref().unwrap().as_ptr() as usize;
        let len = self.internal_vec.as_ref().unwrap().len();
        let cap = self.internal_vec.as_ref().unwrap().capacity();
        DVecRef {
            orig_vec: self.internal_vec.as_ref().unwrap(),
            orig_raw: (raw_addr, len, cap),
            copy: unsafe{Vec::new_in(&LOCAL_ALLOCATOR)},
            copy_exists: false,
        }
    }

    pub fn as_dmut(&mut self) -> DVecMutRef<T> {
        let raw_addr = self.internal_vec.as_ref().unwrap().as_ptr() as usize;
        let len = self.internal_vec.as_ref().unwrap().len();
        let cap = self.internal_vec.as_ref().unwrap().capacity();
        DVecMutRef {
            orig_vec: self.internal_vec.as_mut().unwrap(),
            orig_raw: (raw_addr, len, cap),
            copy: None,
        }
    }
}

pub struct DVecRef<'a, T: DRust> {
    pub orig_vec: &'a Vec<T, &'static good_memory_allocator::SpinLockedAllocator>,
    pub orig_raw: (usize, usize, usize), // (ptr, length, capacity)
    pub copy: Vec<T, &'static good_memory_allocator::SpinLockedAllocator>,
    pub copy_exists: bool
}

impl<'a, T:DRust> Default for DVecRef<'a, T> {
    fn default() -> Self {
        unsafe{
            DVecRef {
                orig_vec: &*(ptr::null() as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator>),
                orig_raw: (0, 0, 0),
                copy: Vec::new_in(&LOCAL_ALLOCATOR),
                copy_exists: false,
            }
        }
    }
}

impl<'a, T> Clone for DVecRef<'a, T>
where
    T: DRust,
{
    fn clone(&self) -> Self {
        DVecRef {
            orig_vec: self.orig_vec,
            orig_raw: self.orig_raw,
            copy: unsafe{Vec::new_in(&LOCAL_ALLOCATOR)},
            copy_exists: false,
        }
    }
}

impl<'a, T: DRust + Sized> DRust for DVecRef<'a, T> {
    fn static_typeid() -> u32 {
        (T::static_typeid() << 8) | 2
    }
    fn typeid(&self) -> u32 {
        (T::static_typeid() << 8) | 2
    }
    fn migrate(&mut self, dest: Destination) -> bool {
        false
    }
}

impl<'a, T: DRust + Sized> Drop for DVecRef<'a, T> {
    fn drop(&mut self) {
        self.drop_copy();
    }
}

impl<'a, T: DRust + Sized> DVecRef<'a, T> {
    pub fn len(&self) -> usize {
        self.orig_raw.1
    }

    pub(crate) fn drop_copy(&mut self) {
        if self.copy_exists {
            if self.copy.as_ptr() as usize == self.orig_raw.0 {

                let empty_copy = unsafe{Vec::new_in(&LOCAL_ALLOCATOR)};
                let mut v = mem::replace(&mut self.copy, empty_copy);
                let _ = v.into_raw_parts();
                return;
            }
            dprintln!("Dropping copy! Vec of Type: {:x}", T::static_typeid());
            let orig_addr = self.orig_raw.0;
            let empty_copy = unsafe{Vec::new_in(&LOCAL_ALLOCATOR)};
            let ref_map = unsafe { Arc::clone(REF_MAP.as_ref().unwrap()) };
            match ref_map.entry(orig_addr) {
                Entry::Occupied(mut entry) => {
                    
                    let mut v = mem::replace(&mut self.copy, empty_copy);
                    self.copy_exists = false;
                    dassert!(current_place(v.as_ptr() as usize) == Destination::Local, "Dropping a remote copy of a DVec copy");
                    
                    let (_, count) = entry.get_mut();
                    *count -= 1;
                    if(*count == 0) {
                        dprintln!(
                            "Dropping last ref to origin: {:x}, copy addr: {:x}",
                            orig_addr,
                            v.as_ptr() as usize
                        );
                        unsafe {
                            if (T::static_typeid() & 0xFF) != 23 {
                                v.set_len(0);
                            }
                        }
                        drop(v);
                        entry.remove();
                    }
                    else {
                        let _ = v.into_raw_parts();
                    }
                    
                }
                Entry::Vacant(_) => {
                    panic!("No entry in ref map");
                }
            };
        } else {
            dassert!(self.copy.capacity() == 0, "Capacity must be 0");
            dprintln!("No copy to drop");
            dprintln!("Copy addr in drop copy for DVecRef: {:x}", self.copy.as_ptr() as usize);
        }
    }

    pub fn local_copy(&self) {
        if self.copy_exists {
            dassert!(current_place(self.copy.as_ptr() as usize) == Destination::Local, "Already have a remote copy");
            return;
        }
        match current_place(self.orig_raw.0) {
            Destination::Local => {
                unsafe{
                    let v= Vec::from_raw_parts_in(
                        self.orig_raw.0 as *mut T,
                        self.orig_raw.1,
                        self.orig_raw.2,
                        &LOCAL_ALLOCATOR,
                    );
                    let dst_ptr = (&self.copy) as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as usize as *mut Vec<T, &'static good_memory_allocator::SpinLockedAllocator>;
                    std::ptr::write_volatile(dst_ptr, v);
                    std::ptr::write_volatile(&self.copy_exists as *const bool as usize as *mut bool, true);
                }
            }
            Destination::Remote(_) => {
                let ref_map = unsafe { Arc::clone(REF_MAP.as_ref().unwrap()) };
                match ref_map.entry(self.orig_raw.0) {
                    Entry::Occupied(mut entry) => {
                        dprintln!("Already have an entry in ref map");
                        let (ptr, count) = entry.get_mut();
                        *count += 1;
                        let v = unsafe {
                            Vec::from_raw_parts_in(
                                *ptr as *mut T,
                                self.orig_raw.1,
                                self.orig_raw.2,
                                &LOCAL_ALLOCATOR,
                            )
                        };
                        unsafe {
                            let dst_ptr = &self.copy as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as *mut Vec<T, &'static good_memory_allocator::SpinLockedAllocator>;
                            std::ptr::write_volatile(dst_ptr, v);
                            std::ptr::write_volatile(&self.copy_exists as *const bool as *mut bool, true);
                            dprintln!("Copy addr: {:x}", self.copy.as_ptr() as usize);
                        }
                        dprintln!(
                            "Copy addr: {:x}",
                            self.copy.as_ptr() as usize
                        );
                    }
                    Entry::Vacant(entry) => {
                        dprintln!("Creating a new entry in ref map");
                        let mut v: Vec<T, &good_memory_allocator::SpinLockedAllocator> =
                            unsafe { Vec::with_capacity_in(self.orig_raw.2, &LOCAL_ALLOCATOR) };
                        let ptr = v.as_ptr();
                        dprintln!("Copy addr: {:x}", ptr as usize);
                        dprintln!("orig_raw.0: {:x}", self.orig_raw.0);
                        dprintln!("orig_raw.1: {:x}", self.orig_raw.1);
                        dprintln!(
                            "thread id: {:x}",
                            thread::current().id().as_u64().get() as usize
                        );
                        unsafe {
                            drust_read_large_sync(
                                (ptr as usize) - LOCAL_HEAP_START,
                                self.orig_raw.0 - GLOBAL_HEAP_START,
                                self.orig_raw.1 * mem::size_of::<T>(),
                                thread::current().id().as_u64().get() as usize,
                            );
                            dprintln!("Copy addr: {:x}", ptr as usize);
                            v.set_len(self.orig_raw.1);
                        }
                        dprintln!("first 4 bytes: {:x}", unsafe { *(ptr as *const usize) });
                        unsafe {
                            let dst_ptr = (&self.copy) as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as usize as *mut Vec<T, &'static good_memory_allocator::SpinLockedAllocator>;
                            dprintln!("dst_ptr: {:x}", dst_ptr as usize);
                            std::ptr::write_volatile(dst_ptr, v);
                            dprintln!("copy_exists: {:x}", &self.copy_exists as *const bool as usize);
                            std::ptr::write_volatile(&self.copy_exists as *const bool as usize as *mut bool, true);
                        }
                        dprintln!(
                            "Copy addr: {:x}",
                            self.copy.as_ptr() as usize
                        );
                        entry.insert((ptr as usize, 1));
                    }
                };
            }
        };
        
    }

    pub fn as_regular(&self) -> &Vec<T, &'static good_memory_allocator::SpinLockedAllocator> {
        self.local_copy();
        &self.copy
    }
}


impl<T> Index<usize> for DVecRef<'_, T> 
where 
    T: DRust,
{
    type Output = T;

    fn index(&self, index: usize) -> &T {
        if current_place(self.orig_raw.0) == Destination::Local {
            unsafe{&*((self.orig_raw.0 as *const T).add(index))}
        } else {
            self.local_copy();
            &self.copy[index]
        }
    }
}

pub struct DVecMutRef<'a, T: DRust> {
    pub orig_vec: &'a Vec<T, &'static good_memory_allocator::SpinLockedAllocator>,
    pub orig_raw: (usize, usize, usize), // (ptr, length, capacity)
    pub copy: Option<Vec<T, &'static good_memory_allocator::SpinLockedAllocator>>,
}

impl<'a, T: DRust + Sized> Drop for DVecMutRef<'a, T> {
    fn drop(&mut self) {
        if self.copy.is_none() {
            return;
        }
        let orig_addr = self.orig_raw.0;
        let mut v = mem::replace(&mut self.copy, None).unwrap();
        if orig_addr == (v.as_ptr() as usize) {
            let _ = v.into_raw_parts();
            return;
        }
        if current_place(v.as_ptr() as usize) != Destination::Local {
            panic!("Dropping a remote copy of a DVecRef");
        }
        unsafe {
            let ptr = v.as_ptr();
            if v.len() != self.orig_raw.1 {
                panic!("Currently do not support length changing in DVecMutRef");
            }
            drust_write_large_sync(
                (ptr as usize) - LOCAL_HEAP_START,
                orig_addr - GLOBAL_HEAP_START,
                mem::size_of::<T>() * v.len(),
                thread::current().id().as_u64().get() as usize,
            );
            v.set_len(0);
            drop(v);
        }
    }
}

impl<'a, T: DRust + Sized> DVecMutRef<'a, T> {
    pub fn len(&self) -> usize {
        if self.copy.is_some() {
            self.copy.as_ref().unwrap().len()
        } else {
            self.orig_raw.1
        }
    }

    pub fn local_copy(&mut self) {
        match self.copy.as_ref() {
            Some(ptr) => {
                if current_place((*ptr).as_ptr() as usize) != Destination::Local {
                    panic!("Already have a remote copy");
                }
                return;
            }
            None => {}
        }
        match current_place(self.orig_raw.0) {
            Destination::Local => {
                self.copy = unsafe {
                    Some(Vec::from_raw_parts_in(
                        self.orig_raw.0 as *mut T,
                        self.orig_raw.1,
                        self.orig_raw.2,
                        &LOCAL_ALLOCATOR,
                    ))
                };
            }
            Destination::Remote(_server_idx) => {
                let mut v: Vec<T, &good_memory_allocator::SpinLockedAllocator> =
                    unsafe { Vec::with_capacity_in(self.orig_raw.2, &LOCAL_ALLOCATOR) };
                let ptr = v.as_ptr();
                unsafe {
                    drust_read_large_sync(
                        (ptr as usize) - LOCAL_HEAP_START,
                        self.orig_raw.0 - GLOBAL_HEAP_START,
                        self.orig_raw.1 * mem::size_of::<T>(),
                        thread::current().id().as_u64().get() as usize,
                    );
                    v.set_len(self.orig_raw.1);
                }
                self.copy = Some(v);
            }
        };
    }

    pub fn as_regular(
        &mut self,
    ) -> &mut Vec<T, &'static good_memory_allocator::SpinLockedAllocator> {
        self.local_copy();
        let v = self.copy.as_mut().unwrap();
        v
    }
}


impl<T:DRust> Borrow<Vec<T, &'static good_memory_allocator::SpinLockedAllocator>> for DVec<T> {
    fn borrow(&self) -> &Vec<T, &'static good_memory_allocator::SpinLockedAllocator> {
        unsafe{
            // &*(self.as_local_ref() as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as usize as *const Vec<T>)
            self.as_local_ref()
        }
    }
}

impl<T:DRust> AsRef<Vec<T, &'static good_memory_allocator::SpinLockedAllocator>> for DVec<T> {
    fn as_ref(&self) -> &Vec<T,  &'static good_memory_allocator::SpinLockedAllocator> {
        unsafe{
            // &*(self.as_local_ref() as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as usize as *const Vec<T>)
            self.as_local_ref()
            // &*(self.internal_vec.as_ref().unwrap() as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as usize as *const Vec<T>)
        }
    }
}

impl<T:DRust> AsMut<Vec<T, &'static good_memory_allocator::SpinLockedAllocator>> for DVec<T> {
    fn as_mut(&mut self) -> &mut Vec<T,  &'static good_memory_allocator::SpinLockedAllocator> {
        unsafe{
            self.as_local_mut_ref()
        }
    }
}

impl<T:DRust> AsRef<Vec<T, &'static good_memory_allocator::SpinLockedAllocator>> for DVecRef<'_, T> {
    fn as_ref(&self) -> &Vec<T,  &'static good_memory_allocator::SpinLockedAllocator> {
        unsafe{
            // &*(self.as_local_ref() as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as usize as *const Vec<T>)
            self.as_regular()
            // &*(self.internal_vec.as_ref().unwrap() as *const Vec<T, &'static good_memory_allocator::SpinLockedAllocator> as usize as *const Vec<T>)
        }
    }
}

impl<T:DRust> AsMut<Vec<T, &'static good_memory_allocator::SpinLockedAllocator>> for DVecMutRef<'_, T> {
    fn as_mut(&mut self) -> &mut Vec<T,  &'static good_memory_allocator::SpinLockedAllocator> {
        unsafe{
            self.as_regular()
        }
    }
}