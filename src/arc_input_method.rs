use std::cmp;
use std::num::Wrapping;
use std::sync::{Arc, Mutex};
use wayland_client::{protocol::wl_seat::WlSeat, Filter, Main};
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ChangeCause, ContentHint, ContentPurpose,
};
use zwp_input_method::input_method_unstable_v2::zwp_input_method_manager_v2::ZwpInputMethodManagerV2;
use zwp_input_method::input_method_unstable_v2::zwp_input_method_v2::{
    Event as InputMethodEvent, ZwpInputMethodV2,
};

use super::traits::IMConnector;
use super::SubmitError;

// Mandatory conversion to apply filter to ZwpInputMethodV2
mod event_enum {
    use wayland_client::event_enum;
    use zwp_input_method::input_method_unstable_v2::zwp_input_method_v2::ZwpInputMethodV2;
    event_enum!(
        Events | InputMethod => ZwpInputMethodV2
    );
}

/// Stores the state of the input method
#[derive(Clone, Debug)]
struct IMProtocolState {
    surrounding_text: String,
    cursor: usize,
    content_purpose: ContentPurpose,
    content_hint: ContentHint,
    text_change_cause: ChangeCause,
    active: bool,
}

impl Default for IMProtocolState {
    fn default() -> IMProtocolState {
        IMProtocolState {
            surrounding_text: String::new(),
            cursor: 0,
            content_hint: ContentHint::None,
            content_purpose: ContentPurpose::Normal,
            text_change_cause: ChangeCause::InputMethod,
            active: false,
        }
    }
}

#[derive(Clone, Debug)]
/// Manages the pending state and the current state of the input method.
///
/// It is called IMServiceArc and not IMService because the new() method
/// wraps IMServiceArc and returns Arc<Mutex<IMServiceArc<T>>>. This is required because it's state could get changed by multiple threads.
/// One thread could handle requests while the other one handles events from the wayland-server
pub struct IMServiceArc {
    im: Main<ZwpInputMethodV2>,
    serial: Wrapping<u32>,
}

impl IMServiceArc {
    /// Creates a new IMServiceArc wrapped in Arc<Mutex<Self>>
    pub fn new<C: IMConnector + 'static>(
        seat: &WlSeat,
        im_manager: Main<ZwpInputMethodManagerV2>,
        connector: C,
    ) -> Self {
        // Get ZwpInputMethodV2 from ZwpInputMethodManagerV2
        let im = im_manager.get_input_method(seat);

        // Assigns a filter to the wayland event queue to handle events for ZwpInputMethodV2
        let filter = Filter::new(move |event, _, _| match event {
            event_enum::Events::InputMethod { event, .. } => match event {
                InputMethodEvent::Activate => connector.activated(),
                InputMethodEvent::Deactivate => connector.deactivated(),
                InputMethodEvent::SurroundingText {
                    text,
                    cursor,
                    anchor,
                } => connector.surrounding_text(text, cursor as usize, anchor as usize),
                InputMethodEvent::TextChangeCause { cause } => connector.text_change_cause(cause),
                InputMethodEvent::ContentType { hint, purpose } => {
                    connector.content_type(hint, purpose)
                }
                InputMethodEvent::Done => connector.done(),
                InputMethodEvent::Unavailable => connector.unavailable(),
                _ => (),
            },
        });
        im.assign(filter);
        #[cfg(feature = "debug")]
        info!("The filter was assigned to Main<ZwpInputMethodV2>");

        // Create IMServiceArc with default values
        let im_service = IMServiceArc {
            im,
            serial: Wrapping(0u32),
        };

        #[cfg(feature = "debug")]
        info!("New IMService was created");
        // Return the wrapped IMServiceArc
        im_service
    }

    /// Sends a 'commit_string' request to the wayland-server
    ///
    /// INPUTS: text -> Text that will be committed
    /// Wayland messages have a maximum length so the length of the text must not exceed 4000 bytes
    pub fn commit_string(&self, text: String) -> Result<(), SubmitError> {
        #[cfg(feature = "debug")]
        info!("Commit string '{}'", text);
        // Check if proxy is still alive. If the proxy was dead, the requests would fail silently
        match self.im.as_ref().is_alive() {
            true => {
                // Send the request to the wayland-server
                self.im.commit_string(text);
                Ok(())
            }
            false => Err(SubmitError::NotAlive),
        }
    }

    /// Sends a 'delete_surrounding_text' request to the wayland server
    ///
    /// INPUTS:
    ///
    /// before -> number of chars to delete from the surrounding_text going left from the cursor
    ///
    /// after  -> number of chars to delete from the surrounding_text going right from the cursor
    pub fn delete_surrounding_text(&self, before: usize, after: usize) -> Result<(), SubmitError> {
        #[cfg(feature = "debug")]
        info!(
            "Send a request to the wayland server to delete {} chars before and {} after the cursor from the surrounding text",
            before, after
        );
        // Check if proxy is still alive. If the proxy was dead, the requests would fail silently
        match self.im.as_ref().is_alive() {
            true => {
                // Send the delete_surrounding_text request to the wayland-server
                self.im.delete_surrounding_text(before as u32, after as u32);
                Ok(())
            }
            false => Err(SubmitError::NotAlive),
        }
    }

    /// Sends a 'commit' request to the wayland server
    ///
    /// This makes the pending changes permanent
    pub fn commit(&mut self) -> Result<(), SubmitError> {
        #[cfg(feature = "debug")]
        info!("Commit the changes");
        // Check if proxy is still alive. If the proxy was dead, the requests would fail silently
        match self.im.as_ref().is_alive() {
            true => {
                // Send request to wayland-server
                self.im.commit(self.serial.0);
                // Increase the serial
                self.serial += Wrapping(1u32);
                Ok(())
            }
            false => Err(SubmitError::NotAlive),
        }
    }

    pub fn make_unavailable(&self) {
        #[cfg(feature = "debug")]
        info!("make_unavailable() was called");
        self.im.destroy();
    }
}
