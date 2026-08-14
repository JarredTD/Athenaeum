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

/// Discord permission bits granted to a guild member or permitted in a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscordPermissions(u64);

impl DiscordPermissions {
    /// Grants every permission and bypasses channel overwrites.
    pub const ADMINISTRATOR: Self = Self(1 << 3);
    /// Permits ordinary messages in a text channel.
    pub const SEND_MESSAGES: Self = Self(1 << 11);
    /// Permits role mentions when Discord's mention policy allows them.
    pub const MENTION_EVERYONE: Self = Self(1 << 17);

    /// Creates a permission set from raw Discord bit flags.
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Parses Discord's decimal permission representation.
    ///
    /// # Errors
    ///
    /// Returns an error when the decimal value cannot fit in Discord's permission bitfield.
    pub fn from_decimal(value: &str) -> Result<Self, DiscordPermissionError> {
        value.parse().map(Self).map_err(|_| DiscordPermissionError)
    }

    /// Returns whether every bit in `required` is present.
    pub const fn contains(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }

    /// Combines two independently granted permission sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns raw permission bits for serialization or diagnostics.
    pub const fn bits(self) -> u64 {
        self.0
    }
}

/// Explains why a Discord permission field cannot be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscordPermissionError;

impl Display for DiscordPermissionError {
    /// Formats an actionable permission parsing error.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Discord permissions must be an unsigned decimal bitfield")
    }
}

impl std::error::Error for DiscordPermissionError {}

/// Guild role permissions used to calculate a member's base access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildRolePermissions {
    /// Guild-local role identity.
    pub id: DiscordSnowflake,
    /// Permissions granted by this role.
    pub permissions: DiscordPermissions,
}

/// Identifies whether a channel overwrite targets a role or an individual member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionOverwriteKind {
    /// An overwrite that applies to every member holding a role.
    Role,
    /// An overwrite that applies to one guild member.
    Member,
}

/// One Discord channel permission overwrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPermissionOverwrite {
    /// Role or member affected by the overwrite.
    pub subject_id: DiscordSnowflake,
    /// Type of Discord subject affected by the overwrite.
    pub kind: PermissionOverwriteKind,
    /// Permission bits explicitly granted by the overwrite.
    pub allow: DiscordPermissions,
    /// Permission bits explicitly denied by the overwrite.
    pub deny: DiscordPermissions,
}

/// Calculates a member's effective channel permissions using Discord's documented precedence.
pub fn effective_channel_permissions(
    guild_id: &DiscordSnowflake,
    member_id: &DiscordSnowflake,
    member_role_ids: &[DiscordSnowflake],
    guild_roles: &[GuildRolePermissions],
    overwrites: &[ChannelPermissionOverwrite],
) -> DiscordPermissions {
    let mut permissions = guild_roles
        .iter()
        .filter(|role| &role.id == guild_id || member_role_ids.contains(&role.id))
        .fold(DiscordPermissions::from_bits(0), |permissions, role| {
            permissions.union(role.permissions)
        });
    if permissions.contains(DiscordPermissions::ADMINISTRATOR) {
        return DiscordPermissions::from_bits(u64::MAX);
    }
    apply_overwrite(
        &mut permissions,
        overwrites.iter().find(|overwrite| {
            overwrite.kind == PermissionOverwriteKind::Role && &overwrite.subject_id == guild_id
        }),
    );
    let (role_deny, role_allow) = overwrites
        .iter()
        .filter(|overwrite| {
            overwrite.kind == PermissionOverwriteKind::Role
                && member_role_ids.contains(&overwrite.subject_id)
        })
        .fold(
            (DiscordPermissions::from_bits(0), DiscordPermissions::from_bits(0)),
            |(deny, allow), overwrite| (deny.union(overwrite.deny), allow.union(overwrite.allow)),
        );
    permissions =
        DiscordPermissions::from_bits((permissions.bits() & !role_deny.bits()) | role_allow.bits());
    apply_overwrite(
        &mut permissions,
        overwrites.iter().find(|overwrite| {
            overwrite.kind == PermissionOverwriteKind::Member && &overwrite.subject_id == member_id
        }),
    );
    permissions
}

/// Applies one overwrite, with denies taking precedence over grants within that overwrite.
fn apply_overwrite(
    permissions: &mut DiscordPermissions,
    overwrite: Option<&ChannelPermissionOverwrite>,
) {
    if let Some(overwrite) = overwrite {
        *permissions = DiscordPermissions::from_bits(
            (permissions.bits() & !overwrite.deny.bits()) | overwrite.allow.bits(),
        );
    }
}

/// Tests snowflake validation and JSON representation.
#[cfg(test)]
mod tests {
    use super::{
        effective_channel_permissions, ChannelPermissionOverwrite, DiscordPermissions,
        DiscordSnowflake, GuildRolePermissions, PermissionOverwriteKind,
    };

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

    /// Deserializes only the same validated snowflake form used by Discord APIs.
    #[test]
    fn deserializes_valid_snowflakes() {
        let snowflake: DiscordSnowflake =
            serde_json::from_str(r#""123456789012345678""#).expect("fixture should deserialize");

        assert_eq!(snowflake.as_str(), "123456789012345678");
        assert!(serde_json::from_str::<DiscordSnowflake>(r#""0""#).is_err());
    }

    /// Parses and combines the decimal permission values supplied by Discord.
    #[test]
    fn parses_and_combines_permissions() {
        let permissions = DiscordPermissions::from_decimal("2048")
            .expect("send-messages bit should parse")
            .union(DiscordPermissions::MENTION_EVERYONE);

        assert!(permissions.contains(DiscordPermissions::SEND_MESSAGES));
        assert!(permissions.contains(DiscordPermissions::MENTION_EVERYONE));
        assert!(DiscordPermissions::from_decimal("not-a-number").is_err());
    }

    /// Lets administrators retain every permission regardless of channel overwrites.
    #[test]
    fn administrators_bypass_channel_overwrites() {
        let guild_id = snowflake("111111111111111111");
        let member_id = snowflake("222222222222222222");
        let permissions = effective_channel_permissions(
            &guild_id,
            &member_id,
            &[],
            &[GuildRolePermissions {
                id: guild_id.clone(),
                permissions: DiscordPermissions::ADMINISTRATOR,
            }],
            &[ChannelPermissionOverwrite {
                subject_id: guild_id.clone(),
                kind: PermissionOverwriteKind::Role,
                allow: DiscordPermissions::from_bits(0),
                deny: DiscordPermissions::SEND_MESSAGES,
            }],
        );

        assert!(permissions.contains(DiscordPermissions::SEND_MESSAGES));
    }

    /// Applies role denials before a member-specific grant restores a required permission.
    #[test]
    fn resolves_channel_permission_overwrites_in_discord_order() {
        let guild_id = snowflake("111111111111111111");
        let member_id = snowflake("222222222222222222");
        let role_id = snowflake("333333333333333333");
        let required =
            DiscordPermissions::SEND_MESSAGES.union(DiscordPermissions::MENTION_EVERYONE);
        let permissions = effective_channel_permissions(
            &guild_id,
            &member_id,
            std::slice::from_ref(&role_id),
            &[GuildRolePermissions { id: role_id.clone(), permissions: required }],
            &[
                ChannelPermissionOverwrite {
                    subject_id: role_id.clone(),
                    kind: PermissionOverwriteKind::Role,
                    allow: DiscordPermissions::from_bits(0),
                    deny: DiscordPermissions::SEND_MESSAGES,
                },
                ChannelPermissionOverwrite {
                    subject_id: member_id.clone(),
                    kind: PermissionOverwriteKind::Member,
                    allow: DiscordPermissions::SEND_MESSAGES,
                    deny: DiscordPermissions::from_bits(0),
                },
            ],
        );

        assert!(permissions.contains(required));
    }

    /// Builds a validated snowflake used in permission tests.
    fn snowflake(value: &str) -> DiscordSnowflake {
        DiscordSnowflake::new(value).expect("fixture snowflake should be valid")
    }
}
