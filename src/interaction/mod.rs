/// Defines inbound Discord interaction payload models.
pub mod request;
/// Defines outbound Discord interaction callback response models.
pub mod response;

pub use request::{Interaction, InteractionKind, Member, User};
pub use response::{
    ApplicationCommandOptionChoice, InteractionCallbackData, InteractionCallbackType,
    InteractionResponse,
};
