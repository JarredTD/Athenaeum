//! AWS Secrets Manager support for application credentials.

use anyhow::{anyhow, Context, Result};

/// Retrieves required string fields from JSON-encoded AWS Secrets Manager values.
#[derive(Clone)]
pub struct JsonSecretReader {
    /// AWS client used to retrieve secret values.
    client: aws_sdk_secretsmanager::Client,
}

impl JsonSecretReader {
    /// Creates a reader backed by the supplied AWS Secrets Manager client.
    pub fn new(client: aws_sdk_secretsmanager::Client) -> Self {
        Self { client }
    }

    /// Retrieves one required non-blank string field from a JSON secret.
    ///
    /// Secrets must be ordinary JSON objects, such as `{ "token": "value" }`. Double-encoded
    /// JSON strings are deliberately unsupported so deployed credential formats remain explicit.
    ///
    /// # Errors
    ///
    /// Returns an error when AWS rejects the request, the secret is not a JSON object, or the
    /// requested field is missing, non-string, or blank.
    pub async fn get_required_string(&self, secret_id: &str, field: &str) -> Result<String> {
        let response = self
            .client
            .get_secret_value()
            .secret_id(secret_id)
            .send()
            .await
            .context("Failed to retrieve secret")?;
        let secret =
            response.secret_string().ok_or_else(|| anyhow!("Secret has no string value"))?;
        required_json_string_field(secret, field)
    }
}

/// Reads a required non-blank field from a canonical JSON object secret.
fn required_json_string_field(secret: &str, field: &str) -> Result<String> {
    let secret = serde_json::from_str::<serde_json::Value>(secret)
        .context("Secret must be a JSON object")?;
    let object = secret.as_object().ok_or_else(|| anyhow!("Secret must be a JSON object"))?;
    let value = object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("Secret is missing required non-blank {field} field"))?;
    Ok(value.to_string())
}

/// Tests canonical secret parsing without AWS requests.
#[cfg(test)]
mod tests {
    use super::required_json_string_field;

    /// Reads a non-blank string from an ordinary JSON object secret.
    #[test]
    fn reads_a_named_required_field() {
        let field = required_json_string_field(r#"{ "token": "bot-token" }"#, "token")
            .expect("fixture should contain a token");

        assert_eq!(field, "bot-token");
    }

    /// Rejects missing, blank, and double-encoded secret values.
    #[test]
    fn rejects_noncanonical_or_missing_required_fields() {
        assert!(required_json_string_field(r#"{ "key": "value" }"#, "token").is_err());
        assert!(required_json_string_field(r#"{ "token": " " }"#, "token").is_err());
        assert!(required_json_string_field(r#""{\\"token\\":\\"value\\"}""#, "token").is_err());
    }
}
