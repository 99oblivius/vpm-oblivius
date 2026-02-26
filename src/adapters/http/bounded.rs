use std::{fmt, ops::Deref};

use serde::de::{self, Deserialize, Deserializer};

/// A string that rejects deserialization if empty or longer than `N` bytes.
///
/// Use in payload structs to enforce length limits at parse time:
/// ```ignore
/// #[derive(Deserialize)]
/// struct Payload {
///     name: Bounded<128>,
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Bounded<const N: usize>(String);

impl<const N: usize> Deref for Bounded<N> {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl<const N: usize> fmt::Display for Bounded<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<const N: usize> AsRef<str> for Bounded<N> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<const N: usize> From<Bounded<N>> for String {
    fn from(b: Bounded<N>) -> Self {
        b.0
    }
}

impl<'de, const N: usize> Deserialize<'de> for Bounded<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            return Err(de::Error::custom("field must not be empty"));
        }
        if s.len() > N {
            return Err(de::Error::custom(format!(
                "field exceeds maximum length of {N}"
            )));
        }
        Ok(Bounded(s))
    }
}
