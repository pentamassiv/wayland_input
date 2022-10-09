#[cfg(feature = "debug")]
use log::{debug, error, info, log_enabled, Level};

use wayland_input::DummyConnector;

fn main() {
    #[cfg(feature = "debug")]
    env_logger::init();

    println!("Start");
    let imput_service = wayland_input::InputService::new::<DummyConnector>(None);

    println!("Initalizesd");
    imput_service.sync_eventqueue();
    println!("Queue synced");

    // Toggle shift and long press Y
    let keycode = input_event_codes::KEY_Y!();
    let press = wayland_input::KeyState::Pressed;
    let release = wayland_input::KeyState::Released;
    let submission_result = imput_service.send_key(keycode, press);
    if submission_result.is_err() {
        println!("Error");
    };
    let submission_result = imput_service.send_key(keycode, release);
    if submission_result.is_err() {
        println!("Error");
    };
    println!("Entered keycode Y");

    imput_service.sync_eventqueue();
}
