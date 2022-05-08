//! This crate provides an easy to use interface for the zwp_virtual_keyboard and the zwp_input_method_v2 protocols.
//! This could be used for virtual keyboards
//!
#[cfg(feature = "debug")]
#[macro_use]
extern crate log;

use std::convert::{AsRef, TryInto};
use std::io::{Seek, SeekFrom, Write};
use std::num::Wrapping;
use std::os::unix::io::IntoRawFd;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tempfile::tempfile;
use wayland_client::{protocol::wl_seat::WlSeat, EventQueue, Filter, Main};
use zwp_input_method::input_method_unstable_v2::zwp_input_method_manager_v2::ZwpInputMethodManagerV2;
use zwp_input_method::input_method_unstable_v2::zwp_input_method_v2::{
    Event as InputMethodEvent, ZwpInputMethodV2,
};
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

mod keymap;

mod wayland;

mod traits;
pub use traits::*;

pub type KeyCode = u32;

#[derive(Debug, Clone)]
/// Error when sending a request to the wayland-client
pub enum SubmitError {
    /// The wayland connection was dropped
    NotAlive,
    /// The imput_method protocol is unavailable
    IMNotAvailable,
    /// The virtual_keyboard protocol is unavailable
    VKNotAvailable,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyState {
    Pressed = 1,
    Released = 0,
}

// Mandatory conversion to apply filter to ZwpInputMethodV2
mod event_enum {
    use wayland_client::event_enum;
    use zwp_input_method::input_method_unstable_v2::zwp_input_method_v2::ZwpInputMethodV2;
    event_enum!(
        Events | InputMethod => ZwpInputMethodV2
    );
}

#[derive(Debug)]
/// Manages the pending state and the current state of the input method.
pub struct IMService {
    event_queue: Arc<Mutex<EventQueue>>,
    im: Option<(Main<ZwpInputMethodV2>, Arc<Mutex<Wrapping<u32>>>)>,
    vk: Option<(Main<ZwpVirtualKeyboardV1>, std::time::Instant)>,
}

impl IMService {
    pub fn new<C: IMConnector + 'static>(
        connector: Option<C>, //event_queue: EventQueue,
                              //seat: &WlSeat,
                              //im_mgr: Option<(Main<ZwpInputMethodManagerV2>, C)>,
                              //vk_mgr: Option<Main<ZwpVirtualKeyboardManagerV1>>,
    ) -> Self {
        let (event_queue, seat, im_mgr, vk_mgr) = wayland::init_wayland();
        let im = if let Ok(im_mgr) = im_mgr {
            #[cfg(feature = "debug")]
            info!("IM manager was availabe");
            if let Some(connector) = connector {
                Some(Self::new_im(&seat, im_mgr, connector))
            } else {
                Some(Self::new_im(&seat, im_mgr, DummyConnector::default()))
            }
        } else {
            #[cfg(feature = "debug")]
            info!("IM manager was NOT availabe");
            None
        };

        //let im = im_mgr.map(|(im_mgr, connector)| Self::new_im(&seat, im_mgr, connector));
        let vk = vk_mgr
            .map(|vk_mgr| {
                #[cfg(feature = "debug")]
                info!("VK manager was availabe");
                Self::new_vk(&seat, vk_mgr)
            })
            .ok();

        Self {
            event_queue: Arc::new(Mutex::new(event_queue)),
            im,
            vk,
        }
    }

    /// Creates a new IMServiceArc wrapped in Arc<Mutex<Self>>
    pub fn new_im<C: IMConnector + 'static>(
        seat: &WlSeat,
        im_manager: Main<ZwpInputMethodManagerV2>,
        connector: C,
    ) -> (Main<ZwpInputMethodV2>, Arc<Mutex<Wrapping<u32>>>) {
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

        let serial = Wrapping(0u32);
        #[cfg(feature = "debug")]
        info!("New IMService was created");
        // Return the wrapped IMServiceArc
        (im, Arc::new(Mutex::new(serial)))
    }

    /// Creates a new IMServiceArc wrapped in Arc<Mutex<Self>>
    pub fn new_vk(
        seat: &WlSeat,
        vk_manager: Main<ZwpVirtualKeyboardManagerV1>,
    ) -> (Main<ZwpVirtualKeyboardV1>, Instant) {
        let base_time = Instant::now();
        let vk = vk_manager.create_virtual_keyboard(&seat);
        let (keymap_raw_fd, keymap_size_u32) = Self::default_keymap();
        vk.keymap(1, keymap_raw_fd, keymap_size_u32);
        #[cfg(feature = "debug")]
        info!("New VKService was created");
        // Return the wrapped VKServiceArc
        (vk, base_time)
    }

    /// Creates the default keymap, memmory-maps it and returns the file descripter to it
    fn default_keymap() -> (i32, u32) {
        println!("Get default keymap and memory map it");
        let src = keymap::KEYMAP;
        let keymap_size = keymap::KEYMAP.len();
        let keymap_size_u32: u32 = keymap_size.try_into().unwrap(); // Convert it from usize to u32, panics if it is not possible
        let keymap_size_u64: u64 = keymap_size.try_into().unwrap(); // Convert it from usize to u64, panics if it is not possible
        let mut keymap_file = tempfile().expect("Unable to create tempfile");
        // Allocate space in the file first
        keymap_file.seek(SeekFrom::Start(keymap_size_u64)).unwrap();
        keymap_file.write_all(&[0]).unwrap();
        keymap_file.seek(SeekFrom::Start(0)).unwrap();
        let mut data = unsafe {
            memmap2::MmapOptions::new()
                .map_mut(&keymap_file)
                .expect("Could not access data from memory mapped file")
        };
        data[..src.len()].copy_from_slice(src.as_bytes());
        (keymap_file.into_raw_fd(), keymap_size_u32)
    }

    /// Sends a 'commit_string' request to the wayland-server
    ///
    /// INPUTS: text -> Text that will be committed
    /// Wayland messages have a maximum length so the length of the text must not exceed 4000 bytes
    pub fn commit_string(&self, text: String) -> Result<(), SubmitError> {
        #[cfg(feature = "debug")]
        info!("Commit_string method was called");
        if let Some((im, _)) = &self.im {
            // Check if proxy is still alive. If the proxy was dead, the requests would fail silently
            match im.as_ref().is_alive() {
                true => {
                    #[cfg(feature = "debug")]
                    info!("Commit string '{}'", text);
                    // Send the request to the wayland-server
                    im.commit_string(text);
                    Ok(())
                }
                false => {
                    #[cfg(feature = "debug")]
                    info!("Tried commit_string but it was not alive");
                    Err(SubmitError::NotAlive)
                }
            }
        } else {
            #[cfg(feature = "debug")]
            info!("Tried commit_string but it IM was not available");
            Err(SubmitError::IMNotAvailable)
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
        if let Some((im, _)) = &self.im {
            // Check if proxy is still alive. If the proxy was dead, the requests would fail silently
            match im.as_ref().is_alive() {
                true => {
                    // Send the delete_surrounding_text request to the wayland-server
                    im.delete_surrounding_text(before as u32, after as u32);
                    Ok(())
                }
                false => Err(SubmitError::NotAlive),
            }
        } else {
            Err(SubmitError::IMNotAvailable)
        }
    }

    /// Sends a 'commit' request to the wayland server
    ///
    /// This makes the pending changes permanent
    pub fn commit(&self) -> Result<(), SubmitError> {
        #[cfg(feature = "debug")]
        info!("Commit the changes");
        if let Some((im, serial)) = &self.im {
            // Check if proxy is still alive. If the proxy was dead, the requests would fail silently
            match im.as_ref().is_alive() {
                true => {
                    // Send request to wayland-server
                    im.commit(serial.lock().unwrap().0);
                    // Increase the serial
                    serial.lock().unwrap().0 += 1;
                    Ok(())
                }
                false => Err(SubmitError::NotAlive),
            }
        } else {
            Err(SubmitError::IMNotAvailable)
        }
    }

    pub fn make_unavailable(&self) -> Result<(), SubmitError> {
        #[cfg(feature = "debug")]
        info!("make_unavailable() was called");
        if let Some((im, _)) = &self.im {
            // Check if proxy is still alive. If the proxy was dead, the requests would fail silently
            match im.as_ref().is_alive() {
                true => {
                    // Send request to wayland-server
                    im.destroy();
                    Ok(())
                }
                false => Err(SubmitError::NotAlive),
            }
        } else {
            Err(SubmitError::IMNotAvailable)
        }
    }

    pub fn send_key(
        &self,
        keycode: KeyCode,
        desired_key_state: KeyState,
    ) -> Result<(), SubmitError> {
        if let Some((vk, base_time)) = &self.vk {
            let time = Self::elapsed_time_millis(base_time);
            #[cfg(feature = "debug")]
            info!("time: {}, keycode: {}", time, keycode);
            if vk.as_ref().is_alive() {
                vk.key(time, keycode, desired_key_state as u32);
                Ok(())
            } else {
                Err(SubmitError::NotAlive)
            }
        } else {
            Err(SubmitError::IMNotAvailable)
        }
    }

    pub fn modifiers(
        &self,
        mods_depressed: u32,
        mods_latched: u32,
        mods_locked: u32,
        group: u32,
    ) -> Result<(), SubmitError> {
        #[cfg(feature = "debug")]
        info!("Pressed modifiers: {}", mods_depressed);
        if let Some((vk, _)) = &self.vk {
            if vk.as_ref().is_alive() {
                vk.modifiers(mods_depressed, mods_latched, mods_locked, group);
                Ok(())
            } else {
                Err(SubmitError::NotAlive)
            }
        } else {
            Err(SubmitError::VKNotAvailable)
        }
    }

    fn elapsed_time_millis(base_time: &Instant) -> u32 {
        let duration = base_time.elapsed();
        let duration = duration.as_millis();
        if let Ok(duration) = duration.try_into() {
            duration
        } else {
            (duration % std::mem::size_of::<u32>() as u128)
                .try_into()
                .unwrap()
        }
    }

    pub fn sync_eventqueue(&self) {
        self.event_queue
            .lock()
            .unwrap()
            .sync_roundtrip(&mut (), |raw_event, _, _| {
                println!("Unhandled Event: {:?}", raw_event)
            })
            .unwrap();
    }
}

/*
pub fn toggle_shift(&mut self) -> Result<(), SubmitError> {
    // For the modifiers different codes have to be used. Use a bitmap to activate multiple modifiers at once
    let shift_flag = 0x1;
    let mods_depressed;
    let (_mods_latched, _mods_locked, group) = (0, 0, 0);

    match self.shift_state {
        KeyState::Pressed => {
            self.shift_state = KeyState::Released;
            mods_depressed = 0;
        }
        KeyState::Released => {
            self.shift_state = KeyState::Pressed;
            mods_depressed = shift_flag;
        }
    }
    if self.virtual_keyboard.as_ref().is_alive() {
        self.virtual_keyboard.modifiers(
            mods_depressed, //mods_depressed,
            _mods_latched,  //mods_latched
            _mods_locked,   //mods_locked
            group,          //group
        );
        Ok(())
    } else {
        Err(SubmitError::NotAlive)
    }
}
*/
