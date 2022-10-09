#[cfg(feature = "debug")]
use log::{debug, error, info, log_enabled, Level};

use wayland_input::DummyConnector;

fn main() {
    #[cfg(feature = "debug")]
    env_logger::init();

    println!("Start");
    let imput_service = wayland_input::InputService::new::<DummyConnector>(None);

    // Enter a string
    let submission_resulta = imput_service.commit_string("Start typing".to_string());
    imput_service.sync_eventqueue();
    let submission_resultb = imput_service.commit();
    imput_service.sync_eventqueue();
    if submission_resulta.is_err() && submission_resultb.is_err() {
        println!("Error");
    };
    println!("Start typing");

    // Delete some text
    let submission_resulta = imput_service.delete_surrounding_text(6, 0);
    imput_service.sync_eventqueue();
    let submission_resultb = imput_service.commit();
    imput_service.sync_eventqueue();
    if submission_resulta.is_err() && submission_resultb.is_err() {
        println!("Error");
    };
    println!("Deleted some letters");

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
    println!("Second toggle shift and long press x");

    imput_service.sync_eventqueue();
}
