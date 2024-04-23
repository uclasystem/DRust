use serde::{Deserialize, Serialize};

use crate::exclude;

/// An absolute length of time in seconds, milliseconds, microseconds or nanoseconds.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TimeUnit {
    /// Time in seconds.
    Second,
    /// Time in milliseconds.
    Millisecond,
    /// Time in microseconds.
    Microsecond,
    /// Time in nanoseconds.
    Nanosecond,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DateUnit {
    /// Days since the UNIX epoch.
    Day,
    /// Milliseconds indicating UNIX time elapsed since the epoch (no
    /// leap seconds), where the values are evenly divisible by 86400000.
    Millisecond,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IntervalUnit {
    /// Indicates the number of elapsed whole months, stored as 4-byte integers.
    YearMonth,
    /// Indicates the number of elapsed days and milliseconds,
    /// stored as 2 contiguous 32-bit integers (8-bytes in total).
    DayTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DataType {
    /// Null type
    Null,
    /// A boolean datatype representing the values `true` and `false`.
    Boolean,
    /// A signed 8-bit integer.
    Int8,
    /// A signed 16-bit integer.
    Int16,
    /// A signed 32-bit integer.
    Int32,
    /// A signed 64-bit integer.
    Int64,
    /// An unsigned 8-bit integer.
    UInt8,
    /// An unsigned 16-bit integer.
    UInt16,
    /// An unsigned 32-bit integer.
    UInt32,
    /// An unsigned 64-bit integer.
    UInt64,
    /// A 16-bit floating point number.
    Float16,
    /// A 32-bit floating point number.
    Float32,
    /// A 64-bit floating point number.
    Float64,
    /// A timestamp with an optional timezone.
    ///
    /// Time is measured as a Unix epoch, counting the seconds from
    /// 00:00:00.000 on 1 January 1970, excluding leap seconds,
    /// as a 64-bit integer.
    ///
    /// The time zone is a string indicating the name of a time zone, one of:
    ///
    /// * As used in the Olson time zone database (the "tz database" or
    ///   "tzdata"), such as "America/New_York"
    /// * An absolute time zone offset of the form +XX:XX or -XX:XX, such as +07:30
    Timestamp(TimeUnit, Option<String>),
    /// A 32-bit date representing the elapsed time since UNIX epoch (1970-01-01)
    /// in days (32 bits).
    Date32(DateUnit),
    /// A 64-bit date representing the elapsed time since UNIX epoch (1970-01-01)
    /// in milliseconds (64 bits).
    Date64(DateUnit),
    /// A 32-bit time representing the elapsed time since midnight in the unit of `TimeUnit`.
    Time32(TimeUnit),
    /// A 64-bit time representing the elapsed time since midnight in the unit of `TimeUnit`.
    Time64(TimeUnit),
    /// Measure of elapsed time in either seconds, milliseconds, microseconds or nanoseconds.
    Duration(TimeUnit),
    /// A "calendar" interval which models types that don't necessarily
    /// have a precise duration without the context of a base timestamp (e.g.
    /// days can differ in length during day light savings time transitions).
    Interval(IntervalUnit),
    /// Opaque binary data of variable length.
    Binary,
    /// Opaque binary data of fixed size.
    /// Enum parameter specifies the number of bytes per value.
    FixedSizeBinary(i32),
    /// Opaque binary data of variable length and 64-bit offsets.
    LargeBinary,
    /// A variable-length string in Unicode with UTF-8 encoding.
    Utf8,
    /// A variable-length string in Unicode with UFT-8 encoding and 64-bit offsets.
    LargeUtf8,
    /// A list of some logical data type with variable length.
    List(Box<DataType>),
    /// A list of some logical data type with fixed length.
    FixedSizeList(Box<DataType>, i32),
    /// A list of some logical data type with variable length and 64-bit offsets.
    LargeList(Box<DataType>),
    /// A dictionary array where each element is a single value indexed by an integer key.
    /// This is mostly used to represent strings or a limited set of primitive types as integers.
    Dictionary(Box<DataType>, Box<DataType>),
}


pub trait PrimitiveType: 'static {
    /// Corresponding Rust native type for the primitive type.
    type Native: Sync;

    /// Returns the corresponding Arrow data type of this primitive type.
    fn get_data_type() -> DataType;

    /// Returns the bit width of this primitive type.
    fn get_bit_width() -> usize;

    /// Returns a default value of this primitive type.
    ///
    /// This is useful for aggregate array ops like `sum()`, `mean()`.
    fn default_value() -> Self::Native;

    // fn value(&self) -> Self::Native;
}

macro_rules! make_type {
    ($name:ident, $native_ty:ty, $data_ty:expr, $bit_width:expr, $default_val:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {}

        impl PrimitiveType for $name {
            type Native = $native_ty;

            fn get_data_type() -> DataType {
                $data_ty
            }

            fn get_bit_width() -> usize {
                $bit_width
            }

            fn default_value() -> Self::Native {
                $default_val
            }

            // fn value(&self) -> Self::Native {
            //     self.value
            // }
        }
    };
}

make_type!(BooleanType, bool, DataType::Boolean, 8, false);
make_type!(Int8Type, i8, DataType::Int8, 8, 0i8);
make_type!(Int16Type, i16, DataType::Int16, 16, 0i16);
make_type!(Int32Type, i32, DataType::Int32, 32, 0i32);
make_type!(Int64Type, i64, DataType::Int64, 64, 0i64);
make_type!(UInt8Type, u8, DataType::UInt8, 8, 0u8);
make_type!(UInt16Type, u16, DataType::UInt16, 16, 0u16);
make_type!(UInt32Type, u32, DataType::UInt32, 32, 0u32);
make_type!(UInt64Type, u64, DataType::UInt64, 64, 0u64);
make_type!(Float32Type, f32, DataType::Float32, 32, 0.0f32);
make_type!(Float64Type, f64, DataType::Float64, 64, 0.0f64);
make_type!(
    TimestampSecondType,
    i64,
    DataType::Timestamp(TimeUnit::Second, None),
    64,
    0i64
);
make_type!(
    TimestampMillisecondType,
    i64,
    DataType::Timestamp(TimeUnit::Millisecond, None),
    64,
    0i64
);
make_type!(
    TimestampMicrosecondType,
    i64,
    DataType::Timestamp(TimeUnit::Microsecond, None),
    64,
    0i64
);
make_type!(
    TimestampNanosecondType,
    i64,
    DataType::Timestamp(TimeUnit::Nanosecond, None),
    64,
    0i64
);
make_type!(Date32Type, i32, DataType::Date32(DateUnit::Day), 32, 0i32);
make_type!(
    Date64Type,
    i64,
    DataType::Date64(DateUnit::Millisecond),
    64,
    0i64
);
make_type!(
    Time32SecondType,
    i32,
    DataType::Time32(TimeUnit::Second),
    32,
    0i32
);
make_type!(
    Time32MillisecondType,
    i32,
    DataType::Time32(TimeUnit::Millisecond),
    32,
    0i32
);
make_type!(
    Time64MicrosecondType,
    i64,
    DataType::Time64(TimeUnit::Microsecond),
    64,
    0i64
);
make_type!(
    Time64NanosecondType,
    i64,
    DataType::Time64(TimeUnit::Nanosecond),
    64,
    0i64
);
make_type!(
    IntervalYearMonthType,
    i32,
    DataType::Interval(IntervalUnit::YearMonth),
    32,
    0i32
);
make_type!(
    IntervalDayTimeType,
    i64,
    DataType::Interval(IntervalUnit::DayTime),
    64,
    0i64
);
make_type!(
    DurationSecondType,
    i64,
    DataType::Duration(TimeUnit::Second),
    64,
    0i64
);
make_type!(
    DurationMillisecondType,
    i64,
    DataType::Duration(TimeUnit::Millisecond),
    64,
    0i64
);
make_type!(
    DurationMicrosecondType,
    i64,
    DataType::Duration(TimeUnit::Microsecond),
    64,
    0i64
);
make_type!(
    DurationNanosecondType,
    i64,
    DataType::Duration(TimeUnit::Nanosecond),
    64,
    0i64
);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Field {
    name: String,
    data_type: DataType,
    nullable: bool,
    dict_id: i64,
    dict_is_ordered: bool,
}

impl Field {
    /// Creates a new field
    pub fn new(name: &str, data_type: DataType, nullable: bool) -> Self {
        Field {
            name: name.to_string(),
            data_type,
            nullable,
            dict_id: 0,
            dict_is_ordered: false,
        }
    }

    /// Creates a new field
    pub fn new_dict(
        name: &str,
        data_type: DataType,
        nullable: bool,
        dict_id: i64,
        dict_is_ordered: bool,
    ) -> Self {
        Field {
            name: name.to_string(),
            data_type,
            nullable,
            dict_id,
            dict_is_ordered,
        }
    }

    /// Returns an immutable reference to the `Field`'s name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns an immutable reference to the `Field`'s name
    pub fn rename(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Returns an immutable reference to the `Field`'s  data-type
    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    /// Indicates whether this `Field` supports null values
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    // /// Merge field into self if it is compatible. Struct will be merged recursively.
    // ///
    // /// Example:
    // ///
    // /// ```
    // /// use arrow::datatypes::*;
    // ///
    // /// let mut field = Field::new("c1", DataType::Int64, false);
    // /// assert!(field.try_merge(&Field::new("c1", DataType::Int64, true)).is_ok());
    // /// assert!(field.is_nullable());
    // /// ```
    // pub fn try_merge(&mut self, from: &Field) -> Result<(), PolarsError> {
    //     if from.dict_id != self.dict_id {
    //         return Err(PolarsError::SelfArrowError);
    //     }
    //     if from.dict_is_ordered != self.dict_is_ordered {
    //         return Err(PolarsError::SelfArrowError);
    //     }
    //     match &mut self.data_type {
    //         DataType::Struct(nested_fields) => match &from.data_type {
    //             DataType::Struct(from_nested_fields) => {
    //                 for from_field in from_nested_fields {
    //                     let mut is_new_field = true;
    //                     for self_field in nested_fields.iter_mut() {
    //                         if self_field.name != from_field.name {
    //                             continue;
    //                         }
    //                         is_new_field = false;
    //                         self_field.try_merge(&from_field)?;
    //                     }
    //                     if is_new_field {
    //                         nested_fields.push(from_field.clone());
    //                     }
    //                 }
    //             }
    //             _ => {
    //                 return Err(PolarsError::SelfArrowError);
    //             }
    //         },
    //         DataType::Union(nested_fields) => match &from.data_type {
    //             DataType::Union(from_nested_fields) => {
    //                 for from_field in from_nested_fields {
    //                     let mut is_new_field = true;
    //                     for self_field in nested_fields.iter_mut() {
    //                         if from_field == self_field {
    //                             is_new_field = false;
    //                             break;
    //                         }
    //                     }
    //                     if is_new_field {
    //                         nested_fields.push(from_field.clone());
    //                     }
    //                 }
    //             }
    //             _ => {
    //                 return Err(PolarsError::SelfArrowError);
    //             }
    //         },
    //         DataType::Null
    //         | DataType::Boolean
    //         | DataType::Int8
    //         | DataType::Int16
    //         | DataType::Int32
    //         | DataType::Int64
    //         | DataType::UInt8
    //         | DataType::UInt16
    //         | DataType::UInt32
    //         | DataType::UInt64
    //         | DataType::Float16
    //         | DataType::Float32
    //         | DataType::Float64
    //         | DataType::Timestamp(_, _)
    //         | DataType::Date32(_)
    //         | DataType::Date64(_)
    //         | DataType::Time32(_)
    //         | DataType::Time64(_)
    //         | DataType::Duration(_)
    //         | DataType::Binary
    //         | DataType::LargeBinary
    //         | DataType::Interval(_)
    //         | DataType::LargeList(_)
    //         | DataType::List(_)
    //         | DataType::Dictionary(_, _)
    //         | DataType::FixedSizeList(_, _)
    //         | DataType::FixedSizeBinary(_)
    //         | DataType::Utf8
    //         | DataType::LargeUtf8 => {
    //             if self.data_type != from.data_type {
    //                 return Err(PolarsError::SelfArrowError);
    //             }
    //         }
    //     }
    //     if from.nullable {
    //         self.nullable = from.nullable;
    //     }

    //     Ok(())
    // }
}
