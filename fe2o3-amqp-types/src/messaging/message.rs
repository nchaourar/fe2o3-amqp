use serde::{
    de::{self, VariantAccess},
    ser::SerializeStruct,
    Serialize,
};
use serde_amqp::{
    __constants::{DESCRIBED_BASIC, DESCRIPTOR},
    format_code::EncodingCodes,
};

use super::{
    AmqpSequence, AmqpValue, ApplicationProperties, Data, DeliveryAnnotations, Footer, Header,
    MessageAnnotations, Properties,
};

#[derive(Debug, Clone)]
pub struct Message {
    pub header: Option<Header>,
    pub delivery_annotations: Option<DeliveryAnnotations>,
    pub message_annotations: Option<MessageAnnotations>,
    pub properties: Option<Properties>,
    pub application_properties: Option<ApplicationProperties>,
    pub body_section: BodySection,
    pub footer: Option<Footer>,
}

impl Message {
    /// Count number of sections
    pub fn sections(&self) -> u32 {
        // The body section must be present
        let mut count = 1;

        if self.header.is_some() {
            count += 1;
        }
        if self.delivery_annotations.is_some() {
            count += 1;
        }
        if self.message_annotations.is_some() {
            count += 1;
        }
        if self.properties.is_some() {
            count += 1;
        }
        if self.application_properties.is_some() {
            count += 1;
        }
        if self.footer.is_some() {
            count += 1;
        }

        count
    }

    /// A complete message must have at least the body section, so we
    /// only need to whether footer is available
    pub fn last_section_code(&self) -> u8 {
        if self.footer.is_some() {
            0x78
        } else {
            0x77
        }
    }

    // // This should only need to check for Footer or BodySection
    // pub fn last_section_offset(descriptor_code: u8, bytes: &[u8]) -> Option<u64> {
    //     const DESCRIBED_TYPE: u8 = EncodingCodes::DescribedType as u8;
    //     const SMALL_ULONG_TYPE: u8 = EncodingCodes::SmallUlong as u8;
    //     const ULONG_TYPE: u8 = EncodingCodes::ULong as u8;

    //     let len = bytes.len();
    //     let mut iter = bytes.iter().zip(
    //         bytes.iter().skip(1).zip(
    //             bytes.iter().skip(2)
    //         )
    //     );

    //     iter.rposition(|(&b0, (&b1, &b2))| {
    //         match (b0, b1, b2) {
    //             (DESCRIBED_TYPE, SMALL_ULONG_TYPE, code)
    //             | (DESCRIBED_TYPE, ULONG_TYPE, code) => {
    //                 code == descriptor_code
    //             },
    //             _ => false
    //         }
    //     }).map(|val| (len - val) as u64)
    // }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(DESCRIBED_BASIC, 7)?;
        if let Some(header) = &self.header {
            state.serialize_field("header", header)?;
        }
        if let Some(delivery_annotations) = &self.delivery_annotations {
            state.serialize_field("delivery_annotations", delivery_annotations)?;
        }
        if let Some(message_annotations) = &self.message_annotations {
            state.serialize_field("message_annotations", message_annotations)?;
        }
        if let Some(properties) = &self.properties {
            state.serialize_field("properties", properties)?;
        }
        if let Some(application_properties) = &self.application_properties {
            state.serialize_field("application_properties", application_properties)?
        }
        state.serialize_field("body_section", &self.body_section)?;
        if let Some(footer) = &self.footer {
            state.serialize_field("footer", footer)?;
        }
        state.end()
    }
}

enum Field {
    Header,
    DeliveryAnnotations,
    MessageAnnotations,
    Properties,
    ApplicationProperties,
    BodySection,
    Footer,
}

struct FieldVisitor {}

impl<'de> de::Visitor<'de> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Field")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let val = match v {
            "amqp:header:list" => Field::Header,
            "amqp:delivery-annotations:map" => Field::DeliveryAnnotations,
            "amqp:message-annotations:map" => Field::MessageAnnotations,
            "amqp:properties:list" => Field::Properties,
            "amqp:application-properties:map" => Field::ApplicationProperties,
            "amqp:data:binary" | "amqp:amqp-sequence:list" | "amqp:amqp-value:*" => {
                Field::BodySection
            }
            "amqp:footer:map" => Field::Footer,
            _ => return Err(serde_amqp::serde::de::Error::custom("Unknown identifier")),
        };
        Ok(val)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let val = match v {
            0x0000_0000_0000_0070 => Field::Header,
            0x0000_0000_0000_0071 => Field::DeliveryAnnotations,
            0x0000_0000_0000_0072 => Field::MessageAnnotations,
            0x0000_0000_0000_0073 => Field::Properties,
            0x0000_0000_0000_0074 => Field::ApplicationProperties,
            0x0000_0000_0000_0075 | 0x0000_0000_0000_0076 | 0x0000_0000_0000_0077 => {
                Field::BodySection
            }
            0x0000_0000_0000_0078 => Field::Footer,
            _ => return Err(serde_amqp::serde::de::Error::custom("Unknown identifier")),
        };
        Ok(val)
    }
}

impl<'de> de::Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(FieldVisitor {})
    }
}

struct Visitor {}

impl<'de> de::Visitor<'de> for Visitor {
    type Value = Message;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Message")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut header = None;
        let mut delivery_annotations = None;
        let mut message_annotations = None;
        let mut properties = None;
        let mut application_properties = None;
        let mut body_section = None;
        let mut footer = None;

        for _ in 0..7 {
            let opt = match seq.next_element() {
                Ok(o) => o,
                // FIXME: all errors here are just treated as end of stream
                Err(_) => break,
            };
            let field: Field = match opt {
                Some(val) => val,
                None => break,
            };

            match field {
                Field::Header => header = seq.next_element()?,
                Field::DeliveryAnnotations => delivery_annotations = seq.next_element()?,
                Field::MessageAnnotations => message_annotations = seq.next_element()?,
                Field::Properties => properties = seq.next_element()?,
                Field::ApplicationProperties => application_properties = seq.next_element()?,
                Field::BodySection => body_section = seq.next_element()?,
                Field::Footer => footer = seq.next_element()?,
            }
        }

        Ok(Message {
            header,
            delivery_annotations,
            message_annotations,
            properties,
            application_properties,
            body_section: body_section.ok_or_else(|| de::Error::custom("Expecting BodySection"))?,
            footer,
        })
    }
}

impl<'de> de::Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            DESCRIBED_BASIC,
            &[
                DESCRIPTOR,
                "header",
                DESCRIPTOR,
                "delivery_annotations",
                DESCRIPTOR,
                "message_annotations",
                DESCRIPTOR,
                "properties",
                DESCRIPTOR,
                "application_properties",
                DESCRIPTOR,
                "body_section",
                DESCRIPTOR,
                "footer",
            ],
            Visitor {},
        )
    }
}

impl<T> From<T> for Message
where
    T: Into<BodySection>,
{
    fn from(value: T) -> Self {
        Message {
            header: None,
            delivery_annotations: None,
            message_annotations: None,
            properties: None,
            application_properties: None,
            body_section: value.into(),
            footer: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct EmptyBody {}

#[derive(Debug, Default)]
pub struct Builder<T> {
    pub header: Option<Header>,
    pub delivery_annotations: Option<DeliveryAnnotations>,
    pub message_annotations: Option<MessageAnnotations>,
    pub properties: Option<Properties>,
    pub application_properties: Option<ApplicationProperties>,
    pub body_section: T,
    pub footer: Option<Footer>,
}

impl Builder<EmptyBody> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T> Builder<T> {
    pub fn body_section(self, body_section: impl Into<BodySection>) -> Builder<BodySection> {
        Builder::<BodySection> {
            header: self.header,
            delivery_annotations: self.delivery_annotations,
            message_annotations: self.message_annotations,
            properties: self.properties,
            application_properties: self.application_properties,
            body_section: body_section.into(),
            footer: self.footer,
        }
    }

    pub fn header(mut self, header: impl Into<Option<Header>>) -> Self {
        self.header = header.into();
        self
    }

    pub fn delivery_annotations(
        mut self,
        delivery_annotations: impl Into<Option<DeliveryAnnotations>>,
    ) -> Self {
        self.delivery_annotations = delivery_annotations.into();
        self
    }

    pub fn message_annotations(
        mut self,
        message_annotations: impl Into<Option<MessageAnnotations>>,
    ) -> Self {
        self.message_annotations = message_annotations.into();
        self
    }

    pub fn properties(mut self, properties: impl Into<Option<Properties>>) -> Self {
        self.properties = properties.into();
        self
    }

    pub fn application_properties(
        mut self,
        appplication_properties: impl Into<Option<ApplicationProperties>>,
    ) -> Self {
        self.application_properties = appplication_properties.into();
        self
    }

    pub fn footer(mut self, footer: impl Into<Option<Footer>>) -> Self {
        self.footer = footer.into();
        self
    }
}

impl Builder<BodySection> {
    pub fn build(self) -> Message {
        Message {
            header: self.header,
            delivery_annotations: self.delivery_annotations,
            message_annotations: self.message_annotations,
            properties: self.properties,
            application_properties: self.application_properties,
            body_section: self.body_section,
            footer: self.footer,
        }
    }
}

/// Only one section of Data and one section of AmqpSequence
/// is supported for now
#[derive(Debug, Clone)]
pub enum BodySection {
    Data(Data),
    Sequence(AmqpSequence),
    Value(AmqpValue),
}

impl<T: Into<AmqpValue>> From<T> for BodySection {
    fn from(value: T) -> Self {
        BodySection::Value(value.into())
    }
}

impl From<AmqpSequence> for BodySection {
    fn from(val: AmqpSequence) -> Self {
        Self::Sequence(val)
    }
}

impl From<Data> for BodySection {
    fn from(val: Data) -> Self {
        Self::Data(val)
    }
}

mod body_section {
    use super::*;

    impl Serialize for BodySection {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                BodySection::Data(data) => data.serialize(serializer),
                BodySection::Sequence(seq) => seq.serialize(serializer),
                BodySection::Value(val) => val.serialize(serializer),
            }
        }
    }

    struct FieldVisitor {}

    #[derive(Debug)]
    enum Field {
        Data,
        Sequence,
        Value,
    }

    impl<'de> de::Visitor<'de> for FieldVisitor {
        type Value = Field;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter
                .write_str("BodySection variant. One of Vec<Data>, Vec<AmqpSequence>, AmqpValue")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "amqp:data:binary" => Ok(Field::Data),
                "amqp:amqp-sequence:list" => Ok(Field::Sequence),
                "amqp:amqp-value:*" => Ok(Field::Value),
                _ => return Err(de::Error::custom("Invalid descriptor code")),
            }
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                0x0000_0000_0000_0075 => Ok(Field::Data),
                0x0000_0000_0000_0076 => Ok(Field::Sequence),
                0x0000_0000_0000_0077 => Ok(Field::Value),
                _ => return Err(de::Error::custom("Invalid descriptor code")),
            }
        }
    }

    impl<'de> de::Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_ignored_any(FieldVisitor {})
        }
    }

    struct Visitor {}

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = BodySection;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("enum BodySection")
        }

        fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
        where
            A: de::EnumAccess<'de>,
        {
            let (val, variant) = data.variant()?;

            match val {
                Field::Data => {
                    let data = variant.newtype_variant()?;
                    Ok(BodySection::Data(data))
                }
                Field::Sequence => {
                    let sequence = variant.newtype_variant()?;
                    Ok(BodySection::Sequence(sequence))
                }
                Field::Value => {
                    let value = variant.newtype_variant()?;
                    Ok(BodySection::Value(value))
                }
            }
        }
    }

    impl<'de> de::Deserialize<'de> for BodySection {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_enum(
                serde_amqp::__constants::UNTAGGED_ENUM,
                &["Data", "Sequence", "Value"],
                Visitor {},
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, vec};

    use serde_amqp::{from_slice, to_vec, value::Value};
    use serde_bytes::ByteBuf;

    use crate::messaging::{
        message::BodySection, AmqpSequence, AmqpValue, Data, DeliveryAnnotations, Header,
        MessageAnnotations,
    };

    use super::Message;

    #[test]
    fn test_serialize_deserialize_body() {
        let data = b"amqp".to_vec();
        let data = Data(ByteBuf::from(data));
        let body = BodySection::Data(data);
        let serialized = to_vec(&body).unwrap();
        println!("{:x?}", serialized);
        let field: BodySection = from_slice(&serialized).unwrap();
        println!("{:?}", field);

        let body = BodySection::Sequence(AmqpSequence(vec![Value::Bool(true)]));
        let serialized = to_vec(&body).unwrap();
        println!("{:x?}", serialized);
        let field: BodySection = from_slice(&serialized).unwrap();
        println!("{:?}", field);

        let body = BodySection::Value(AmqpValue(Value::Bool(true)));
        let serialized = to_vec(&body).unwrap();
        println!("{:x?}", serialized);
        let field: BodySection = from_slice(&serialized).unwrap();
        println!("{:?}", field);
    }

    #[test]
    fn test_serialize_message() {
        let message = Message {
            header: Some(Header {
                durable: true,
                ..Default::default()
            }),
            // header: None,
            delivery_annotations: None,
            message_annotations: None,
            properties: None,
            application_properties: None,
            body_section: BodySection::Value(AmqpValue(Value::Bool(true))),
            footer: None,
        };
        let serialized = to_vec(&message).unwrap();
        println!("{:x?}", serialized);
    }

    #[test]
    fn test_serialize_deserialize_message() {
        let message = Message {
            header: Some(Header {
                durable: true,
                ..Default::default()
            }),
            // header: None,
            delivery_annotations: Some(DeliveryAnnotations(BTreeMap::new())),
            // delivery_annotations: None,
            message_annotations: Some(MessageAnnotations(BTreeMap::new())),
            // message_annotations: None,
            properties: None,
            application_properties: None,
            body_section: BodySection::Value(AmqpValue(Value::Bool(true))),
            footer: None,
        };
        let serialized = to_vec(&message).unwrap();
        println!("{:x?}", serialized);
        let deserialized: Message = from_slice(&serialized).unwrap();
        println!("{:?}", deserialized);
    }
}
