use std::convert::TryFrom;

use crate::{error::Error, format_code::EncodingCodes};

/// Offset of List in other implementations
/// The two implementations below do not count the byte(s) taken by `size` itself
/// 1. amqpnetlite: for List32
/// ```csharp
/// listSize = (totalBufferLength - 5); // totalBufferLength includes format code
/// ```
/// 2. qpid-protonj2: for List32
/// ```java
/// buffer.setInt(startIndex, endIndex - startIndex - 4); // startIndex is the byte after format code, endIndex is the end of the buffer
/// ```
/// offset includes 1 byte of `count`
pub const OFFSET_LIST8: usize = 1;
/// offset includes 4 byte of `count`
pub const OFFSET_LIST32: usize = 4;

/// offset includes 1 byte of `count`
pub const OFFSET_MAP8: usize = 1;
/// offset includes 4 byte of `count`
pub const OFFSET_MAP32: usize = 4;

/// offset includes 1 byte of `count` and 1 byte of element format code
pub const OFFSET_ARRAY8: usize = 2;
/// offset includes 4 bytes of `count` and 1 byte of element format code
pub const OFFSET_ARRAY32: usize = 5;

pub enum Category {
    Fixed(FixedWidth),
    Encoded(EncodedWidth),
}

#[repr(u8)]
pub enum FixedWidth {
    Zero = 0,
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
}

#[repr(u8)]
pub enum EncodedWidth {
    Zero = 0,
    One = 1,
    Four = 4,
}

// #[repr(u8)]
// pub enum VariableWidth {
//     One = 1,
//     Four = 4,
// }

// #[repr(u8)]
// pub enum CompoundWidth {
//     Zero = 0,
//     One = 1,
//     Four = 4,
// }

// #[repr(u8)]
// pub enum ArrayWidth {
//     One = 1,
//     Four = 4,
// }

impl TryFrom<EncodingCodes> for Category {
    type Error = Error;

    fn try_from(value: EncodingCodes) -> Result<Self, Self::Error> {
        let value = match value {
            EncodingCodes::DescribedType => return Err(Error::IsDescribedType),

            EncodingCodes::Null => Category::Fixed(FixedWidth::Zero),

            EncodingCodes::Boolean => Category::Fixed(FixedWidth::One),
            EncodingCodes::BooleanTrue => Category::Fixed(FixedWidth::Zero),
            EncodingCodes::BooleanFalse => Category::Fixed(FixedWidth::Zero),

            // u8
            EncodingCodes::UByte => Category::Fixed(FixedWidth::One),

            // u16
            EncodingCodes::UShort => Category::Fixed(FixedWidth::Two),

            // u32
            EncodingCodes::UInt => Category::Fixed(FixedWidth::Four),
            EncodingCodes::SmallUInt => Category::Fixed(FixedWidth::One),
            EncodingCodes::UInt0 => Category::Fixed(FixedWidth::Zero),

            // u64
            EncodingCodes::ULong => Category::Fixed(FixedWidth::Eight),
            EncodingCodes::SmallULong => Category::Fixed(FixedWidth::One),
            EncodingCodes::ULong0 => Category::Fixed(FixedWidth::Zero),

            // i8
            EncodingCodes::Byte => Category::Fixed(FixedWidth::One),

            // i16
            EncodingCodes::Short => Category::Fixed(FixedWidth::Two),

            // i32
            EncodingCodes::Int => Category::Fixed(FixedWidth::Four),
            EncodingCodes::SmallInt => Category::Fixed(FixedWidth::One),

            // i64
            EncodingCodes::Long => Category::Fixed(FixedWidth::Eight),
            EncodingCodes::SmallLong => Category::Fixed(FixedWidth::One),

            // f32
            EncodingCodes::Float => Category::Fixed(FixedWidth::Four),

            // f64
            EncodingCodes::Double => Category::Fixed(FixedWidth::Eight),

            EncodingCodes::Decimal32 => Category::Fixed(FixedWidth::Four),
            EncodingCodes::Decimal64 => Category::Fixed(FixedWidth::Eight),
            EncodingCodes::Decimal128 => Category::Fixed(FixedWidth::Sixteen),

            EncodingCodes::Char => Category::Fixed(FixedWidth::Four),

            EncodingCodes::Timestamp => Category::Fixed(FixedWidth::Eight),

            EncodingCodes::Uuid => Category::Fixed(FixedWidth::Sixteen),

            EncodingCodes::VBin8 => Category::Encoded(EncodedWidth::One),
            EncodingCodes::VBin32 => Category::Encoded(EncodedWidth::Four),

            EncodingCodes::Str8 => Category::Encoded(EncodedWidth::One),
            EncodingCodes::Str32 => Category::Encoded(EncodedWidth::Four),

            EncodingCodes::Sym8 => Category::Encoded(EncodedWidth::One),
            EncodingCodes::Sym32 => Category::Encoded(EncodedWidth::Four),

            EncodingCodes::List0 => Category::Encoded(EncodedWidth::Zero),
            EncodingCodes::List8 => Category::Encoded(EncodedWidth::One),
            EncodingCodes::List32 => Category::Encoded(EncodedWidth::Four),

            EncodingCodes::Map8 => Category::Encoded(EncodedWidth::One),
            EncodingCodes::Map32 => Category::Encoded(EncodedWidth::Four),

            EncodingCodes::Array8 => Category::Encoded(EncodedWidth::One),
            EncodingCodes::Array32 => Category::Encoded(EncodedWidth::Four),
        };

        Ok(value)
    }
}
