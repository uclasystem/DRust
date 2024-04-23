use crate::{dprintln, drust_std::{alloc::LOCAL_ALLOCATOR, collections::dvec::DVec, primitives::dref::*}, exclude};
use super::super::{prelude::*, datatypes::AnyType, chunked_array::conf::CHUNK_SIZE};

pub struct Chunk {
    /// The number of elements in this array data
    pub(crate) len: usize,

    /// The offset into this array data
    pub(crate) element_size: usize,

    /// The buffers for this array data. Note that depending on the array types, this
    /// could hold different kinds of buffers (e.g., value buffer, value offset buffer)
    /// at different positions.
    pub(crate) buffer: DBox<[u8; CHUNK_SIZE]>,
}


impl Chunk {
    pub fn new(element_size: usize) -> Self {
        let temp_vec: DVec<u8> = DVec::with_capacity(CHUNK_SIZE);
        let v = unsafe {
            let (buffer, _ , _) =  temp_vec.into_raw_parts();
            DBox::from_raw(buffer as *mut [u8; CHUNK_SIZE])
        };
        Chunk {
            len: 0,
            element_size,
            buffer: v,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_full(&self) -> bool {
        self.len == CHUNK_SIZE / self.element_size
    }

    pub fn get(&self, index: usize) -> Vec<u8> {
        let offset = index * self.element_size;
        let tmp_buffer = self.buffer.clone();
        tmp_buffer[offset..offset + self.element_size].to_vec()
    }

    pub fn raw_get(&self, index: usize) -> Vec<u8> {
        let offset = index * self.element_size;
        self.buffer[offset..offset + self.element_size].to_vec()
    }


    pub fn get_four_bytes(&self, index: usize) -> [u8; 4] {
        let mut bytes = [0; 4];
        let offset = index * self.element_size;
        bytes[..self.element_size].copy_from_slice(
            &self.buffer[offset..offset + self.element_size]
        );
        bytes
    }

    pub fn get_four_bytes_from_buffer(&self, index: usize, buffer: &DBox<[u8; CHUNK_SIZE]>) -> [u8; 4] {
        let mut bytes = [0; 4];
        let offset = index * self.element_size;
        // dprintln!("offset: {}", offset);
        bytes[..self.element_size].copy_from_slice(
            &(**buffer)[offset..offset + self.element_size]
        );
        // dprintln!("src addr: {:p}", &(**buffer) as *const _);
        bytes
    }


    pub fn set(&mut self, index: usize, value: &[u8]) {
        let offset = index * self.element_size;
        self.buffer[offset..offset + self.element_size].copy_from_slice(value);
    }

    pub fn push(&mut self, value: &[u8]) -> usize {
        if self.is_full() {
            panic!("tried to push to full chunk");
        }
        let offset = self.len * self.element_size;
        self.buffer[offset..offset + self.element_size].copy_from_slice(value);
        self.len += 1;
        self.len - 1
    }

    pub fn copy_from_vec(&mut self, vec: &[u8]) {
        let len = vec.len();
        self.buffer[..len].copy_from_slice(vec);
        self.len = len / self.element_size;
    }

    // pub async fn dget_any(arr: &Self, index: usize, datatype: DataType) -> AnyType {
    //     Chunk::get_any(arr, index, datatype)
    // }

    pub fn get_any(arr: &Self, idx: usize, datatype: DataType) -> AnyType {
        match datatype {
            DataType::Boolean => AnyType::Boolean(if arr.get(idx)[0] == 0 { false } else { true }),
            DataType::UInt8 => AnyType::UInt8(arr.get(idx)[0]),
            DataType::UInt16 => AnyType::UInt16(convert_bytes_to_u16(arr.get(idx))),
            DataType::UInt32 => AnyType::UInt32(convert_bytes_to_u32(arr.get(idx))),
            DataType::UInt64 => AnyType::UInt64(convert_bytes_to_u64(arr.get(idx))),
            DataType::Int8 => AnyType::Int8(arr.get(idx)[0] as i8),
            DataType::Int16 => AnyType::Int16(convert_bytes_to_u16(arr.get(idx)) as i16),
            DataType::Int32 => AnyType::Int32(convert_bytes_to_u32(arr.get(idx)) as i32),
            DataType::Int64 => AnyType::Int64(convert_bytes_to_u64(arr.get(idx)) as i64),
            DataType::Float32 => AnyType::Float32(f32::from_bits(convert_bytes_to_u32(arr.get(idx)))),
            DataType::Float64 => AnyType::Float64(f64::from_bits(convert_bytes_to_u64(arr.get(idx)))),
            DataType::Date32(DateUnit::Day) => AnyType::Date32(convert_bytes_to_u32(arr.get(idx)) as i32),
            DataType::Date64(DateUnit::Millisecond) => AnyType::Date64(convert_bytes_to_u64(arr.get(idx)) as i64),
            DataType::Time32(TimeUnit::Millisecond) => {
                let v = convert_bytes_to_u32(arr.get(idx)) as i32;
                AnyType::Time32(v, TimeUnit::Millisecond)
            }
            DataType::Time32(TimeUnit::Second) => {
                let v = convert_bytes_to_u32(arr.get(idx)) as i32;
                AnyType::Time32(v, TimeUnit::Second)
            }
            DataType::Time64(TimeUnit::Nanosecond) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::Time64(v, TimeUnit::Nanosecond)
            }
            DataType::Time64(TimeUnit::Microsecond) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::Time64(v, TimeUnit::Microsecond)
            }
            DataType::Interval(IntervalUnit::DayTime) => {
                AnyType::IntervalDayTime(convert_bytes_to_u64(arr.get(idx)) as i64)
            }
            DataType::Interval(IntervalUnit::YearMonth) => {
                AnyType::IntervalYearMonth(convert_bytes_to_u32(arr.get(idx)) as i32)
            }
            DataType::Duration(TimeUnit::Nanosecond) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::Duration(v, TimeUnit::Nanosecond)
            }
            DataType::Duration(TimeUnit::Microsecond) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::Duration(v, TimeUnit::Microsecond)
            }
            DataType::Duration(TimeUnit::Millisecond) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::Duration(v, TimeUnit::Millisecond)
            }
            DataType::Duration(TimeUnit::Second) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::Duration(v, TimeUnit::Second)
            }
            DataType::Timestamp(TimeUnit::Nanosecond, _) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::TimeStamp(v, TimeUnit::Nanosecond)
            }
            DataType::Timestamp(TimeUnit::Microsecond, _) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::TimeStamp(v, TimeUnit::Microsecond)
            }
            DataType::Timestamp(TimeUnit::Millisecond, _) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::TimeStamp(v, TimeUnit::Millisecond)
            }
            DataType::Timestamp(TimeUnit::Second, _) => {
                let v = convert_bytes_to_u64(arr.get(idx)) as i64;
                AnyType::TimeStamp(v, TimeUnit::Second)
            }
            _ => unimplemented!(),
        }
    }
}
