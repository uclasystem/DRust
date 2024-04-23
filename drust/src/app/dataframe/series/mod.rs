pub mod input;

use std::time::Duration;
use enum_dispatch::enum_dispatch;
use tarpc::context;

use super::{
    chunked_array::ChunkedArray,
    prelude::*,
};
use crate::{dprintln, drust_std::{collections::dvec::{DVec, DVecMutRef, DVecRef}, thread::dspawn, utils::*}};

#[enum_dispatch]
pub enum Series {
    UInt8(ChunkedArray<UInt8Type>),
    UInt16(ChunkedArray<UInt16Type>),
    UInt32(ChunkedArray<UInt32Type>),
    UInt64(ChunkedArray<UInt64Type>),
    Int8(ChunkedArray<Int8Type>),
    Int16(ChunkedArray<Int16Type>),
    Int32(ChunkedArray<Int32Type>),
    Int64(ChunkedArray<Int64Type>),
    Float32(ChunkedArray<Float32Type>),
    Float64(ChunkedArray<Float64Type>),
    // Utf8(ChunkedArray<Utf8Type>),
    Bool(ChunkedArray<BooleanType>),
    Date32(ChunkedArray<Date32Type>),
    Date64(ChunkedArray<Date64Type>),
    Time32Millisecond(Time32MillisecondChunked),
    Time32Second(Time32SecondChunked),
    Time64Nanosecond(ChunkedArray<Time64NanosecondType>),
    Time64Microsecond(ChunkedArray<Time64MicrosecondType>),
    DurationNanosecond(ChunkedArray<DurationNanosecondType>),
    DurationMicrosecond(DurationMicrosecondChunked),
    DurationMillisecond(DurationMillisecondChunked),
    DurationSecond(DurationSecondChunked),
    IntervalDayTime(IntervalDayTimeChunked),
    IntervalYearMonth(IntervalYearMonthChunked),
    TimestampNanosecond(TimestampNanosecondChunked),
    TimestampMicrosecond(TimestampMicrosecondChunked),
    TimestampMillisecond(TimestampMillisecondChunked),
    TimestampSecond(TimestampSecondChunked),
    // LargeList(LargeListChunked),
}

#[macro_export]
macro_rules! apply_method_all_series {
    ($self:ident, $method:ident, $($args:expr),*) => {
        match $self {
            // Series::Utf8(a) => a.$method($($args),*),
            Series::Bool(a) => a.$method($($args),*),
            Series::UInt8(a) => a.$method($($args),*),
            Series::UInt16(a) => a.$method($($args),*),
            Series::UInt32(a) => a.$method($($args),*),
            Series::UInt64(a) => a.$method($($args),*),
            Series::Int8(a) => a.$method($($args),*),
            Series::Int16(a) => a.$method($($args),*),
            Series::Int32(a) => a.$method($($args),*),
            Series::Int64(a) => a.$method($($args),*),
            Series::Float32(a) => a.$method($($args),*),
            Series::Float64(a) => a.$method($($args),*),
            Series::Date32(a) => a.$method($($args),*),
            Series::Date64(a) => a.$method($($args),*),
            Series::Time32Millisecond(a) => a.$method($($args),*),
            Series::Time32Second(a) => a.$method($($args),*),
            Series::Time64Nanosecond(a) => a.$method($($args),*),
            Series::Time64Microsecond(a) => a.$method($($args),*),
            Series::DurationNanosecond(a) => a.$method($($args),*),
            Series::DurationMicrosecond(a) => a.$method($($args),*),
            Series::DurationMillisecond(a) => a.$method($($args),*),
            Series::DurationSecond(a) => a.$method($($args),*),
            Series::TimestampNanosecond(a) => a.$method($($args),*),
            Series::TimestampMicrosecond(a) => a.$method($($args),*),
            Series::TimestampMillisecond(a) => a.$method($($args),*),
            Series::TimestampSecond(a) => a.$method($($args),*),
            Series::IntervalDayTime(a) => a.$method($($args),*),
            Series::IntervalYearMonth(a) => a.$method($($args),*),
            // Series::LargeList(a) => a.$method($($args),*),
        }
    }
}

#[macro_export]
macro_rules! apply_async_method_all_series {
    ($self:ident, $method:ident, $($args:expr),*) => {
        match $self {
            // Series::Utf8(a) => a.$method($($args),*),
            Series::Bool(a) => a.$method($($args),*).await,
            Series::UInt8(a) => a.$method($($args),*).await,
            Series::UInt16(a) => a.$method($($args),*).await,
            Series::UInt32(a) => a.$method($($args),*).await,
            Series::UInt64(a) => a.$method($($args),*).await,
            Series::Int8(a) => a.$method($($args),*).await,
            Series::Int16(a) => a.$method($($args),*).await,
            Series::Int32(a) => a.$method($($args),*).await,
            Series::Int64(a) => a.$method($($args),*).await,
            Series::Float32(a) => a.$method($($args),*).await,
            Series::Float64(a) => a.$method($($args),*).await,
            Series::Date32(a) => a.$method($($args),*).await,
            Series::Date64(a) => a.$method($($args),*).await,
            Series::Time32Millisecond(a) => a.$method($($args),*).await,
            Series::Time32Second(a) => a.$method($($args),*).await,
            Series::Time64Nanosecond(a) => a.$method($($args),*).await,
            Series::Time64Microsecond(a) => a.$method($($args),*).await,
            Series::DurationNanosecond(a) => a.$method($($args),*).await,
            Series::DurationMicrosecond(a) => a.$method($($args),*).await,
            Series::DurationMillisecond(a) => a.$method($($args),*).await,
            Series::DurationSecond(a) => a.$method($($args),*).await,
            Series::TimestampNanosecond(a) => a.$method($($args),*).await,
            Series::TimestampMicrosecond(a) => a.$method($($args),*).await,
            Series::TimestampMillisecond(a) => a.$method($($args),*).await,
            Series::TimestampSecond(a) => a.$method($($args),*).await,
            Series::IntervalDayTime(a) => a.$method($($args),*).await,
            Series::IntervalYearMonth(a) => a.$method($($args),*).await,
            // Series::LargeList(a) => a.$method($($args),*),
        }
    }
}

macro_rules! apply_method_all_series_and_return {
    ($self:ident, $method:ident, [$($args:expr),*], $($opt_question_mark:tt)*) => {
        match $self {
            Series::UInt8(a) => Series::UInt8(a.$method($($args),*)$($opt_question_mark)*),
            Series::UInt16(a) => Series::UInt16(a.$method($($args),*)$($opt_question_mark)*),
            Series::UInt32(a) => Series::UInt32(a.$method($($args),*)$($opt_question_mark)*),
            Series::UInt64(a) => Series::UInt64(a.$method($($args),*)$($opt_question_mark)*),
            Series::Int8(a) => Series::Int8(a.$method($($args),*)$($opt_question_mark)*),
            Series::Int16(a) => Series::Int16(a.$method($($args),*)$($opt_question_mark)*),
            Series::Int32(a) => Series::Int32(a.$method($($args),*)$($opt_question_mark)*),
            Series::Int64(a) => Series::Int64(a.$method($($args),*)$($opt_question_mark)*),
            Series::Float32(a) => Series::Float32(a.$method($($args),*)$($opt_question_mark)*),
            Series::Float64(a) => Series::Float64(a.$method($($args),*)$($opt_question_mark)*),
            // Series::Utf8(a) => Series::Utf8(a.$method($($args),*)$($opt_question_mark)*),
            Series::Bool(a) => Series::Bool(a.$method($($args),*)$($opt_question_mark)*),
            Series::Date32(a) => Series::Date32(a.$method($($args),*)$($opt_question_mark)*),
            Series::Date64(a) => Series::Date64(a.$method($($args),*)$($opt_question_mark)*),
            Series::Time32Millisecond(a) => Series::Time32Millisecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::Time32Second(a) => Series::Time32Second(a.$method($($args),*)$($opt_question_mark)*),
            Series::Time64Nanosecond(a) => Series::Time64Nanosecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::Time64Microsecond(a) => Series::Time64Microsecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::DurationNanosecond(a) => Series::DurationNanosecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::DurationMicrosecond(a) => Series::DurationMicrosecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::DurationMillisecond(a) => Series::DurationMillisecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::DurationSecond(a) => Series::DurationSecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::TimestampNanosecond(a) => Series::TimestampNanosecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::TimestampMicrosecond(a) => Series::TimestampMicrosecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::TimestampMillisecond(a) => Series::TimestampMillisecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::TimestampSecond(a) => Series::TimestampSecond(a.$method($($args),*)$($opt_question_mark)*),
            Series::IntervalDayTime(a) => Series::IntervalDayTime(a.$method($($args),*)$($opt_question_mark)*),
            Series::IntervalYearMonth(a) => Series::IntervalYearMonth(a.$method($($args),*)$($opt_question_mark)*),
            // Series::LargeList(a) => Series::LargeList(a.$method($($args),*)$($opt_question_mark)*),
        }
    }
}

#[macro_export]
macro_rules! apply_datatype_to_series {
    ($datatype:expr, $method:ident, $($args:expr),*) => {
        match $datatype {
            DataType::Boolean => Series::Bool(ChunkedArray::<BooleanType>::$method($($args),*)),
            DataType::UInt8 => Series::UInt8(ChunkedArray::<UInt8Type>::$method($($args),*)),
            DataType::UInt16 => Series::UInt16(ChunkedArray::<UInt16Type>::$method($($args),*)),
            DataType::UInt32 => Series::UInt32(ChunkedArray::<UInt32Type>::$method($($args),*)),
            DataType::UInt64 => Series::UInt64(ChunkedArray::<UInt64Type>::$method($($args),*)),
            DataType::Int8 => Series::Int8(ChunkedArray::<Int8Type>::$method($($args),*)),
            DataType::Int16 => Series::Int16(ChunkedArray::<Int16Type>::$method($($args),*)),
            DataType::Int32 => Series::Int32(ChunkedArray::<Int32Type>::$method($($args),*)),
            DataType::Int64 => Series::Int64(ChunkedArray::<Int64Type>::$method($($args),*)),
            DataType::Float32 => Series::Float32(ChunkedArray::<Float32Type>::$method($($args),*)),
            DataType::Float64 => Series::Float64(ChunkedArray::<Float64Type>::$method($($args),*)),
            // DataType::Utf8 => Series::Utf8(ChunkedArray::<Utf8Type>::$method($($args),*)),
            DataType::Date32(DateUnit::Day) => Series::Date32(ChunkedArray::<Date32Type>::$method($($args),*)),
            DataType::Date64(DateUnit::Millisecond) => Series::Date64(ChunkedArray::<Date64Type>::$method($($args),*)),
            DataType::Time32(TimeUnit::Millisecond) => Series::Time32Millisecond(ChunkedArray::<Time32MillisecondType>::$method($($args),*)),
            DataType::Time32(TimeUnit::Second) => Series::Time32Second(ChunkedArray::<Time32SecondType>::$method($($args),*)),
            DataType::Time64(TimeUnit::Nanosecond) => Series::Time64Nanosecond(ChunkedArray::<Time64NanosecondType>::$method($($args),*)),
            DataType::Time64(TimeUnit::Microsecond) => Series::Time64Microsecond(ChunkedArray::<Time64MicrosecondType>::$method($($args),*)),
            DataType::Duration(TimeUnit::Nanosecond) => Series::DurationNanosecond(ChunkedArray::<DurationNanosecondType>::$method($($args),*)),
            DataType::Duration(TimeUnit::Microsecond) => Series::DurationMicrosecond(ChunkedArray::<DurationMicrosecondType>::$method($($args),*)),
            DataType::Duration(TimeUnit::Millisecond) => Series::DurationMillisecond(ChunkedArray::<DurationMillisecondType>::$method($($args),*)),
            DataType::Duration(TimeUnit::Second) => Series::DurationSecond(ChunkedArray::<DurationSecondType>::$method($($args),*)),
            DataType::Timestamp(TimeUnit::Nanosecond, _) => Series::TimestampNanosecond(ChunkedArray::<TimestampNanosecondType>::$method($($args),*)),
            DataType::Timestamp(TimeUnit::Microsecond, _) => Series::TimestampMicrosecond(ChunkedArray::<TimestampMicrosecondType>::$method($($args),*)),
            DataType::Timestamp(TimeUnit::Millisecond, _) => Series::TimestampMillisecond(ChunkedArray::<TimestampMillisecondType>::$method($($args),*)),
            DataType::Timestamp(TimeUnit::Second, _) => Series::TimestampSecond(ChunkedArray::<TimestampSecondType>::$method($($args),*)),
            DataType::Interval(IntervalUnit::DayTime) => Series::IntervalDayTime(ChunkedArray::<IntervalDayTimeType>::$method($($args),*)),
            DataType::Interval(IntervalUnit::YearMonth) => Series::IntervalYearMonth(ChunkedArray::<IntervalYearMonthType>::$method($($args),*)),
            // DataType::LargeList(_) => Series::LargeList(ChunkedArray::<LargeListType>::$method($($args),*)),
            _ => unimplemented!(),
        }
    }
}

#[macro_export]
macro_rules! apply_method_all_series_pair {
    ($self:ident, $dst:ident, $method:ident, $($args:expr),*) => {
        match ($self, $dst) {
            // (Series::Utf8(a), Series::Utf(b)) => a.$method(b, $($args),*),
            (Series::Bool(a), Series::Bool(b)) => a.$method(b, $($args),*),
            (Series::UInt8(a), Series::UInt8(b)) => a.$method(b, $($args),*),
            (Series::UInt16(a), Series::UInt16(b)) => a.$method(b, $($args),*),
            (Series::UInt32(a), Series::UInt32(b)) => a.$method(b, $($args),*),
            (Series::UInt64(a), Series::UInt64(b)) => a.$method(b, $($args),*),
            (Series::Int8(a), Series::Int8(b)) => a.$method(b, $($args),*),
            (Series::Int16(a), Series::Int16(b)) => a.$method(b, $($args),*),
            (Series::Int32(a), Series::Int32(b)) => a.$method(b, $($args),*),
            (Series::Int64(a), Series::Int64(b)) => a.$method(b, $($args),*),
            (Series::Float32(a), Series::Float32(b)) => a.$method(b, $($args),*),
            (Series::Float64(a), Series::Float64(b)) => a.$method(b, $($args),*),
            (Series::Date32(a), Series::Date32(b)) => a.$method(b, $($args),*),
            (Series::Date64(a), Series::Date64(b)) => a.$method(b, $($args),*),
            (Series::Time32Millisecond(a), Series::Time32Millisecond(b)) => a.$method(b, $($args),*),
            (Series::Time32Second(a), Series::Time32Second(b)) => a.$method(b, $($args),*),
            (Series::Time64Nanosecond(a), Series::Time64Nanosecond(b)) => a.$method(b, $($args),*),
            (Series::Time64Microsecond(a), Series::Time64Microsecond(b)) => a.$method(b, $($args),*),
            (Series::DurationNanosecond(a), Series::DurationNanosecond(b)) => a.$method(b, $($args),*),
            (Series::DurationMicrosecond(a), Series::DurationMicrosecond(b)) => a.$method(b, $($args),*),
            (Series::DurationMillisecond(a), Series::DurationMillisecond(b)) => a.$method(b, $($args),*),
            (Series::DurationSecond(a), Series::DurationSecond(b)) => a.$method(b, $($args),*),
            (Series::TimestampNanosecond(a), Series::TimestampNanosecond(b)) => a.$method(b, $($args),*),
            (Series::TimestampMicrosecond(a), Series::TimestampMicrosecond(b)) => a.$method(b, $($args),*),
            (Series::TimestampMillisecond(a), Series::TimestampMillisecond(b)) => a.$method(b, $($args),*),
            (Series::TimestampSecond(a), Series::TimestampSecond(b)) => a.$method(b, $($args),*),
            (Series::IntervalDayTime(a), Series::IntervalDayTime(b)) => a.$method(b, $($args),*),
            (Series::IntervalYearMonth(a), Series::IntervalYearMonth(b)) => a.$method(b, $($args),*),
            // (Series::LargeList(a), Series::LargeList(b)) => a.$method(b, $($args),*),
            _ => unimplemented!(),
        }
    }
}

macro_rules! unpack_series {
    ($self:ident, $variant:ident) => {
        if let Series::$variant(ca) = $self {
            Ok(ca)
        } else {
            Err(PolarsError::DataTypeMisMatch)
        }
    };
}

impl Series {
    /// Create series from records
    pub fn new_from_name(datatype: DataType, name: &str, line_cnt: usize) -> Self {
        apply_datatype_to_series!(datatype, new_from_name, name, line_cnt)
    }
        
    /// Name of series.
    pub fn name(&self) -> &str {
        apply_method_all_series!(self, name,)
    }

    /// Rename series.
    pub fn rename(&mut self, name: &str) {
        apply_method_all_series!(self, rename, name)
    }

    /// Get datatype of series.
    pub fn dtype(&self) -> &DataType {
        apply_method_all_series!(self, dtype,)
    }

    /// No. of chunks
    pub fn n_chunks(&self) -> usize {
        apply_method_all_series!(self, chunks_num,)
    }

    /// Take by index from an . This operation clones the data.
    pub unsafe fn take_iter_unchecked(
        &mut self,
        src_chunks: DVecRef<'_, Chunk>,
        iter: DVecRef<'_, usize>,
    ) {
        dprintln!("In take_iter_unchecked");
        apply_method_all_series!(self, take_unchecked, src_chunks, iter)
    }

    /// Get length of series.
    pub fn len(&self) -> usize {
        apply_method_all_series!(self, len,)
    }

    /// Get a single value by index. Don't use this operation for loops as a runtime cast is
    /// needed for every iteration.
    pub fn rawget(&self, index: usize) -> AnyType {
        apply_method_all_series!(self, rawget, index)
    }


    // pub unsafe fn agg_sum_numeric_unchecked(&mut self, datatype: DataType, src_series: &Vec<chunk::Chunk, &'static good_memory_allocator::SpinLockedAllocator>, indices: &Vec<usize,&'static good_memory_allocator::SpinLockedAllocator>, groups: &Vec<usize,&'static good_memory_allocator::SpinLockedAllocator>) {
    //     apply_method_all_series!(self, agg_sum, datatype, src_series, indices, groups)
    // }

    // pub unsafe fn agg_min_numeric_unchecked(&mut self, datatype: DataType, src_series: &Vec<chunk::Chunk, &'static good_memory_allocator::SpinLockedAllocator>, indices: &Vec<usize,&'static good_memory_allocator::SpinLockedAllocator>, groups: &Vec<usize,&'static good_memory_allocator::SpinLockedAllocator>) {
    //     apply_method_all_series!(self, agg_min, datatype, src_series, indices, groups)
    // }

    pub fn into_raw(self) -> (Field, DVec<Chunk>) {
        apply_method_all_series!(self, into_raw,)
    }

    pub fn from_raw(field: Field, chunks: DVec<Chunk>) -> Self {
        apply_datatype_to_series!(field.data_type(), from_raw, field, chunks)
    }
    
    // pub fn destroy(self) {
    //     apply_method_all_series!(self, destroy,)
    // }

    pub fn push_item(&mut self, item: AnyType, row_id: usize) {
        let bytes = match item {
            AnyType::Null => None,
            AnyType::Boolean(v) => Some(vec![v as u8]),
            AnyType::UInt8(v) => Some(vec![v]),
            AnyType::UInt16(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::UInt32(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::UInt64(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::Int8(v) => Some(vec![v as u8]),
            AnyType::Int16(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::Int32(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::Int64(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::Float32(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::Float64(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::Date32(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::Date64(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::Time32(v, _) => Some(v.to_le_bytes().to_vec()),
            AnyType::Time64(v, _) => Some(v.to_le_bytes().to_vec()),
            AnyType::Duration(v, _) => Some(v.to_le_bytes().to_vec()),
            AnyType::TimeStamp(v, _) => Some(v.to_le_bytes().to_vec()),
            AnyType::IntervalDayTime(v) => Some(v.to_le_bytes().to_vec()),
            AnyType::IntervalYearMonth(v) => Some(v.to_le_bytes().to_vec()),
            // AnyType::Utf8(v) => Some(v.into_bytes()),
            // AnyType::LargeList(v) => Some(v.to_le_bytes().to_vec()),
            _ => unimplemented!(),
        };

        apply_method_all_series!(self, push, bytes, row_id);
    }

    // /// Get a single value by index. Don't use this operation for loops as a runtime cast is
    // /// needed for every iteration.
    pub async fn get(&self, index: usize) -> AnyType {
        apply_async_method_all_series!(self, get, index)
    }


    /// Get remote server idx
    // pub fn server_idx(&self) -> usize {
    //     apply_method_all_series!(self, server_idx,)
    // }

    pub fn get_ref<'a>(&'a self) -> DVecRef<'a, Chunk> {
        apply_method_all_series!(self, get_ref,)
    }
    
    pub fn get_mut_ref<'a>(&'a mut self) -> DVecMutRef<'a, Chunk>  {
        apply_method_all_series!(self, get_mut_ref,)
    }
}