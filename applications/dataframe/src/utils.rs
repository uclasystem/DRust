use std::sync::{atomic::AtomicBool, Condvar, Mutex};
use std::mem;
use std::ops::{Deref, DerefMut};

use super::self_arrow::DataType;



/// Used to split the mantissa and exponent of floating point numbers
/// https://stackoverflow.com/questions/39638363/how-can-i-use-a-hashmap-with-f64-as-key-in-rust
pub(crate) fn integer_decode(val: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { mem::transmute(val) };
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

pub(crate) fn floating_encode_f64(mantissa: u64, exponent: i16, sign: i8) -> f64 {
    sign as f64 * mantissa as f64 * (2.0f64).powf(exponent as f64)
}

#[macro_export]
macro_rules! exec_concurrent {
    ($block_a:block, $block_b:block) => {{
        thread::scope(|s| {
            let handle_left = s.spawn(|_| $block_a);
            let handle_right = s.spawn(|_| $block_b);
            let return_left = handle_left.join().expect("thread panicked");
            let return_right = handle_right.join().expect("thread panicked");
            (return_left, return_right)
        })
        .expect("could not join threads or thread panicked")
    }};
}

/// Just a wrapper structure. Useful for certain impl specializations
pub struct Xob<T> {
    inner: T,
}

impl<T> Xob<T> {
    pub fn new(inner: T) -> Self {
        Xob { inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Deref for Xob<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Xob<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn get_iter_capacity<T, I: Iterator<Item = T>>(iter: &I) -> usize {
    match iter.size_hint() {
        (_lower, Some(upper)) => upper,
        (0, None) => 1024,
        (lower, None) => lower,
    }
}

#[macro_export]
macro_rules! match_data_type_apply_macro {
    ($obj:expr, $macro:ident, $macro_utf8:ident) => {{
        match $obj {
            DataType::Utf8 => $macro_utf8!(),
            DataType::Boolean => $macro!(BooleanType),
            DataType::UInt8 => $macro!(UInt8Type),
            DataType::UInt16 => $macro!(UInt16Type),
            DataType::UInt32 => $macro!(UInt32Type),
            DataType::UInt64 => $macro!(UInt64Type),
            DataType::Int8 => $macro!(Int8Type),
            DataType::Int16 => $macro!(Int16Type),
            DataType::Int32 => $macro!(Int32Type),
            DataType::Int64 => $macro!(Int64Type),
            DataType::Float32 => $macro!(Float32Type),
            DataType::Float64 => $macro!(Float64Type),
            DataType::Date32(DateUnit::Day) => $macro!(Date32Type),
            DataType::Date64(DateUnit::Millisecond) => $macro!(Date64Type),
            DataType::Time32(TimeUnit::Millisecond) => $macro!(Time32MillisecondType),
            DataType::Time32(TimeUnit::Second) => $macro!(Time32SecondType),
            DataType::Time64(TimeUnit::Nanosecond) => $macro!(Time64NanosecondType),
            DataType::Time64(TimeUnit::Microsecond) => $macro!(Time64MicrosecondType),
            DataType::Interval(IntervalUnit::DayTime) => $macro!(IntervalDayTimeType),
            DataType::Interval(IntervalUnit::YearMonth) => $macro!(IntervalYearMonthType),
            DataType::Duration(TimeUnit::Nanosecond) => $macro!(DurationNanosecondType),
            DataType::Duration(TimeUnit::Microsecond) => $macro!(DurationMicrosecondType),
            DataType::Duration(TimeUnit::Millisecond) => $macro!(DurationMillisecondType),
            DataType::Duration(TimeUnit::Second) => $macro!(DurationSecondType),
            DataType::Timestamp(TimeUnit::Nanosecond, _) => $macro!(TimestampNanosecondType),
            DataType::Timestamp(TimeUnit::Microsecond, _) => $macro!(TimestampMicrosecondType),
            DataType::Timestamp(TimeUnit::Millisecond, _) => $macro!(Time32MillisecondType),
            DataType::Timestamp(TimeUnit::Second, _) => $macro!(TimestampSecondType),
            _ => unimplemented!(),
        }
    }};
}

pub fn convert_bytes_to_u16(bytes: Vec<u8>) -> u16 {
    let mut buf = [0u8; 2];
    buf.copy_from_slice(&bytes);
    u16::from_le_bytes(buf)
}

pub fn convert_bytes_to_u32(bytes: Vec<u8>) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes);
    u32::from_le_bytes(buf)
}

pub fn convert_bytes_to_u64(bytes: Vec<u8>) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes);
    u64::from_le_bytes(buf)
}

pub fn convert_u16_to_bytes(val: u16) -> Vec<u8> {
    val.to_le_bytes().to_vec()
}

pub fn convert_u32_to_bytes(val: u32) -> Vec<u8> {
    val.to_le_bytes().to_vec()
}

pub fn convert_u64_to_bytes(val: u64) -> Vec<u8> {
    val.to_le_bytes().to_vec()
}

pub fn datatype_size(d: &DataType) -> usize {
    match d {
        DataType::Boolean => 1,
        DataType::Int8 => 1,
        DataType::Int16 => 2,
        DataType::Int32 => 4,
        DataType::Int64 => 8,
        DataType::UInt8 => 1,
        DataType::UInt16 => 2,
        DataType::UInt32 => 4,
        DataType::UInt64 => 8,
        DataType::Float32 => 4,
        DataType::Float64 => 8,
        _ => { panic!("not implemented!");}
    }
}


pub static mut COMPUTES: Option<ResourceManager> = None;

pub struct ResourceManager {
    condvar: Condvar,
    resource_num: usize,
    avail_num: Mutex<usize>,
    avail_resources: Vec<AtomicBool>,
}
impl ResourceManager {
    pub fn new(num: usize) -> Self {
        let mut avail_resources = Vec::with_capacity(num);
        for _ in 0..num {
            avail_resources.push(AtomicBool::new(true));
        }
        ResourceManager {
            condvar: Condvar::new(),
            resource_num: num,
            avail_num: Mutex::new(num),
            avail_resources,
        }
    }

    pub fn get_resource(&self, start_id: usize) -> usize {
        let mut lock = self.avail_num.lock().unwrap();
        while *lock == 0 {
            lock = self.condvar.wait(lock).unwrap();
        }
        *lock -= 1;
        let mut rem = start_id % self.resource_num;
        let cycle = rem;
        loop {
            if self.avail_resources[rem]
                .compare_exchange(
                    true,
                    false,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
            {
                break;
            }
            rem = (rem + 1) % self.resource_num;
        }
        rem
    }

    pub fn release_resource(&self, res: usize) {
        self.avail_resources[res].store(true, std::sync::atomic::Ordering::SeqCst);
        let mut lock = self.avail_num.lock().unwrap();
        *lock += 1;
        self.condvar.notify_all();
    }
}