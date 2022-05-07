use super::SubmitError;
use wayland_client::{protocol::wl_seat::WlSeat, Main};
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::ChangeCause;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};
use zwp_input_method::input_method_unstable_v2::zwp_input_method_manager_v2::ZwpInputMethodManagerV2;

/// Trait to get notified when the input method should be active or deactivated
///
/// If the user clicks for example on a text field, the method activate_im() is called
pub trait IMConnector {
    fn activated(&self);
    fn deactivated(&self);
    fn surrounding_text(&self, text: String, cursor: usize, anchor: usize);
    fn text_change_cause(&self, change_cause: ChangeCause);
    fn content_type(&self, content_hint: ContentHint, content_purpose: ContentPurpose);
    fn done(&self);
    fn unavailable(&self);
}
