use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};
use std::fmt;

// The enum and its impl blocks are now here.
// We make them `pub` so they can be used by other modules.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Id {
    String(String),
    Number(u64),
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Id::String(s) => write!(f, "{}", s),
            Id::Number(n) => write!(f, "{}", n),
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or an integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<Id, E>
            where
                E: de::Error,
            {
                Ok(Id::String(value.to_owned()))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Id, E>
            where
                E: de::Error,
            {
                Ok(Id::Number(value))
            }
        }

        deserializer.deserialize_any(IdVisitor)
    }
}
