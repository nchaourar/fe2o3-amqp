//! Definition of `Described<T>` type

use std::marker::PhantomData;

use serde::{de, ser};

use crate::{
    __constants::{DESCRIBED_BASIC, DESCRIPTOR},
    descriptor::Descriptor,
};

/// Contains a Box to descriptor and a Box to value T.
///
/// This should usually be avoided other than in Value type.
/// Two pointers are used to reduce the memory size of the Value type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Described<T> {
    /// Descriptor of descriptor
    pub descriptor: Descriptor,

    /// Value of described
    pub value: Box<T>,
}

impl<T: ser::Serialize> ser::Serialize for Described<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use ser::SerializeStruct;
        let mut state = serializer.serialize_struct(DESCRIBED_BASIC, 2)?;
        state.serialize_field(DESCRIPTOR, &self.descriptor)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

struct Visitor<'de, T> {
    marker: PhantomData<T>,
    lifetime: PhantomData<&'de ()>,
}

impl<'de, T: de::Deserialize<'de>> de::Visitor<'de> for Visitor<'de, T> {
    type Value = Described<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Described")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let descriptor: Descriptor = match seq.next_element()? {
            Some(val) => val,
            None => return Err(de::Error::custom("Expecting descriptor")),
        };

        let value: T = match seq.next_element()? {
            Some(val) => val,
            None => {
                return Err(de::Error::custom(
                    "Insufficient number of elements. Expecting value",
                ))
            }
        };

        Ok(Described {
            descriptor: descriptor,
            value: Box::new(value),
        })
    }
}

impl<'de, T: de::Deserialize<'de>> de::Deserialize<'de> for Described<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &[DESCRIPTOR, "value"];
        deserializer.deserialize_struct(
            DESCRIBED_BASIC,
            FIELDS,
            Visitor {
                marker: PhantomData,
                lifetime: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_amqp_derive::{SerializeComposite, DeserializeComposite};

    use crate::{descriptor::Descriptor, from_slice, to_vec};

    use super::Described;

    #[test]
    fn test_serialize_described_value() {
        let descriptor = Descriptor::Code(0x11);
        let value = Box::new(vec![1i32, 2]);
        let described = Described { descriptor, value };
        let buf = to_vec(&described).unwrap();
        println!("{:x?}", buf);
    }

    #[test]
    fn test_deserialzie_described_value() {
        let descriptor = Descriptor::Code(0x11);
        let value = Box::new(vec![1i32, 2]);
        let described = Described { descriptor, value };
        let buf = to_vec(&described).unwrap();
        println!("{:?}", &buf);
        let recovered: Described<Vec<i32>> = from_slice(&buf).unwrap();
        println!("{:?}", recovered);
    }

    // Expanded macro
    use crate as serde_amqp;

    #[derive(Debug, PartialEq, SerializeComposite, DeserializeComposite)]
    struct Foo {
        b: u64,
        is_fool: Option<bool>,
        a: i32,
    }

    #[test]
    fn test_expanded_list_macro() {
        let foo = Foo {
            b: 0,
            is_fool: None,
            a: 0,
        };
        let serialized = to_vec(&foo).unwrap();
        println!("{:?}", &serialized);
        let deserialized: Foo = from_slice(&serialized).unwrap();
        println!("{:?}", &deserialized);
        assert_eq!(foo, deserialized);
    }

    #[derive(Debug, PartialEq, SerializeComposite, DeserializeComposite)]
    struct Test {
        a: Option<i32>,
        b: bool,
    }

    #[test]
    fn test_expanded_map_macro() {
        let test = Test {
            a: Some(1),
            b: true,
        };
        let serialized = to_vec(&test).unwrap();
        println!("{:?}", &serialized);
        let deserialized: Test = from_slice(&serialized).unwrap();
        println!("{:?}", &deserialized);
        assert_eq!(test, deserialized);
    }
}
