use super::{chunked_array::ChunkedArray, self_arrow::*};
use serde::{Deserialize, Serialize};
use std::{any::Any, sync::Arc};

#[derive(Clone)]
pub struct Utf8Type {}

// pub struct LargeListType {}

pub trait PolarsDataType {
    fn get_data_type() -> DataType;
}

impl<T> PolarsDataType for T
where
    T: PrimitiveType,
{
    fn get_data_type() -> DataType {
        T::get_data_type()
    }
}

// impl PolarsDataType for Utf8Type {
//     fn get_data_type() -> DataType {
//         DataType::Utf8
//     }
// }

// impl PolarsDataType for LargeListType {
//     fn get_data_type() -> DataType {
//         // null as we cannot no anything without self.
//         DataType::LargeList(Box::new(DataType::Null))
//     }
// }

/// Any type that is not nested
pub trait PolarsSingleType: PolarsDataType {}

impl<T> PolarsSingleType for T where T: PrimitiveType + PolarsDataType {}

// impl PolarsSingleType for Utf8Type {}

pub trait PolarsNumericType: PrimitiveType {}

impl PolarsNumericType for UInt8Type {}
impl PolarsNumericType for UInt16Type {}
impl PolarsNumericType for UInt32Type {}
impl PolarsNumericType for UInt64Type {}
impl PolarsNumericType for Int8Type {}
impl PolarsNumericType for Int16Type {}
impl PolarsNumericType for Int32Type {}
impl PolarsNumericType for Int64Type {}
impl PolarsNumericType for Float32Type {}
impl PolarsNumericType for Float64Type {}
impl PolarsNumericType for Date32Type {}
impl PolarsNumericType for Date64Type {}
impl PolarsNumericType for Time64NanosecondType {}
impl PolarsNumericType for Time64MicrosecondType {}
impl PolarsNumericType for Time32MillisecondType {}
impl PolarsNumericType for Time32SecondType {}
impl PolarsNumericType for DurationNanosecondType {}
impl PolarsNumericType for DurationMicrosecondType {}
impl PolarsNumericType for DurationMillisecondType {}
impl PolarsNumericType for DurationSecondType {}
impl PolarsNumericType for IntervalYearMonthType {}
impl PolarsNumericType for IntervalDayTimeType {}
impl PolarsNumericType for TimestampNanosecondType {}
impl PolarsNumericType for TimestampMicrosecondType {}
impl PolarsNumericType for TimestampMillisecondType {}
impl PolarsNumericType for TimestampSecondType {}

pub trait PolarsIntegerType: PolarsNumericType {}
impl PolarsIntegerType for UInt8Type {}
impl PolarsIntegerType for UInt16Type {}
impl PolarsIntegerType for UInt32Type {}
impl PolarsIntegerType for UInt64Type {}
impl PolarsIntegerType for Int8Type {}
impl PolarsIntegerType for Int16Type {}
impl PolarsIntegerType for Int32Type {}
impl PolarsIntegerType for Int64Type {}
impl PolarsIntegerType for Date32Type {}
impl PolarsIntegerType for Date64Type {}
impl PolarsIntegerType for Time64NanosecondType {}
impl PolarsIntegerType for Time64MicrosecondType {}
impl PolarsIntegerType for Time32MillisecondType {}
impl PolarsIntegerType for Time32SecondType {}
impl PolarsIntegerType for DurationNanosecondType {}
impl PolarsIntegerType for DurationMicrosecondType {}
impl PolarsIntegerType for DurationMillisecondType {}
impl PolarsIntegerType for DurationSecondType {}
impl PolarsIntegerType for IntervalYearMonthType {}
impl PolarsIntegerType for IntervalDayTimeType {}
impl PolarsIntegerType for TimestampNanosecondType {}
impl PolarsIntegerType for TimestampMicrosecondType {}
impl PolarsIntegerType for TimestampMillisecondType {}
impl PolarsIntegerType for TimestampSecondType {}

// pub type LargeListChunked = ChunkedArray<LargeListType>;
pub type BooleanChunked = ChunkedArray<BooleanType>;
pub type UInt8Chunked = ChunkedArray<UInt8Type>;
pub type UInt16Chunked = ChunkedArray<UInt16Type>;
pub type UInt32Chunked = ChunkedArray<UInt32Type>;
pub type UInt64Chunked = ChunkedArray<UInt64Type>;
pub type Int8Chunked = ChunkedArray<Int8Type>;
pub type Int16Chunked = ChunkedArray<Int16Type>;
pub type Int32Chunked = ChunkedArray<Int32Type>;
pub type Int64Chunked = ChunkedArray<Int64Type>;
pub type Float32Chunked = ChunkedArray<Float32Type>;
pub type Float64Chunked = ChunkedArray<Float64Type>;
// pub type Utf8Chunked = ChunkedArray<Utf8Type>;
pub type Date32Chunked = ChunkedArray<Date32Type>;
pub type Date64Chunked = ChunkedArray<Date64Type>;
pub type DurationNanosecondChunked = ChunkedArray<DurationNanosecondType>;
pub type DurationMicrosecondChunked = ChunkedArray<DurationMicrosecondType>;
pub type DurationMillisecondChunked = ChunkedArray<DurationMillisecondType>;
pub type DurationSecondChunked = ChunkedArray<DurationSecondType>;

pub type Time64NanosecondChunked = ChunkedArray<Time64NanosecondType>;
pub type Time64MicrosecondChunked = ChunkedArray<Time64MicrosecondType>;
pub type Time32MillisecondChunked = ChunkedArray<Time32MillisecondType>;
pub type Time32SecondChunked = ChunkedArray<Time32SecondType>;
pub type IntervalDayTimeChunked = ChunkedArray<IntervalDayTimeType>;
pub type IntervalYearMonthChunked = ChunkedArray<IntervalYearMonthType>;

pub type TimestampNanosecondChunked = ChunkedArray<TimestampNanosecondType>;
pub type TimestampMicrosecondChunked = ChunkedArray<TimestampMicrosecondType>;
pub type TimestampMillisecondChunked = ChunkedArray<TimestampMillisecondType>;
pub type TimestampSecondChunked = ChunkedArray<TimestampSecondType>;

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum AnyType {
    #[default]
    Null,
    /// A binary true or false.
    Boolean(bool),
    /// A UTF8 encoded string type.
    // Utf8(&'a str),
    /// An unsigned 8-bit integer number.
    UInt8(u8),
    /// An unsigned 16-bit integer number.
    UInt16(u16),
    /// An unsigned 32-bit integer number.
    UInt32(u32),
    /// An unsigned 64-bit integer number.
    UInt64(u64),
    /// An 8-bit integer number.
    Int8(i8),
    /// A 16-bit integer number.
    Int16(i16),
    /// A 32-bit integer number.
    Int32(i32),
    /// A 64-bit integer number.
    Int64(i64),
    /// A 32-bit floating point number.
    Float32(f32),
    /// A 64-bit floating point number.
    Float64(f64),
    /// A 32-bit date representing the elapsed time since UNIX epoch (1970-01-01)
    /// in days (32 bits).
    Date32(i32),
    /// A 64-bit date representing the elapsed time since UNIX epoch (1970-01-01)
    /// in milliseconds (64 bits).
    Date64(i64),
    /// A 64-bit time representing the elapsed time since midnight in the unit of `TimeUnit`.
    Time64(i64, TimeUnit),
    /// A 32-bit time representing the elapsed time since midnight in the unit of `TimeUnit`.
    Time32(i32, TimeUnit),
    /// Measure of elapsed time in either seconds, milliseconds, microseconds or nanoseconds.
    Duration(i64, TimeUnit),
    /// Naive Time elapsed from the Unix epoch, 00:00:00.000 on 1 January 1970, excluding leap seconds, as a 64-bit integer.
    /// Note that UNIX time does not include leap seconds.
    TimeStamp(i64, TimeUnit),
    /// A "calendar" interval which models types that don't necessarily have a precise duration without the context of a base timestamp
    /// (e.g. days can differ in length during day light savings time transitions).
    IntervalDayTime(i64),
    IntervalYearMonth(i32),
    // LargeList(Series),
}

pub trait ToStr {
    fn to_str(&self) -> String;
}

impl ToStr for DataType {
    fn to_str(&self) -> String {
        // TODO: add types here
        let s = match self {
            DataType::Null => "null",
            DataType::Boolean => "bool",
            DataType::UInt8 => "u8",
            DataType::UInt16 => "u16",
            DataType::UInt32 => "u32",
            DataType::UInt64 => "u64",
            DataType::Int8 => "i8",
            DataType::Int16 => "i16",
            DataType::Int32 => "i32",
            DataType::Int64 => "i64",
            DataType::Float32 => "f32",
            DataType::Float64 => "f64",
            // DataType::Utf8 => "str",
            DataType::Date32(DateUnit::Day) => "date32",
            DataType::Date64(DateUnit::Millisecond) => "date64",
            DataType::Time32(TimeUnit::Second) => "time64(s)",
            DataType::Time32(TimeUnit::Millisecond) => "time64(ms)",
            DataType::Time64(TimeUnit::Nanosecond) => "time64(ns)",
            DataType::Time64(TimeUnit::Microsecond) => "time64(μs)",
            // Note: Polars doesn't support the optional TimeZone in the timestamps.
            DataType::Timestamp(TimeUnit::Nanosecond, _) => "timestamp(ns)",
            DataType::Timestamp(TimeUnit::Microsecond, _) => "timestamp(μs)",
            DataType::Timestamp(TimeUnit::Millisecond, _) => "timestamp(ms)",
            DataType::Timestamp(TimeUnit::Second, _) => "timestamp(s)",
            DataType::Duration(TimeUnit::Nanosecond) => "duration(ns)",
            DataType::Duration(TimeUnit::Microsecond) => "duration(μs)",
            DataType::Duration(TimeUnit::Millisecond) => "duration(ms)",
            DataType::Duration(TimeUnit::Second) => "duration(s)",
            DataType::Interval(IntervalUnit::DayTime) => "interval(daytime)",
            DataType::Interval(IntervalUnit::YearMonth) => "interval(year-month)",
            // DataType::LargeList(tp) => return format!("list [{}]", tp.to_str()),
            _ => panic!("{:?} not implemented", self),
        };
        s.into()
    }
}

impl PartialEq for AnyType {
    // Everything of Any is slow. Don't use.
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl AnyType {
    pub fn to_string(&self) -> String {
        match self {
            AnyType::Null => String::from("null"),
            AnyType::Boolean(v) => format!("{}", v),
            AnyType::UInt8(v) => format!("{}", v),
            AnyType::UInt16(v) => format!("{}", v),
            AnyType::UInt32(v) => format!("{}", v),
            AnyType::UInt64(v) => format!("{}", v),
            AnyType::Int8(v) => format!("{}", v),
            AnyType::Int16(v) => format!("{}", v),
            AnyType::Int32(v) => format!("{}", v),
            AnyType::Int64(v) => format!("{}", v),
            AnyType::Float32(v) => format!("{:.3}", v),
            AnyType::Float64(v) => format!("{:.3}", v),
            AnyType::Date32(v) => format!("{}", v),
            AnyType::Date64(v) => format!("{}", v),
            AnyType::Time32(v, u) => format!("{}({:?})", v, u),
            AnyType::Time64(v, u) => format!("{}({:?})", v, u),
            AnyType::Duration(v, u) => format!("{}({:?})", v, u),
            AnyType::TimeStamp(v, u) => format!("{}({:?})", v, u),
            AnyType::IntervalDayTime(v) => format!("{}", v),
            AnyType::IntervalYearMonth(v) => format!("{}", v),
        }
    }

    pub fn to_num(self) -> f64 {
        match self {
            AnyType::Null => 0.0,
            AnyType::Boolean(v) => v as u8 as f64,
            AnyType::UInt8(v) => v as f64,
            AnyType::UInt16(v) => v as f64,
            AnyType::UInt32(v) => v as f64,
            AnyType::UInt64(v) => v as f64,
            AnyType::Int8(v) => v as f64,
            AnyType::Int16(v) => v as f64,
            AnyType::Int32(v) => v as f64,
            AnyType::Int64(v) => v as f64,
            AnyType::Float32(v) => v as f64,
            AnyType::Float64(v) => v,
            AnyType::Date32(v) => v as f64,
            AnyType::Date64(v) => v as f64,
            AnyType::Time32(v, _) => v as f64,
            AnyType::Time64(v, _) => v as f64,
            AnyType::Duration(v, _) => v as f64,
            AnyType::TimeStamp(v, _) => v as f64,
            AnyType::IntervalDayTime(v) => v as f64,
            AnyType::IntervalYearMonth(v) => v as f64,
        }
    }
}

#[macro_export]
macro_rules! datatype_to_anytype {
    ($datatype:expr, $value:expr) => {
        match $datatype {
            DataType::Boolean => AnyType::Boolean($value),
            DataType::UInt8 => AnyType::UInt8($value),
            DataType::UInt16 => AnyType::UInt16($value),
            DataType::UInt32 => AnyType::UInt32($value),
            DataType::UInt64 => AnyType::UInt64($value),
            DataType::Int8 => AnyType::Int8($value),
            DataType::Int16 => AnyType::Int16($value),
            DataType::Int32 => AnyType::Int32($value),
            DataType::Int64 => AnyType::Int64($value),
            DataType::Float32 => AnyType::Float32($value),
            DataType::Float64 => AnyType::Float64($value),
            DataType::Date32(DateUnit::Day) => AnyType::Date32($value),
            DataType::Date64(DateUnit::Millisecond) => AnyType::Date64($value),
            DataType::Time32(TimeUnit::Millisecond) => {
                AnyType::Time32($value, TimeUnit::Millisecond)
            }
            DataType::Time32(TimeUnit::Second) => AnyType::Time32($value, TimeUnit::Second),
            DataType::Time64(TimeUnit::Nanosecond) => AnyType::Time64($value, TimeUnit::Nanosecond),
            DataType::Time64(TimeUnit::Microsecond) => {
                AnyType::Time64($value, TimeUnit::Microsecond)
            }
            DataType::Interval(IntervalUnit::DayTime) => AnyType::IntervalDayTime($value),
            DataType::Interval(IntervalUnit::YearMonth) => AnyType::IntervalYearMonth($value),
            DataType::Duration(TimeUnit::Nanosecond) => {
                AnyType::Duration($value, TimeUnit::Nanosecond)
            }
            DataType::Duration(TimeUnit::Microsecond) => {
                AnyType::Duration($value, TimeUnit::Microsecond)
            }
            DataType::Duration(TimeUnit::Millisecond) => {
                AnyType::Duration($value, TimeUnit::Millisecond)
            }
            DataType::Duration(TimeUnit::Second) => AnyType::Duration($value, TimeUnit::Second),
            DataType::Timestamp(TimeUnit::Nanosecond, _) => {
                AnyType::TimeStamp($value, TimeUnit::Nanosecond)
            }
            DataType::Timestamp(TimeUnit::Microsecond, _) => {
                AnyType::TimeStamp($value, TimeUnit::Microsecond)
            }
            DataType::Timestamp(TimeUnit::Millisecond, _) => {
                AnyType::TimeStamp($value, TimeUnit::Millisecond)
            }
            DataType::Timestamp(TimeUnit::Second, _) => {
                AnyType::TimeStamp($value, TimeUnit::Second)
            }
            _ => panic!("not implemented"),
        }
    };
}
