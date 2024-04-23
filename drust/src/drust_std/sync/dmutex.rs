use std::{alloc::{Allocator, Layout}, mem, ptr, sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize}, thread::{self, current}};
use dashmap::{mapref::entry::Entry, DashMap};
use futures::lock::Mutex;
use crate::{dassert, drust_std::{alloc::LOCAL_ALLOCATOR, comm::{drust_atomic_cmp_exchg_sync, drust_read_sync, drust_write_sync}, primitives::{current_place, dbox::DBox, DRust, Destination}, GLOBAL_HEAP_START, LOCAL_HEAP_START, NUM_SERVERS, SERVER_INDEX}, exclude};

// TODO: DMutex currently requires mlx 4.
// TODO: DMutex currently has lock priority issues.

pub static mut LOCAL_MUTEX_CACHE: Option<DashMap<usize, bool>> = None;

// pub struct InnerData<T: DRust + Sized> {
//     pub data: T,
//     pub forward: [usize; NUM_SERVERS],
// }

pub struct InnerMutex<T: DRust + Sized> {
    pub inner: *mut T,
    pub lock: AtomicUsize,
}

pub struct DMutex<T: DRust + Sized> {
    pub(crate) inner: *mut InnerMutex<T>,
    pub(crate) orig: *mut InnerMutex<T>,
}


unsafe impl<T: DRust + Sized + Send> Send for DMutex<T> {}
unsafe impl<T: DRust + Sized + Sync> Sync for DMutex<T> {}
// impl<T:DRust> DRust for InnerData<T> {
//     fn static_typeid() -> u32 where Self: Sized {
//         (T::static_typeid() << 8) | 21

//     }
//     fn typeid(&self) -> u32 {
//         (T::static_typeid() << 8) | 21
//     }
//     fn migrate(&mut self, _dst: Destination) -> bool {
//         false
//     }
// }

impl<T:DRust> DRust for InnerMutex<T> {
    fn static_typeid() -> u32 where Self: Sized {
        (T::static_typeid() << 8) | 22

    }
    fn typeid(&self) -> u32 {
        (T::static_typeid() << 8) | 22
    }
    fn migrate(&mut self, _dst: Destination) -> bool {
        false
    }
}

impl<T:DRust> DRust for DMutex<T> {
    fn static_typeid() -> u32 where Self: Sized {
        (T::static_typeid() << 8) | 23

    }
    fn typeid(&self) -> u32 {
        (T::static_typeid() << 8) | 23
    }
    fn migrate(&mut self, _dst: Destination) -> bool {
        false
    }
}

impl <T: DRust + Sized> InnerMutex<T>{
    pub fn local_copy(&mut self) {
        let current_addr = self.inner as usize;
        if current_place(current_addr) != Destination::Local {
            let new_addr = unsafe {
                LOCAL_ALLOCATOR
                    .allocate(Layout::new::<T>())
                    .unwrap()
                    .as_mut_ptr() as *mut T
            };
            unsafe {drust_read_sync(new_addr as usize - LOCAL_HEAP_START, 
                current_addr - GLOBAL_HEAP_START,
                mem::size_of::<T>(),
                thread::current().id().as_u64().get() as usize);}
            self.inner = new_addr;
        }
    }
}

impl<T: DRust + Sized> Drop for DMutex<T> {
    fn drop(&mut self) {
        if current_place(self.inner as usize) == Destination::Local {
            unsafe {
                LOCAL_MUTEX_CACHE.as_ref().unwrap().remove(&(self.orig as usize));
            }
        }
        if self.inner != self.orig {
            unsafe {
            let mut inner_mutex = DBox::from_raw(
                self.inner
            );
            inner_mutex.lock.store(0, std::sync::atomic::Ordering::SeqCst);
            let orig_addr = self.orig as usize;
            drust_write_sync(inner_mutex.get_addr() as usize - LOCAL_HEAP_START, 
                orig_addr - GLOBAL_HEAP_START,
                mem::size_of::<InnerMutex<T>>(),
                thread::current().id().as_u64().get() as usize);}
        }
    }
    
}

impl<T: DRust + Sized> DMutex<T>{
    // pub fn new_uninit() -> Self {
    //     let mut result = Self {
    //         inner: std::ptr::null_mut(),
    //         orig: std::ptr::null_mut(),
    //     };
    //     result
    // }

    pub fn new(data: T) -> Self {
        let inner_data = DBox::new(data);
        let mut inner_mutex = DBox::new(InnerMutex {
            inner: unsafe{inner_data.into_raw()},
            lock: AtomicUsize::new(0),
        });
        unsafe {
            let ptr = inner_mutex.into_raw();
            Self {
                inner: ptr,
                orig: ptr,
            }
        }
    }

    // TODO: has concurrency issues for concurrency level > 3
    pub fn lock(&self) -> &mut T {
        if current_place(self.inner as usize) == Destination::Local {
            let inner_mutex = unsafe { &mut *self.inner };
            loop {
                if inner_mutex.lock.compare_exchange(0, 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_ok() {
                    inner_mutex.local_copy();
                    return unsafe { &mut *inner_mutex.inner };
                }
            }
        }
        let orig_addr = self.orig as usize;
        match unsafe{LOCAL_MUTEX_CACHE.as_ref().unwrap().entry(orig_addr)} {
            Entry::Occupied(mut entry) => {
                let current_addr = self.inner as usize;
                dassert!(current_place(current_addr) == Destination::Local);
                let inner_mutex = unsafe { &mut *self.inner };
                loop {
                    if inner_mutex.lock.compare_exchange(0, 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_ok() {
                        inner_mutex.local_copy();
                        return unsafe { &mut *inner_mutex.inner };
                    }
                }
            },
            Entry::Vacant(entry) => {
                let mut inner_mutex = DBox::new(InnerMutex {
                    inner: ptr::null_mut(),
                    lock: AtomicUsize::new(0),
                });
                unsafe {
                    let local_src_offset = &mut inner_mutex.lock as *mut AtomicUsize as *mut usize as usize - LOCAL_HEAP_START;
                    let remote_dst_offset = orig_addr - GLOBAL_HEAP_START + mem::size_of::<*mut T>();
                    loop {
                        inner_mutex.lock.store(1, std::sync::atomic::Ordering::SeqCst);
                        drust_atomic_cmp_exchg_sync(local_src_offset, remote_dst_offset, 0, 1, thread::current().id().as_u64().get() as usize);
                        if inner_mutex.lock.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                            break;
                        }
                    }
                    drust_read_sync(inner_mutex.get_addr() as usize - LOCAL_HEAP_START, 
                        orig_addr - GLOBAL_HEAP_START,
                        mem::size_of::<InnerMutex<T>>(),
                        thread::current().id().as_u64().get() as usize);
                    inner_mutex.lock.store(0, std::sync::atomic::Ordering::SeqCst);
                    ptr::write_volatile(&self.inner as *const *mut InnerMutex<T> as *mut *mut InnerMutex<T>, inner_mutex.into_raw());
                }
                entry.insert(true);
                let inner_mutex = unsafe { &mut *self.inner };
                loop {
                    if inner_mutex.lock.compare_exchange(0, 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_ok() {
                        inner_mutex.local_copy();
                        return unsafe { &mut *(inner_mutex.inner) };
                    }
                }

            }
        }
    }

    pub fn unlock(&self, r: &mut T) {
        let inner_mutex = unsafe { &mut *self.inner };
        inner_mutex.lock.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}