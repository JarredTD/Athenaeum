#![deny(missing_docs, clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
//! Reusable primitives for Discord interaction webhooks.

/// Implements Discord request verification.
pub mod auth;
/// Implements optional HTTP interaction callback delivery.
#[cfg(feature = "http")]
pub mod http;
/// Defines Discord interaction payload and callback models.
pub mod interaction;
