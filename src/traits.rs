use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ChangeCause, ContentHint, ContentPurpose,
};

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

#[derive(Debug, Clone, Copy, Default)]
pub struct DummyConnector {}

impl IMConnector for DummyConnector {
    fn activated(&self) {}
    fn deactivated(&self) {}
    fn surrounding_text(&self, _: String, _: usize, _: usize) {}
    fn text_change_cause(&self, _: ChangeCause) {}
    fn content_type(&self, _: ContentHint, _: ContentPurpose) {}
    fn done(&self) {}
    fn unavailable(&self) {}
}
