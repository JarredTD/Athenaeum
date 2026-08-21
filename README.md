# Athenaeum

[![CI](https://github.com/JarredTD/Athenaeum/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/JarredTD/Athenaeum/actions/workflows/ci.yml)
[![License: AGPL-3.0-only](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](LICENSE)

Athenaeum is a Rust library for Discord interaction endpoints and shared Discord bot primitives. It verifies Discord signatures, decodes common request envelopes, and serializes protocol-correct responses.

## Features

- Ed25519 verification of Discord's timestamped interaction requests.
- A generic `Interaction<T>` envelope: applications choose their own command-data type.
- Models and helpers for pings, ephemeral messages, deferrals, and autocomplete responses.
- Validated Discord snowflakes and effective channel-permission evaluation.
- An optional `http` feature for interaction callbacks, bot-authenticated Discord REST requests,
  and channel messages with explicit role-only mentions.
- An optional `aws-secrets` feature for required JSON fields in AWS Secrets Manager values.

## Setup

Add Athenaeum with the features your application needs:

```toml
[dependencies]
athenaeum = { git = "https://github.com/JarredTD/Athenaeum.git", features = ["http", "aws-secrets"] }
```

The core interaction types, verifier, identities, and permission evaluator have no HTTP or AWS dependency. Enable `http` for interaction callbacks or bot REST requests, and `aws-secrets` when the runtime reads JSON credentials from Secrets Manager.

## Usage

```rust
use athenaeum::{
    auth::InteractionVerifier,
    interaction::{Interaction, InteractionResponse},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct CommandData {
    name: String,
}

fn verify_request(
    verifier: &InteractionVerifier,
    signature: &str,
    timestamp: &str,
    body: &[u8],
    public_key: &str,
) -> anyhow::Result<Interaction<CommandData>> {
    verifier.verify(signature, timestamp, body, public_key)?;
    Ok(serde_json::from_slice(body)?)
}

let response = InteractionResponse::ephemeral("All set.");
```

Verify the exact raw body before parsing or transforming it. Discord signs the raw timestamp-and-body payload.

## License

Athenaeum is licensed under the [GNU Affero General Public License v3.0 only](LICENSE).
