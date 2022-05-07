use wayland_input::DummyConnector;

fn main() {
    println!("Start");
    let mut vk_service = wayland_input::IMService::new::<DummyConnector>(None);

    // Enter a string
    let submission_result = vk_service.commit_string("Start typing".to_string());
    let submission_result = vk_service.commit();
    if submission_result.is_err() {
        println!("Error: {:?}", submission_result);
    };
    println!("Start typing");

    // Delete some text
    let submission_result = vk_service.delete_surrounding_text(6, 0);
    let submission_result = vk_service.commit();
    if submission_result.is_err() {
        println!("Error: {:?}", submission_result);
    };
    println!("Deleted some letters");

    // Toggle shift and long press Y
    let keycode = input_event_codes::KEY_Y!();
    let press = wayland_input::KeyState::Pressed;
    let release = wayland_input::KeyState::Pressed;
    let submission_result = vk_service.send_key(keycode, press);
    if submission_result.is_err() {
        println!("Error: {:?}", submission_result);
    };
    let submission_result = vk_service.send_key(keycode, release);
    if submission_result.is_err() {
        println!("Error: {:?}", submission_result);
    };
    println!("Second toggle shift and long press x");

    vk_service.sync_eventqueue();
}
