use core::panic;

use crate::{app::{dataframe::prelude::{AnyType, Chunk, DataType, CHUNK_SIZE}, kv::entry::GlobalEntry, socialnet::media::Image}, drust_std::{alloc::*, collections::dvec::DVec, primitives::*}, exclude};

// use crate::{app::{dataframe::prelude::{Chunk, CHUNK_SIZE}, kvstore::GlobalEntry, sequential::{CHUNK_NUM, ELEMENT_UNIT_NUM}, socialnet::utils::FRAME_SIZE}, drust_std::alloc::LOCAL_ALLOCATOR, prelude::*};

exclude!(usize, 16);
exclude!(i32, 17);
exclude!(u8, 19);
exclude!(DataType, 20);
exclude!([u8; CHUNK_SIZE], 3);
exclude!(Chunk, 4);
exclude!(AnyType, 7);
exclude!(GlobalEntry, 6);
exclude!((), 64);
exclude!(Image, 21);


impl<T> DRust for (DVec<T>, DVec<T>, DVec<T>) 
    where T: DRust + Default + Send + 'static {

    fn static_typeid() -> u32 {
        (3 << 16) | (T::static_typeid() << 8) | 1
    }
    fn typeid(&self) -> u32 {
        (3 << 16) | (T::static_typeid() << 8) | 1
    }
    fn migrate(&mut self, _dst: Destination) -> bool {
        false
    }
}

impl<T> DRust for (DVec<T>, DVec<T>) 
    where T: DRust + Default + Send + 'static {

    fn static_typeid() -> u32 {
        (2 << 16) | (T::static_typeid() << 8) | 1
        
    }
    fn typeid(&self) -> u32 {
        (2 << 16) | (T::static_typeid() << 8) | 1
    }
    fn migrate(&mut self, _dst: Destination) -> bool {
        false
    }
}

pub fn drop_vec_with_id(type_id: u32, addr: usize, capacity: usize, len: usize) {
    match (type_id & 0xFF) {
        1 => {
            panic!("does not support drop a vector of disjoint pointer to remote vec!");
        },
        2 => {
            panic!("does not support drop a vector of disjoint pointer to remote vecref!");
        },
        3 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut [u8; CHUNK_SIZE], len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        4 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut Chunk, len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        // 5 => {
        //     let v = unsafe {Vec::from_raw_parts_in(addr as *mut [u8; FRAME_SIZE], len, capacity, &LOCAL_ALLOCATOR)};
        //     drop(v);
        // },
        6 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut GlobalEntry, len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        7 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut AnyType, len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        16 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut usize, len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        17 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut i32, len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        // // 18 => {
        // //     let v = unsafe {Vec::from_raw_parts_in(addr as *mut [usize; 1024], len, capacity, &LOCAL_ALLOCATOR)};
        // //     drop(v);
        // // },
        19 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut u8, len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        20 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut DataType, len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        21 => {
            let v = unsafe {Vec::from_raw_parts_in(addr as *mut Image, len, capacity, &LOCAL_ALLOCATOR)};
            drop(v);
        },
        _ => {
            panic!("unknown vec type id: {}", type_id);
        }
    }
}

pub fn from_id_to_type(
    type_id: u32,
    addr: usize,
) -> Box<dyn DRust, &'static good_memory_allocator::SpinLockedAllocator> {
    match (type_id & 0xFF) {
        1 => {
            panic!("does not support drop disjoint pointer to remote vec!");
        },
        2 => {
            panic!("does not support drop disjoint pointer to remote vecref!");
        },
        3 => {
            let v = unsafe {Box::from_raw_in(addr as *mut [u8; CHUNK_SIZE], &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 2, "type id: {}, compared value: {}", v.typeid(), 2);
            v
        },
        4 => {
            let v = unsafe {Box::from_raw_in(addr as *mut Chunk, &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 3, "type id: {}, compared value: {}", v.typeid(), 3);
            v
        },
        // 5 => {
        //     let v = unsafe {Box::from_raw_in(addr as *mut [u8; FRAME_SIZE], &LOCAL_ALLOCATOR)};
        //     debug_assert_eq!(v.typeid(), 5, "type id: {}, compared value: {}", v.typeid(), 5);
        //     v
        // },
        6 => {
            let v = unsafe {Box::from_raw_in(addr as *mut GlobalEntry, &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 6, "type id: {}, compared value: {}", v.typeid(), 6);
            v
        },
        7 => {
            let v = unsafe {Box::from_raw_in(addr as *mut AnyType, &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 7, "type id: {}, compared value: {}", v.typeid(), 7);
            v
        },
        16 => {
            let v = unsafe {Box::from_raw_in(addr as *mut usize, &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 16, "type id: {}, compared value: {}", v.typeid(), 16);
            v
        },
        17 => {
            let v = unsafe {Box::from_raw_in(addr as *mut i32, &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 17, "type id: {}, compared value: {}", v.typeid(), 17);
            v
        },
        // // 18 => {
        // //     let v = unsafe {Box::from_raw_in(addr as *mut [usize; 1024], &LOCAL_ALLOCATOR)};
        // //     debug_assert_eq!(v.typeid(), 18, "type id: {}, compared value: {}", v.typeid(), 18);
        // //     v
        // // },
        19 => {
            let v = unsafe {Box::from_raw_in(addr as *mut u8, &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 19, "type id: {}, compared value: {}", v.typeid(), 19);
            v
        },
        20 => {
            let v = unsafe {Box::from_raw_in(addr as *mut DataType, &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 20, "type id: {}, compared value: {}", v.typeid(), 20);
            v
        },
        21 => {
            let v = unsafe {Box::from_raw_in(addr as *mut Image, &LOCAL_ALLOCATOR)};
            debug_assert_eq!(v.typeid(), 21, "type id: {}, compared value: {}", v.typeid(), 21);
            v
        },
        _ => {
            panic!("unknown type id: {}", type_id);
        }
    }
}

// #[macro_export]
// macro_rules! dprintln {
//     ($($arg:tt)*) => {
//         {
//             println!($($arg)*);
//         }
//     }
// }

// #[macro_export]
// macro_rules! dassert {
//     ($($arg:tt)*) => {
//         {
//             assert!($($arg)*);
//         }
//     }
// }

#[macro_export]
macro_rules! dassert {
    ($($arg:tt)*) => {
        {
        }
    }
}


#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => {
        {
        }
    }
}
