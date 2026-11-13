//! Shapes shared by several families.

use std::fmt;

use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Whether `key` is a namespaced extension key: it must begin
/// `com.{vendor}.`, with `{vendor}` drawn from `[a-z0-9-]+`, as every
/// schema's `extensions.patternProperties` requires.
#[must_use]
pub fn is_extension_key(key: &str) -> bool {
    let Some(rest) = key.strip_prefix("com.") else {
        return false;
    };
    match rest.find('.') {
        Some(end) => {
            let vendor = &rest[..end];
            !vendor.is_empty()
                && vendor
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        }
        None => false,
    }
}

/// An extension key outside the `com.{vendor}.` namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidExtensionKey(String);

impl InvalidExtensionKey {
    /// The key that was refused.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.0
    }

    /// The refused key, by value.
    #[must_use]
    pub fn into_key(self) -> String {
        self.0
    }
}

impl fmt::Display for InvalidExtensionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "extension key {:?} is not namespaced com.{{vendor}}.",
            self.0
        )
    }
}

impl std::error::Error for InvalidExtensionKey {}

/// Vendor fields, keyed `com.{vendor}.*`. Values are opaque: a consumer
/// ignores keys it does not recognise (conformance §5), and a key outside the
/// `com.` namespace is refused on decode, as the schemas refuse it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extensions(Map<String, Value>);

impl Extensions {
    /// An empty set of extensions.
    #[must_use]
    pub fn new() -> Self {
        Extensions(Map::new())
    }

    /// Inserts a vendor field, returning the value it replaced.
    ///
    /// # Errors
    ///
    /// [`InvalidExtensionKey`] if the key is not `com.{vendor}.`-namespaced;
    /// nothing is inserted.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: Value,
    ) -> Result<Option<Value>, InvalidExtensionKey> {
        let key = key.into();
        if is_extension_key(&key) {
            Ok(self.0.insert(key, value))
        } else {
            Err(InvalidExtensionKey(key))
        }
    }

    /// The value under `key`, if present.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// The fields, in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter()
    }

    /// Number of fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> IntoIterator for &'a Extensions {
    type Item = (&'a String, &'a Value);
    type IntoIter = serde_json::map::Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Serialize for Extensions {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Extensions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ExtensionsVisitor;

        impl<'de> Visitor<'de> for ExtensionsVisitor {
            type Value = Extensions;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an object keyed com.{vendor}.*")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Extensions, A::Error> {
                let mut out = Map::new();
                while let Some(key) = access.next_key::<String>()? {
                    if !is_extension_key(&key) {
                        return Err(de::Error::custom(InvalidExtensionKey(key)));
                    }
                    let value: Value = access.next_value()?;
                    out.insert(key, value);
                }
                Ok(Extensions(out))
            }
        }

        deserializer.deserialize_map(ExtensionsVisitor)
    }
}

/// Whether `value` is a vendor registry value, `x-[a-z0-9_]+`, as the
/// governed-but-extensible registries (`asset_type`, `source_type`,
/// `transform_type`) admit beside their seed values.
#[must_use]
pub fn is_registry_extension(value: &str) -> bool {
    value.strip_prefix("x-").is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    })
}

/// A value that is neither a registry's seed value nor an `x-` vendor value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownRegistryValue {
    /// The registry, such as `AssetType`.
    pub registry: &'static str,
    /// The value that was refused.
    pub value: String,
}

impl fmt::Display for UnknownRegistryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} is neither a {} seed value nor an x- vendor value",
            self.value, self.registry
        )
    }
}

impl std::error::Error for UnknownRegistryValue {}

/// Declares a governed-but-extensible registry: the seed values the schema
/// lists, plus `Extension` for an `x-`-prefixed vendor value.
macro_rules! registry {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$vmeta])* $variant,)+
            /// A vendor value, `x-[a-z0-9_]+`.
            Extension(String),
        }

        impl $name {
            /// Every seed value, in schema order.
            pub const SEED: &'static [$name] = &[$($name::$variant,)+];

            /// The wire value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                match self {
                    $($name::$variant => $wire,)+
                    $name::Extension(value) => value,
                }
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::model::common::UnknownRegistryValue;

            /// Parses a wire value: a seed value or an `x-` vendor value.
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($wire => Ok($name::$variant),)+
                    other if $crate::model::common::is_registry_extension(other) => {
                        Ok($name::Extension(other.to_owned()))
                    }
                    _ => Err($crate::model::common::UnknownRegistryValue {
                        registry: stringify!($name),
                        value: value.to_owned(),
                    }),
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}
pub(crate) use registry;

/// A TAMS timerange, or a list of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TimeRanges {
    /// One range, such as `[0:0_10:0)`.
    One(String),
    /// Several ranges.
    Many(Vec<String>),
}

/// A non-TAMS media store reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Locator {
    /// The store.
    pub store: String,
    /// The reference within the store; the schema's `ref`.
    #[serde(rename = "ref")]
    pub reference: String,
}

/// The audit actor shape shared by `system.audit` and telling transforms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    /// Who acted.
    pub actor_id: String,
    /// What kind of actor, such as `user` or `system`.
    pub actor_type: String,
}
