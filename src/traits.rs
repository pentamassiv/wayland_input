use super::SubmitError;
use wayland_client::{protocol::wl_seat::WlSeat, Main};
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};
use zwp_input_method::input_method_unstable_v2::zwp_input_method_manager_v2::ZwpInputMethodManagerV2;

/// Trait to get notified when the input method should be active or deactivated
///
/// If the user clicks for example on a text field, the method activate_im() is called
pub trait IMVisibility {
    fn activate_im(&self);
    fn deactivate_im(&self);
}

/// Trait to get notified when the text surrounding the cursor changes
pub trait ReceiveSurroundingText {
    fn text_changed(&self, string_left_of_cursor: String, string_right_of_cursor: String);
}

/// Trait to get notified when the hint or the purpose of the content changes
pub trait HintPurpose {
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose);
}
