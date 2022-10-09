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
    let submission_resulta = imput_service.commit_string("Start typing".to_string());
    imput_service.sync_eventqueue();
    let submission_resultb = imput_service.commit();
    imput_service.sync_eventqueue();
    if submission_resulta.is_err() && submission_resultb.is_err() {
        println!("Error");
    };
    println!("Entered string");

    imput_service.sync_eventqueue();
}
