use serde::Serialize;
use serde_repr::Serialize_repr;

bitflags::bitflags! {
    /// Bit flags that control Discord interaction message behavior.
    pub struct MessageFlags: u64 {
        /// Limits the message's visibility to the invoking user.
        const EPHEMERAL = 1 << 6;
    }
}

/// Identifies the callback protocol response sent to Discord.
#[derive(Debug, Copy, Clone, Serialize_repr)]
#[repr(u8)]
pub enum InteractionCallbackType {
    /// Acknowledges a Discord ping interaction.
    Pong = 1,
    /// Creates an immediate interaction response message.
    ChannelMessageWithSource = 4,
    /// Defers an ephemeral message while command work continues asynchronously.
    DeferredChannelMessageWithSource = 5,
    /// Supplies choices for an autocomplete interaction.
    ApplicationCommandAutocompleteResult = 8,
    /// Opens a modal for user input.
    Modal = 9,
}

/// Represents a response to a Discord interaction webhook.
#[derive(Debug, Serialize)]
pub struct InteractionResponse {
    /// Callback type that controls Discord's response handling.
    #[serde(rename = "type")]
    pub kind: InteractionCallbackType,
    /// Optional callback payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<InteractionCallbackData>,
}

/// Contains message or autocomplete data in an interaction response.
#[derive(Debug, Serialize)]
pub struct InteractionCallbackData {
    /// Message text displayed to the invoking user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Discord message flags, such as ephemeral visibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u64>,
    /// Choices returned for an autocomplete interaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<ApplicationCommandOptionChoice>>,
    /// Developer-defined identifier returned when the modal is submitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    /// User-facing modal title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Input fields displayed in a modal response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<InteractionComponent>>,
}

/// Represents one selectable autocomplete choice.
#[derive(Debug, Serialize)]
pub struct ApplicationCommandOptionChoice {
    /// Human-readable text shown in Discord.
    pub name: String,
    /// Value submitted to the command when selected.
    pub value: String,
}

/// A Discord modal text-input component.
#[derive(Debug, Serialize)]
pub struct ModalTextInput {
    /// Discord component type for a text input.
    #[serde(rename = "type")]
    component_type: u8,
    /// Developer-defined field identifier included in modal submission data.
    custom_id: String,
    /// User-facing field label.
    label: String,
    /// Text-input style: one line (`1`) or multiple lines (`2`).
    style: u8,
    /// Whether Discord requires a response for this field.
    required: bool,
    /// Optional initial value shown in the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

/// A Discord action row that contains one modal text input.
#[derive(Debug, Serialize)]
pub struct ModalActionRow {
    /// Discord component type for an action row.
    #[serde(rename = "type")]
    component_type: u8,
    /// Text input displayed in this row.
    components: Vec<ModalTextInput>,
}

/// A Discord button component used in an ordinary interaction response.
#[derive(Debug, Serialize)]
pub struct Button {
    /// Discord component type for a button.
    #[serde(rename = "type")]
    component_type: u8,
    /// Developer-defined value returned in the button interaction.
    custom_id: String,
    /// User-visible button label.
    label: String,
    /// Button style: primary (`1`), secondary (`2`), success (`3`), or danger (`4`).
    style: u8,
}

/// A Discord action row containing ordinary message buttons.
#[derive(Debug, Serialize)]
pub struct ButtonActionRow {
    /// Discord component type for an action row.
    #[serde(rename = "type")]
    component_type: u8,
    /// Buttons shown in this row.
    components: Vec<Button>,
}

/// One action-row variant accepted by Discord interaction response payloads.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum InteractionComponent {
    /// A row containing one modal text input.
    Modal(ModalActionRow),
    /// A row containing ordinary message buttons.
    Buttons(ButtonActionRow),
}

impl ModalActionRow {
    /// Wraps one text input in the action row required by Discord's modal format.
    fn text_input(input: ModalTextInput) -> Self {
        Self { component_type: 1, components: vec![input] }
    }
}

impl ModalTextInput {
    /// Creates a required single-line text input.
    pub fn short(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            component_type: 4,
            custom_id: custom_id.into(),
            label: label.into(),
            style: 1,
            required: true,
            value: None,
        }
    }
}

impl Button {
    /// Creates a primary button for the principal action in a response.
    pub fn primary(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { component_type: 2, custom_id: custom_id.into(), label: label.into(), style: 1 }
    }

    /// Creates a secondary button for a non-destructive response action.
    pub fn secondary(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { component_type: 2, custom_id: custom_id.into(), label: label.into(), style: 2 }
    }

    /// Creates a danger button for an irreversible response action.
    pub fn danger(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { component_type: 2, custom_id: custom_id.into(), label: label.into(), style: 4 }
    }
}

impl ButtonActionRow {
    /// Groups up to five buttons into the action row required by Discord.
    pub fn new(buttons: Vec<Button>) -> Self {
        Self { component_type: 1, components: buttons }
    }
}

impl InteractionResponse {
    /// Builds a response that acknowledges a Discord ping.
    pub fn pong() -> Self {
        Self { kind: InteractionCallbackType::Pong, data: None }
    }

    /// Builds an ephemeral message response visible only to the invoking user.
    ///
    /// # Arguments
    ///
    /// * `content` - Text displayed in the response message.
    pub fn ephemeral(content: impl Into<String>) -> Self {
        Self {
            kind: InteractionCallbackType::ChannelMessageWithSource,
            data: Some(InteractionCallbackData {
                content: Some(content.into()),
                flags: Some(MessageFlags::EPHEMERAL.bits()),
                choices: None,
                custom_id: None,
                title: None,
                components: None,
            }),
        }
    }

    /// Builds an ephemeral acknowledgement for a command that will respond later.
    pub fn deferred_ephemeral() -> Self {
        Self {
            kind: InteractionCallbackType::DeferredChannelMessageWithSource,
            data: Some(InteractionCallbackData {
                content: None,
                flags: Some(MessageFlags::EPHEMERAL.bits()),
                choices: None,
                custom_id: None,
                title: None,
                components: None,
            }),
        }
    }

    /// Builds an autocomplete response containing the supplied choices.
    ///
    /// # Arguments
    ///
    /// * `choices` - Values Discord presents for the focused command option.
    pub fn autocomplete(choices: Vec<ApplicationCommandOptionChoice>) -> Self {
        Self {
            kind: InteractionCallbackType::ApplicationCommandAutocompleteResult,
            data: Some(InteractionCallbackData {
                content: None,
                flags: None,
                choices: Some(choices),
                custom_id: None,
                title: None,
                components: None,
            }),
        }
    }

    /// Builds a modal containing the supplied text-input fields.
    ///
    /// # Arguments
    ///
    /// * `custom_id` - Identifier returned in the modal submission interaction.
    /// * `title` - User-visible modal title.
    /// * `components` - One to five text inputs displayed by Discord.
    pub fn modal(
        custom_id: impl Into<String>,
        title: impl Into<String>,
        components: Vec<ModalTextInput>,
    ) -> Self {
        Self {
            kind: InteractionCallbackType::Modal,
            data: Some(InteractionCallbackData {
                content: None,
                flags: None,
                choices: None,
                custom_id: Some(custom_id.into()),
                title: Some(title.into()),
                components: Some(
                    components
                        .into_iter()
                        .map(ModalActionRow::text_input)
                        .map(InteractionComponent::Modal)
                        .collect(),
                ),
            }),
        }
    }

    /// Builds an ephemeral response with a row of interactive buttons.
    pub fn ephemeral_buttons(content: impl Into<String>, buttons: Vec<Button>) -> Self {
        Self {
            kind: InteractionCallbackType::ChannelMessageWithSource,
            data: Some(InteractionCallbackData {
                content: Some(content.into()),
                flags: Some(MessageFlags::EPHEMERAL.bits()),
                choices: None,
                custom_id: None,
                title: None,
                components: Some(vec![InteractionComponent::Buttons(ButtonActionRow::new(
                    buttons,
                ))]),
            }),
        }
    }
}

/// Tests JSON payloads sent to Discord's interaction API.
#[cfg(test)]
mod tests {
    use super::{ApplicationCommandOptionChoice, Button, InteractionResponse, ModalTextInput};

    /// Confirms that ordinary messages are marked ephemeral.
    #[test]
    fn serializes_ephemeral_message() {
        let response = InteractionResponse::ephemeral("Saved");

        assert_eq!(
            serde_json::to_value(response).expect("response should serialize"),
            serde_json::json!({
                "type": 4,
                "data": { "content": "Saved", "flags": 64 }
            })
        );
    }

    /// Confirms that command deferrals preserve ephemeral visibility.
    #[test]
    fn serializes_ephemeral_deferral() {
        let response = InteractionResponse::deferred_ephemeral();

        assert_eq!(
            serde_json::to_value(response).expect("response should serialize"),
            serde_json::json!({ "type": 5, "data": { "flags": 64 } })
        );
    }

    /// Confirms that autocomplete responses contain Discord-compatible choices.
    #[test]
    fn serializes_autocomplete_choices() {
        let response = InteractionResponse::autocomplete(vec![ApplicationCommandOptionChoice {
            name: "Moderator".to_string(),
            value: "Moderator".to_string(),
        }]);

        assert_eq!(
            serde_json::to_value(response).expect("response should serialize"),
            serde_json::json!({
                "type": 8,
                "data": { "choices": [{ "name": "Moderator", "value": "Moderator" }] }
            })
        );
    }

    /// Confirms that a modal exposes its text fields using Discord's callback shape.
    #[test]
    fn serializes_modal_response() {
        let response = InteractionResponse::modal(
            "countdown-details",
            "Countdown details",
            vec![ModalTextInput::short("event-name", "Event name")],
        );

        assert_eq!(
            serde_json::to_value(response).expect("response should serialize"),
            serde_json::json!({
                "type": 9,
                "data": {
                    "custom_id": "countdown-details",
                    "title": "Countdown details",
                    "components": [{
                        "type": 1,
                        "components": [{
                            "type": 4,
                            "custom_id": "event-name",
                            "label": "Event name",
                            "style": 1,
                            "required": true
                        }]
                    }]
                }
            })
        );
    }

    /// Confirms that private responses can include a reusable button action row.
    #[test]
    fn serializes_ephemeral_buttons() {
        let response = InteractionResponse::ephemeral_buttons(
            "Review",
            vec![Button::primary("confirm", "Confirm"), Button::danger("cancel", "Cancel")],
        );

        assert_eq!(
            serde_json::to_value(response).expect("response should serialize"),
            serde_json::json!({
                "type": 4,
                "data": {
                    "content": "Review",
                    "flags": 64,
                    "components": [{
                        "type": 1,
                        "components": [
                            { "type": 2, "custom_id": "confirm", "label": "Confirm", "style": 1 },
                            { "type": 2, "custom_id": "cancel", "label": "Cancel", "style": 4 }
                        ]
                    }]
                }
            })
        );
    }
}
