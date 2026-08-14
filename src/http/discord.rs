//! Bot-authenticated Discord REST request construction.

use anyhow::{Context, Result};
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::discord::DiscordSnowflake;

/// Discord's versioned production REST API base URL.
pub const DISCORD_API_V10_BASE: &str = "https://discord.com/api/v10";

/// Explicit Discord mention policy for a channel message.
#[derive(Debug, Clone, Serialize)]
pub struct AllowedMentions {
    /// Mention categories Discord must not infer from message text.
    parse: Vec<String>,
    /// Role snowflakes explicitly permitted to receive a mention.
    roles: Vec<String>,
}

impl AllowedMentions {
    /// Allows exactly one stored role to receive a mention from the message content.
    pub fn only_role(role_id: &DiscordSnowflake) -> Self {
        Self { parse: Vec::new(), roles: vec![role_id.as_str().to_string()] }
    }
}

/// Content and mention policy for a Discord channel-message request.
#[derive(Debug, Serialize)]
pub struct ChannelMessage<'a> {
    /// Plain Discord message content.
    content: &'a str,
    /// Explicit policy that controls which mentions Discord may resolve.
    allowed_mentions: AllowedMentions,
}

impl<'a> ChannelMessage<'a> {
    /// Creates a channel message with the supplied explicit mention policy.
    pub fn new(content: &'a str, allowed_mentions: AllowedMentions) -> Self {
        Self { content, allowed_mentions }
    }
}

/// Discord response fields returned after a channel message is created.
#[derive(Debug, Deserialize)]
pub struct CreatedChannelMessage {
    /// Snowflake assigned by Discord to the created message.
    pub id: DiscordSnowflake,
}

/// Builds authenticated requests to Discord's bot REST API.
#[derive(Clone)]
pub struct DiscordBotClient {
    /// Reusable HTTP client used to send Discord API requests.
    client: Client,
    /// Preformatted bot authorization header value.
    authorization: String,
    /// Base URL used for production and mock Discord endpoints.
    api_base_url: String,
}

impl DiscordBotClient {
    /// Creates a client targeting Discord's production version 10 REST API.
    pub fn new(client: Client, bot_token: impl Into<String>) -> Self {
        Self::with_api_base_url(client, bot_token, DISCORD_API_V10_BASE)
    }

    /// Creates a client targeting a Discord-compatible REST endpoint.
    pub fn with_api_base_url(
        client: Client,
        bot_token: impl Into<String>,
        api_base_url: impl Into<String>,
    ) -> Self {
        Self {
            client,
            authorization: format!("Bot {}", bot_token.into()),
            api_base_url: api_base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Builds one bot-authorized request for a relative Discord API path.
    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, format!("{}/{}", self.api_base_url, path.trim_start_matches('/')))
            .header("Authorization", &self.authorization)
    }

    /// Builds one bot-authorized GET request.
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.request(Method::GET, path)
    }

    /// Builds one bot-authorized POST request.
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.request(Method::POST, path)
    }

    /// Builds one bot-authorized PUT request.
    pub fn put(&self, path: &str) -> RequestBuilder {
        self.request(Method::PUT, path)
    }

    /// Builds one bot-authorized DELETE request.
    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.request(Method::DELETE, path)
    }

    /// Creates a channel message with an explicit mention policy.
    ///
    /// # Errors
    ///
    /// Returns an error when Discord rejects the request or returns an invalid message identity.
    pub async fn create_channel_message(
        &self,
        channel_id: &DiscordSnowflake,
        message: ChannelMessage<'_>,
    ) -> Result<CreatedChannelMessage> {
        self.post(&format!("channels/{}/messages", channel_id.as_str()))
            .json(&message)
            .send()
            .await
            .context("Discord channel message request failed")?
            .error_for_status()
            .context("Discord rejected the channel message")?
            .json()
            .await
            .context("Discord returned an invalid channel message")
    }
}

/// Tests Discord request construction without network access.
#[cfg(test)]
mod tests {
    use super::{AllowedMentions, ChannelMessage, DiscordBotClient};
    use crate::discord::DiscordSnowflake;
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    /// Adds the bot authorization header and normalizes a relative path.
    #[test]
    fn builds_authorized_relative_requests() {
        let request = DiscordBotClient::with_api_base_url(
            reqwest::Client::new(),
            "token",
            "https://discord.example/",
        )
        .get("/channels/123")
        .build()
        .expect("request should build");

        assert_eq!(request.url().as_str(), "https://discord.example/channels/123");
        assert_eq!(request.headers()["authorization"], "Bot token");
    }

    /// Builds every supported authenticated request method.
    #[test]
    fn builds_all_supported_request_methods() {
        let client = DiscordBotClient::new(reqwest::Client::new(), "token");
        for request in [client.post("messages"), client.put("roles"), client.delete("roles")] {
            assert_eq!(
                request.build().expect("request should build").headers()["authorization"],
                "Bot token"
            );
        }
    }

    /// Sends a channel message while allowing only the configured role mention.
    #[tokio::test]
    async fn creates_a_role_limited_channel_message() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/channels/222222222222222222/messages"))
            .and(body_json(serde_json::json!({
                "content": "<@&123456789012345678> 5 days until Universal",
                "allowed_mentions": { "parse": [], "roles": ["123456789012345678"] }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "333333333333333333"
            })))
            .mount(&server)
            .await;
        let client =
            DiscordBotClient::with_api_base_url(reqwest::Client::new(), "token", server.uri());
        let channel_id = snowflake("222222222222222222");
        let role_id = snowflake("123456789012345678");

        let message = client
            .create_channel_message(
                &channel_id,
                ChannelMessage::new(
                    "<@&123456789012345678> 5 days until Universal",
                    AllowedMentions::only_role(&role_id),
                ),
            )
            .await
            .expect("mocked Discord message should succeed");

        assert_eq!(message.id.as_str(), "333333333333333333");
    }

    /// Builds a validated Discord snowflake used by HTTP fixtures.
    fn snowflake(value: &str) -> DiscordSnowflake {
        DiscordSnowflake::new(value).expect("fixture snowflake should be valid")
    }
}
