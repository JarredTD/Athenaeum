//! Bot-authenticated Discord REST request construction.

use reqwest::{Client, Method, RequestBuilder};

/// Discord's versioned production REST API base URL.
pub const DISCORD_API_V10_BASE: &str = "https://discord.com/api/v10";

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
}

/// Tests Discord request construction without network access.
#[cfg(test)]
mod tests {
    use super::DiscordBotClient;

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
}
