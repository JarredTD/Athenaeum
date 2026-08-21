use anyhow::{anyhow, Context, Result};

use crate::interaction::{Interaction, InteractionResponse};

/// Provides bot-authenticated Discord REST requests.
pub mod discord;
pub use discord::{AllowedMentions, ChannelMessage, CreatedChannelMessage, DiscordBotClient};

/// Base URL for Discord's REST API.
const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Delivers deferred and completed Discord interaction responses over HTTP.
#[derive(Clone)]
pub struct InteractionResponder {
    /// HTTP client used to call Discord interaction webhook endpoints.
    client: reqwest::Client,
    /// Base URL for Discord interaction webhook requests.
    api_base_url: String,
}

impl InteractionResponder {
    /// Creates an interaction responder that targets Discord's production REST API.
    pub fn new(client: reqwest::Client) -> Self {
        Self { client, api_base_url: DISCORD_API_BASE.to_string() }
    }

    /// Creates a responder that targets a Discord-compatible REST endpoint.
    ///
    /// # Arguments
    ///
    /// * `client` - HTTP client used to issue interaction webhook requests.
    /// * `api_base_url` - Base URL for a Discord-compatible REST API.
    pub fn with_api_base_url(client: reqwest::Client, api_base_url: impl Into<String>) -> Self {
        Self { client, api_base_url: api_base_url.into().trim_end_matches('/').to_string() }
    }

    /// Acknowledges a command before Discord's three-second interaction deadline.
    ///
    /// # Errors
    ///
    /// Returns an error when callback identifiers are missing or Discord rejects the acknowledgement.
    pub async fn defer_ephemeral<T>(&self, interaction: &Interaction<T>) -> Result<()> {
        let interaction_id = require_interaction_id(interaction)?;
        let token = require_interaction_token(interaction)?;
        self.client
            .post(format!("{}/interactions/{interaction_id}/{token}/callback", self.api_base_url))
            .json(&InteractionResponse::deferred_ephemeral())
            .send()
            .await
            .context("Discord interaction acknowledgement request failed")?
            .error_for_status()
            .context("Discord rejected the interaction acknowledgement")?;
        Ok(())
    }

    /// Replaces the deferred interaction message with a completed message response.
    ///
    /// # Errors
    ///
    /// Returns an error when identifiers or response data are absent, or Discord rejects the update.
    pub async fn update_original_response<T>(
        &self,
        interaction: &Interaction<T>,
        response: &InteractionResponse,
    ) -> Result<()> {
        let application_id = require_application_id(interaction)?;
        let token = require_interaction_token(interaction)?;
        let data = response
            .data
            .as_ref()
            .ok_or_else(|| anyhow!("Interaction response did not contain message data"))?;
        self.client
            .patch(format!(
                "{}/webhooks/{application_id}/{token}/messages/@original",
                self.api_base_url
            ))
            .json(data)
            .send()
            .await
            .context("Discord interaction response update request failed")?
            .error_for_status()
            .context("Discord rejected the interaction response update")?;
        Ok(())
    }
}

/// Returns the interaction ID required by Discord's callback endpoint.
fn require_interaction_id<T>(interaction: &Interaction<T>) -> Result<&str> {
    interaction.id.as_deref().ok_or_else(|| anyhow!("Missing interaction id"))
}

/// Returns the application ID required by Discord's webhook endpoint.
fn require_application_id<T>(interaction: &Interaction<T>) -> Result<&str> {
    interaction.application_id.as_deref().ok_or_else(|| anyhow!("Missing application id"))
}

/// Returns the token required by Discord interaction response endpoints.
fn require_interaction_token<T>(interaction: &Interaction<T>) -> Result<&str> {
    interaction.token.as_deref().ok_or_else(|| anyhow!("Missing interaction token"))
}

/// Tests Discord interaction webhook requests against a local HTTP server.
#[cfg(test)]
mod tests {
    use super::{InteractionResponder, DISCORD_API_BASE};
    use crate::interaction::{Interaction, InteractionKind, InteractionResponse};
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    /// Confirms that command deferral uses Discord's callback endpoint and payload.
    #[tokio::test]
    async fn defers_ephemeral_interaction() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/interactions/interaction/token/callback"))
            .and(body_json(serde_json::json!({ "type": 5, "data": { "flags": 64 } })))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;
        let responder =
            InteractionResponder::with_api_base_url(reqwest::Client::new(), server.uri());

        responder
            .defer_ephemeral(&interaction())
            .await
            .expect("mocked Discord deferral should succeed");
    }

    /// Uses Discord's production API when no compatible endpoint is supplied.
    #[test]
    fn creates_a_production_responder() {
        let responder = InteractionResponder::new(reqwest::Client::new());

        assert_eq!(responder.api_base_url, DISCORD_API_BASE);
    }

    /// Confirms that a deferred response is replaced through Discord's original-message endpoint.
    #[tokio::test]
    async fn updates_deferred_interaction() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/webhooks/application/token/messages/@original"))
            .and(body_json(serde_json::json!({ "content": "Done", "flags": 64 })))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let responder =
            InteractionResponder::with_api_base_url(reqwest::Client::new(), server.uri());

        responder
            .update_original_response(&interaction(), &InteractionResponse::ephemeral("Done"))
            .await
            .expect("mocked Discord update should succeed");
    }

    /// Builds an interaction with the identifiers required by Discord webhook endpoints.
    fn interaction() -> Interaction {
        Interaction {
            id: Some("interaction".to_string()),
            application_id: Some("application".to_string()),
            token: Some("token".to_string()),
            kind: InteractionKind::ApplicationCommand,
            data: None,
            guild_id: None,
            channel_id: None,
            member: None,
        }
    }
}
