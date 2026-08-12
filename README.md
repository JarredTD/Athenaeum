# Athenaeum

[![CI](https://github.com/JarredTD/Athenaeum/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/JarredTD/Athenaeum/actions/workflows/ci.yml)
[![License: AGPL-3.0-only](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](LICENSE)

Athenaeum is a Rust library for Discord interactions endpoints. It verifies Discord signatures, decodes common request envelopes, and serializes protocol-correct responses.

## What it provides

- Ed25519 verification of Discord's timestamped interaction requests.
- A generic `Interaction<T>` envelope: applications choose their own command-data type.
- Models and helpers for pings, ephemeral messages, deferrals, and autocomplete responses.
- An optional `http` feature for deferring and updating interaction responses through Discord's REST API.

## Installation

Use the published repository at the `v0.1.0` release tag:

```toml
[dependencies]
athenaeum = { git = "https://github.com/JarredTD/Athenaeum.git", tag = "v0.1.0", features = ["http"] }
```

The core interaction types and verifier have no HTTP dependency. Enable `http` when the application sends interaction callback requests through Athenaeum.

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
