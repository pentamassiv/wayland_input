#[cfg(feature = "debug")]
use log::{debug, error, info, log_enabled, Level};

use wayland_input::DummyConnector;

fn main() {
    #[cfg(feature = "debug")]
    env_logger::init();

    println!("Start");
    let vk_service = wayland_input::IMService::new::<DummyConnector>(None);

    // Enter a string
    let submission_resulta = vk_service.commit_string("Start typing".to_string());
    let submission_resultb = vk_service.commit();
    if submission_resulta.is_err() && submission_resultb.is_err() {
        println!("Error");
    };
    println!("Start typing");

    // Delete some text
    let submission_resulta = vk_service.delete_surrounding_text(6, 0);
    let submission_resultb = vk_service.commit();
    if submission_resulta.is_err() && submission_resultb.is_err() {
        println!("Error");
    };
    println!("Deleted some letters");

    // Toggle shift and long press Y
    let keycode = input_event_codes::KEY_Y!();
    let press = wayland_input::KeyState::Pressed;
    let release = wayland_input::KeyState::Released;
    let submission_result = vk_service.send_key(keycode, press);
    if submission_result.is_err() {
        println!("Error");
    };
    //let submission_result = vk_service.send_key(keycode, release);
    //let submission_result = vk_service.send_key(keycode, release);
    if submission_result.is_err() {
        println!("Error");
    };
    println!("Second toggle shift and long press x");

    vk_service.sync_eventqueue();
}
