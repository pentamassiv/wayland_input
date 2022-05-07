//! This crate provides an easy to use interface for the zwp_virtual_keyboard and the zwp_input_method_v2 protocols.
//! This could be used for virtual keyboards
//!
#[cfg(feature = "debug")]
#[macro_use]
extern crate log;

use std::sync::{Arc, Mutex};
use wayland_client::{protocol::wl_seat::WlSeat, EventQueue, Main};
use zwp_input_method::input_method_unstable_v2::zwp_input_method_manager_v2::ZwpInputMethodManagerV2;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;

mod keymap;

mod traits;
pub use traits::*;

use arc_vk::*;
mod arc_vk;
use arc_input_method::*;
mod arc_input_method;

pub type KeyCode = u32;

#[derive(Debug, Clone)]
/// Error when sending a request to the wayland-client
pub enum SubmitError {
    /// The wayland connection was dropped
    NotAlive,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyState {
    Pressed = 1,
    Released = 0,
}

#[derive(Clone, Debug)]
/// Manages the pending state and the current state of the input method.
pub struct IMService {
    im_service_arc: IMServiceArc,
}

impl IMService {
    fn new<C: IMConnector + 'static>(
        seat: &WlSeat,
        im_manager: Main<ZwpInputMethodManagerV2>,
        connector: C,
    ) -> Self {
        let im_service_arc = IMServiceArc::new(seat, im_manager, connector);
        IMService { im_service_arc }
    }

    fn commit_string(&self, text: String) -> Result<(), SubmitError> {
        self.im_service_arc.commit_string(text)
    }

    fn delete_surrounding_text(&self, before: usize, after: usize) -> Result<(), SubmitError> {
        self.im_service_arc.delete_surrounding_text(before, after)
    }

    fn commit(&mut self) -> Result<(), SubmitError> {
        self.im_service_arc.commit()
    }
}

#[derive(Clone, Debug)]
/// Manages the pending state and the current state of the input method.
pub struct VKService {
    vk_service: VKServiceArc, // provides an easy to use interface by hiding the Arc<Mutex<>>
}

impl VKService {
    fn new(
        event_queue: EventQueue,
        seat: &WlSeat,
        vk_manager: Main<ZwpVirtualKeyboardManagerV1>,
    ) -> (EventQueue, Self) {
        let (event_queue, vk_service) = VKServiceArc::new(event_queue, seat, vk_manager);
        (event_queue, Self { vk_service })
    }

    fn send_key(&self, keycode: KeyCode, desired_key_state: KeyState) -> Result<(), SubmitError> {
        self.vk_service.send_key(keycode, desired_key_state)
    }
}

/*fn send_event(&mut self) {
    self.event_queue
        .sync_roundtrip(&mut (), |raw_event, _, _| {
            println!("Unhandled Event: {:?}", raw_event)
        })
        .unwrap();
}
*/
