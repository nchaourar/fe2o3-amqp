pub(crate) enum NewType {
    None,
    Array,
    Dec32,
    Dec64,
    Dec128,
    Symbol,
    Timestamp,
    Uuid,
}

impl Default for NewType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone)]
pub enum IsArrayElement {
    False,
    FirstElement,
    OtherElement,
}

#[derive(Debug, Clone)]
pub enum EnumType {
    None,
    Descriptor,
    Value,
}

impl Default for EnumType {
    fn default() -> Self {
        Self::None
    }
}
