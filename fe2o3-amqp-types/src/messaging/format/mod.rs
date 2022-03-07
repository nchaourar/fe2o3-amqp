use serde::{Deserialize, Serialize};
use serde_amqp::{
    macros::{DeserializeComposite, SerializeComposite},
    primitives::{Binary, Boolean, Symbol, UByte, UInt, ULong, Uuid},
    value::Value,
};
use std::{collections::BTreeMap, fmt::Display};

use crate::{definitions::Milliseconds, primitives::SimpleValue};

/// 3.2.1 Header
/// Transport headers for a message.
/// <type name="header" class="composite" source="list" provides="section">
///     <descriptor name="amqp:header:list" code="0x00000000:0x00000070"/>
/// </type>
#[derive(Debug, Clone, Default, DeserializeComposite, SerializeComposite)]
#[amqp_contract(
    name = "amqp:header:list",
    code = 0x0000_0000_0000_0070,
    encoding = "list",
    rename_all = "kebab-case"
)]
pub struct Header {
    /// <field name="durable" type="boolean" default="false"/>
    #[amqp_contract(default)]
    pub durable: Boolean,

    /// <field name="priority" type="ubyte" default="4"/>
    #[amqp_contract(default)]
    pub priority: Priority,

    /// <field name="ttl" type="milliseconds"/>
    pub ttl: Option<Milliseconds>,

    /// <field name="first-acquirer" type="boolean" default="false"/>
    #[amqp_contract(default)]
    pub first_acquirer: Boolean,

    /// <field name="delivery-count" type="uint" default="0"/>
    #[amqp_contract(default)]
    pub delivery_count: UInt,
}

/// relative message priority
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Priority(pub UByte);

impl Default for Priority {
    fn default() -> Self {
        Self(4)
    }
}

impl From<UByte> for Priority {
    fn from(value: UByte) -> Self {
        Self(value)
    }
}

impl From<Priority> for UByte {
    fn from(value: Priority) -> Self {
        value.0
    }
}

/// 3.2.2 Delivery Annotations
/// <type name="delivery-annotations" class="restricted" source="annotations" provides="section">
///     <descriptor name="amqp:delivery-annotations:map" code="0x00000000:0x00000071"/>
/// </type>
#[derive(Debug, Clone, DeserializeComposite, SerializeComposite)]
#[amqp_contract(
    name = "amqp:delivery-annotations:map",
    code = 0x0000_0000_0000_0071,
    encoding = "basic", // A simple wrapper over a map
)]
pub struct DeliveryAnnotations(pub Annotations);

/// 3.2.3 Message Annotations
/// <type name="message-annotations" class="restricted" source="annotations" provides="section">
///     <descriptor name="amqp:message-annotations:map" code="0x00000000:0x00000072"/>
/// </type>
#[derive(Debug, Clone, SerializeComposite, DeserializeComposite)]
#[amqp_contract(
    name = "amqp:message-annotations:map",
    code = 0x0000_0000_0000_0072,
    encoding = "basic"
)]
pub struct MessageAnnotations(pub Annotations);

pub mod properties;
pub use properties::Properties;

/// 3.2.5 Application Properties
/// <type name="application-properties" class="restricted" source="map" provides="section">
///     <descriptor name="amqp:application-properties:map" code="0x00000000:0x00000074"/>
/// </type>
#[derive(Debug, Clone, SerializeComposite, DeserializeComposite)]
#[amqp_contract(
    name = "amqp:application-properties:map",
    code = 0x0000_0000_0000_0074,
    encoding = "basic"
)]
pub struct ApplicationProperties(pub BTreeMap<String, SimpleValue>);

/// 3.2.6 Data
/// <type name="data" class="restricted" source="binary" provides="section">
///     <descriptor name="amqp:data:binary" code="0x00000000:0x00000075"/>
/// </type>
#[derive(Debug, Clone, SerializeComposite, DeserializeComposite)]
#[amqp_contract(
    name = "amqp:data:binary",
    code = 0x0000_0000_0000_0075,
    encoding = "basic"
)]
pub struct Data(pub Binary);

impl TryFrom<Value> for Data {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Binary(buf) = value {
            Ok(Data(buf))
        } else {
            Err(value)
        }
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Data of length: {}", self.0.len())
    }
}

/// 3.2.7 AMQP Sequence
/// <type name="amqp-sequence" class="restricted" source="list" provides="section">
///     <descriptor name="amqp:amqp-sequence:list" code="0x00000000:0x00000076"/>
/// </type>
#[derive(Debug, Clone, SerializeComposite, DeserializeComposite)]
#[amqp_contract(
    name = "amqp:amqp-sequence:list",
    code = 0x0000_0000_0000_0076,
    encoding = "basic"
)]
pub struct AmqpSequence<T>(pub Vec<T>); // Vec doesnt implement Display trait

// impl TryFrom<Value> for AmqpSequence {
//     type Error = Value;

//     fn try_from(value: Value) -> Result<Self, Self::Error> {
//         if let Value::List(vals) = value {
//             Ok(AmqpSequence(vals))
//         } else {
//             Err(value)
//         }
//     }
// }

/// 3.2.8 AMQP Value
/// <type name="amqp-value" class="restricted" source="*" provides="section">
///     <descriptor name="amqp:amqp-value:*" code="0x00000000:0x00000077"/>
/// </type>
#[derive(Debug, Clone, SerializeComposite, DeserializeComposite)]
#[amqp_contract(
    name = "amqp:amqp-value:*",
    code = 0x0000_0000_0000_0077,
    encoding = "basic"
)]
pub struct AmqpValue<T>(pub T);

// impl<T: Into<Value>> From<T> for AmqpValue {
//     fn from(val: T) -> Self {
//         Self(val.into())
//     }
// }

/// 3.2.9 Footer
/// Transport footers for a message.
/// <type name="footer" class="restricted" source="annotations" provides="section">
///     <descriptor name="amqp:footer:map" code="0x00000000:0x00000078"/>
/// </type>
#[derive(Debug, Clone, SerializeComposite, DeserializeComposite)]
#[amqp_contract(
    name = "amqp:footer:map",
    code = 0x0000_0000_0000_0078,
    encoding = "basic"
)]
pub struct Footer(pub Annotations);

/// 3.2.10 Annotations
/// <type name="annotations" class="restricted" source="map"/>
pub type Annotations = BTreeMap<Symbol, Value>;

/// The annotations type is a map where the keys are restricted to be of type symbol or of type ulong. All ulong
/// keys, and all symbolic keys except those beginning with “x-” are reserved. Keys beginning with “x-opt-” MUST be
/// ignored if not understood. On receiving an annotation key which is not understood, and which does not begin with
/// “x-opt”, the receiving AMQP container MUST detach the link with a not-implemented error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageId {
    /// 3.2.11 Message ID ULong
    /// <type name="message-id-ulong" class="restricted" source="ulong" provides="message-id"/>
    ULong(ULong),

    /// 3.2.12 Message ID UUID
    /// <type name="message-id-uuid" class="restricted" source="uuid" provides="message-id"/>
    Uuid(Uuid),

    /// 3.2.13 Message ID Binary
    /// <type name="message-id-binary" class="restricted" source="binary" provides="message-id"/>
    Binary(Binary),

    /// 3.2.14 Message ID String
    /// <type name="message-id-string" class="restricted" source="string" provides="message-id"/>
    String(String),
}

/// 3.2.15 Address String
/// Address of a node.
/// <type name="address-string" class="restricted" source="string" provides="address"/>
pub type Address = String;

/// 3.2.16 CONSTANTS
pub const MESSAGE_FORMAT: u32 = 0; // FIXME: type of message format?

#[cfg(test)]
mod tests {
    use serde_amqp::{to_vec, primitives::Binary};

    use super::{Header, Priority};

    #[test]
    fn test_serialize_deserialize_header() {
        let header = Header {
            durable: true,
            priority: Priority(0),
            ttl: None,
            first_acquirer: false,
            delivery_count: 0,
        };
        let serialized = to_vec(&header).unwrap();
        println!("{:x?}", &serialized);
    }

    #[test]
    fn test_display_data() {
        let b = Binary::from(vec![1u8, 2]);
        println!("{}", b.len());
    }
}
