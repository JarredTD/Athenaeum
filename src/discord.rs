//! Reusable Discord identity types.

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A validated non-zero decimal Discord snowflake identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiscordSnowflake(String);

impl DiscordSnowflake {
    /// Validates and creates a Discord snowflake from a decimal identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier is empty, longer than Discord's current 20-digit
    /// representation, non-decimal, or zero.
    pub fn new(value: impl Into<String>) -> Result<Self, DiscordSnowflakeError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 20
            && value.bytes().all(|byte| byte.is_ascii_digit())
            && value.bytes().any(|byte| byte != b'0');
        if valid {
            Ok(Self(value))
        } else {
            Err(DiscordSnowflakeError)
        }
    }

    /// Returns the canonical decimal representation used by Discord APIs.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DiscordSnowflake {
    /// Writes the snowflake's canonical decimal representation.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for DiscordSnowflake {
    /// Serializes a snowflake as Discord's ordinary JSON string identifier.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for DiscordSnowflake {
    /// Deserializes and validates Discord's ordinary JSON string identifier.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Explains why a value cannot represent a Discord snowflake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscordSnowflakeError;

impl Display for DiscordSnowflakeError {
    /// Formats an actionable snowflake-validation error.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Discord snowflake must be a non-zero decimal identifier")
    }
}

impl std::error::Error for DiscordSnowflakeError {}

/// Tests snowflake validation and JSON representation.
#[cfg(test)]
mod tests {
    use super::DiscordSnowflake;

    /// Accepts the decimal snowflakes returned by Discord APIs.
    #[test]
    fn accepts_and_serializes_a_valid_snowflake() {
        let snowflake =
            DiscordSnowflake::new("123456789012345678").expect("fixture snowflake should be valid");

        assert_eq!(snowflake.as_str(), "123456789012345678");
        assert_eq!(
            serde_json::to_string(&snowflake).expect("snowflake should serialize"),
            r#""123456789012345678""#
        );
    }

    /// Rejects values that cannot identify a Discord resource.
    #[test]
    fn rejects_invalid_snowflakes() {
        for value in ["", "0", "not-a-snowflake", "123456789012345678901"] {
            assert!(DiscordSnowflake::new(value).is_err());
        }
    }
}
