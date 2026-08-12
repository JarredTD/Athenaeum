use serde::Deserialize;
use serde_repr::Deserialize_repr;

/// Identifies the kind of Discord interaction received by the webhook.
#[derive(Debug, Deserialize_repr, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InteractionKind {
    /// Initial verification request sent by Discord.
    Ping = 1,
    /// Invocation of an application command.
    ApplicationCommand = 2,
    /// Request for command-option autocomplete choices.
    ApplicationCommandAutocomplete = 4,
    /// An interaction type not yet modeled by Athenaeum.
    #[serde(other)]
    Unknown,
}

/// Represents a Discord interaction envelope with application-defined command data.
#[derive(Debug, Deserialize, Clone)]
pub struct Interaction<T = serde_json::Value> {
    /// Snowflake identifier for this interaction.
    #[serde(default)]
    pub id: Option<String>,
    /// Snowflake identifier for the Discord application receiving the interaction.
    #[serde(default)]
    pub application_id: Option<String>,
    /// One-time interaction token used to send follow-up interaction responses.
    #[serde(default)]
    pub token: Option<String>,
    /// Type of interaction supplied by Discord.
    #[serde(rename = "type")]
    pub kind: InteractionKind,
    /// Bot-specific payload for command-related interactions.
    #[serde(default)]
    pub data: Option<T>,
    /// ID of the guild in which the interaction occurred.
    #[serde(default)]
    pub guild_id: Option<String>,
    /// Invoking member details for guild interactions.
    #[serde(default)]
    pub member: Option<Member>,
}

/// Represents the member that invoked a guild interaction.
#[derive(Debug, Deserialize, Clone)]
pub struct Member {
    /// Guild permissions granted to the member as a decimal bitfield string.
    #[serde(default)]
    pub permissions: Option<String>,
    /// Discord user associated with the member.
    pub user: User,
}

/// Represents the invoking Discord user.
#[derive(Debug, Deserialize, Clone)]
pub struct User {
    /// Discord snowflake identifier for the user.
    pub id: String,
}

/// Tests deserialization of the protocol-level Discord interaction envelope.
#[cfg(test)]
mod tests {
    use super::{Interaction, InteractionKind};

    /// Retains common command envelope fields without interpreting application-specific data.
    #[test]
    fn deserializes_generic_command_envelope() {
        let interaction: Interaction = serde_json::from_value(serde_json::json!({
            "id": "interaction-id",
            "application_id": "application-id",
            "token": "interaction-token",
            "type": 2,
            "guild_id": "guild-id",
            "member": { "permissions": "8", "user": { "id": "user-id" } },
            "data": { "name": "role", "options": [{ "name": "save" }] }
        }))
        .expect("fixture should match Discord's interaction schema");

        assert!(matches!(interaction.kind, InteractionKind::ApplicationCommand));
        assert_eq!(interaction.guild_id.as_deref(), Some("guild-id"));
        assert_eq!(
            interaction.member.as_ref().map(|member| member.user.id.as_str()),
            Some("user-id")
        );
        assert_eq!(interaction.data.expect("data should be retained")["name"], "role");
    }

    /// Deserializes unsupported interaction kinds without rejecting a valid request envelope.
    #[test]
    fn deserializes_unknown_interaction_kind() {
        let interaction: Interaction = serde_json::from_str(r#"{ "type": 99 }"#)
            .expect("unknown interaction kind should deserialize");

        assert!(matches!(interaction.kind, InteractionKind::Unknown));
    }
}
