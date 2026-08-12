# Athenaeum

Athenaeum is a Rust library for Discord interactions endpoints. It verifies Discord signatures, decodes common request envelopes, and serializes protocol-correct responses.

It is not a bot framework. Command definitions, business rules, persistence, and hosting remain application concerns. Athenaeum provides only Discord interaction boundary primitives.

## What it provides

- Ed25519 verification of Discord's timestamped interaction requests.
- A generic `Interaction<T>` envelope: applications choose their own command-data type.
- Models and helpers for pings, ephemeral messages, deferrals, and autocomplete responses.
- An optional `http` feature for deferring and updating interaction responses through Discord's REST API.

## Installation

Athenaeum is currently consumed as a local path dependency:

```toml
[dependencies]
athenaeum = { path = "../Athenaeum", features = ["http"] }
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

## Development

Athenaeum uses the Rust version in [`rust-toolchain.toml`](rust-toolchain.toml). The complete local quality suite is:

```sh
cargo fmt --check
cargo check --no-default-features
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo doc --all-features --no-deps --document-private-items
cargo llvm-cov --all-features --fail-under-lines 90
```

CI runs the same checks with pinned GitHub Actions. The coverage threshold applies to the library as a whole, including the optional HTTP support.

## Scope and stability

The public API is intentionally small. Applications should compose Athenaeum's protocol primitives and retain application-specific behavior. Once a public remote exists, dependencies should use a pinned release tag or commit rather than a moving branch.

## License

Athenaeum is licensed under the [GNU Affero General Public License v3.0 only](LICENSE).
