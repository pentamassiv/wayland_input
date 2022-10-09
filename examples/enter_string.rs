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

    // Enter a string
    if imput_service
        .commit_string("Start typing".to_string())
        .is_err()
    {
        println!("Error commit_string");
    }
    imput_service.sync_eventqueue();
    if imput_service.commit().is_err() {
        println!("Error commit");
    };
    imput_service.sync_eventqueue();
    println!("Entered string");

    imput_service.sync_eventqueue();
}
