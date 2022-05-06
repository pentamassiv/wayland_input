use std::sync::{Arc, Mutex};
use wayland_client::{protocol::wl_seat::WlSeat, EventQueue, Main};

use std::convert::AsRef;
use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::io::IntoRawFd;
use std::time::Instant;
use tempfile::tempfile;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

use super::{KeyCode, KeyState, SubmitError};

/// Stores the state of the virtual keyboard
#[derive(Clone, Debug)]
struct VKModifierState {
    // Removed
}

impl Default for VKModifierState {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
/// Manages the pending state and the current state of the input method.
///
/// It is called IMServiceArc and not IMService because the new() method
/// wraps IMServiceArc and returns Arc<Mutex<IMServiceArc<T>>>. This is required because it's state could get changed by multiple threads.
/// One thread could handle requests while the other one handles events from the wayland-server
pub struct VKServiceArc {
    vk: Main<ZwpVirtualKeyboardV1>,
    modifiers: VKModifierState,
    //event_queue: EventQueue, // Preventing event_queue from being dropped
    base_time: std::time::Instant,
}

impl VKServiceArc {
    /// Creates a new IMServiceArc wrapped in Arc<Mutex<Self>>
    pub fn new(
        event_queue: EventQueue,
        seat: &WlSeat,
        vk_manager: Main<ZwpVirtualKeyboardManagerV1>,
    ) -> (EventQueue, Arc<Mutex<VKServiceArc>>) {
        let base_time = Instant::now();
        let modifiers = VKModifierState::default();

        // Get ZwpInputMethodV2 from ZwpInputMethodManagerV2
        let vk = vk_manager.create_virtual_keyboard(&seat);

        // Create VKServiceArc with default values
        let vk_service = VKServiceArc {
            vk,
            modifiers: VKModifierState::default(),

            base_time,
        };

        vk_service.init_virtual_keyboard();

        // Wrap VKServiceArc to allow mutability from multiple threads
        let vk_service = Arc::new(Mutex::new(vk_service));
        #[cfg(feature = "debug")]
        info!("New VKService was created");
        // Return the wrapped VKServiceArc
        (event_queue, vk_service)
    }

    fn init_virtual_keyboard(&self) {
        println!("keyboard initialized");
        let src = super::keymap::KEYMAP;
        let keymap_size = super::keymap::KEYMAP.len();
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
        let keymap_raw_fd = keymap_file.into_raw_fd();
        self.vk.keymap(1, keymap_raw_fd, keymap_size_u32);
    }

    fn get_duration(&mut self) -> u32 {
        let duration = self.base_time.elapsed();
        let duration = duration.as_millis();
        if let Ok(duration) = duration.try_into() {
            duration
        } else {
            // Reset the base time if it was too big for a u32
            self.base_time = Instant::now();
            self.get_duration()
        }
    }

    pub fn send_key(
        &mut self,
        keycode: KeyCode,
        desired_key_state: KeyState,
    ) -> Result<(), SubmitError> {
        let time = self.get_duration();
        #[cfg(feature = "debug")]
        info!("time: {}, keycode: {}", time, keycode);
        if self.vk.as_ref().is_alive() {
            self.vk.key(time, keycode, desired_key_state as u32);
            Ok(())
        } else {
            Err(SubmitError::NotAlive)
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
}
