use gtk::prelude::*;
use gtk::{gio, glib};
use std::convert::AsRef;
use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::io::IntoRawFd;
use std::time::{Duration, Instant};
use tempfile::tempfile;
use wayland_client::{protocol::wl_seat::WlSeat, EventQueue, Main};
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.entry-completion"),
        Default::default(),
    );
    application.connect_activate(build_ui);

    // When activated, shuts down the application
    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(
        glib::clone!(@weak application => move |_action, _parameter| {
            application.quit();
        }),
    );
    application.connect_startup(|application| {
        application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    });
    application.add_action(&quit);

    // Run the application
    application.run();
}

fn build_ui(application: &gtk::Application) {
    // create the main window
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("Entry with autocompletion");
    window.set_border_width(5);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(840, 480);

    // Create a title label
    let win_title = gtk::Label::new(None);
    win_title.set_markup("<big>Which country would you like to spend a holiday in?</big>");

    // Create an EntryCompletion widget
    let completion_countries = gtk::EntryCompletion::new();
    // Use the first (and only) column available to set the autocompletion text
    completion_countries.set_text_column(0);
    // how many keystrokes to wait before attempting to autocomplete?
    completion_countries.set_minimum_key_length(1);
    // whether the completions should be presented in a popup window
    completion_countries.set_popup_completion(true);

    // Create a ListStore of items
    // These will be the source for the autocompletion
    // as the user types into the field
    // For a more evolved example of ListStore see src/bin/list_store.rs
    let ls = create_list_model();
    completion_countries.set_model(Some(&ls));

    let input_field = gtk::Entry::new();
    input_field.set_completion(Some(&completion_countries));

    let row = gtk::Box::new(gtk::Orientation::Vertical, 5);
    row.add(&win_title);
    row.pack_start(&input_field, false, false, 10);

    let button = gtk::Button::with_label("Input text!");

    let (event_queue, seat, vk_mgr) = gdk_wayland::init_wayland();
    let mut vk_service = VKService::new(event_queue, &seat, vk_mgr.unwrap());
    let mut callback = move |_| {
        // Long press K
        let key = input_event_codes::KEY_K!();
        let submission_result = vk_service.long_press_keycode(key);
        if submission_result.is_err() {
            println!("Error: {:?}", submission_result);
        };
        println!("Long press done");

        // Toggle shift and long press E
        let key = input_event_codes::KEY_E!();
        let submission_result = vk_service.toggle_shift();
        if submission_result.is_err() {
            println!("Error: {:?}", submission_result);
        };
        let submission_result = vk_service.long_press_keycode(key);
        if submission_result.is_err() {
            println!("Error: {:?}", submission_result);
        };
        println!("First toggle shift and long press x");

        // Toggle shift and long press Y
        let key = input_event_codes::KEY_Y!();
        let submission_result = vk_service.toggle_shift();
        if submission_result.is_err() {
            println!("Error: {:?}", submission_result);
        };
        let submission_result = vk_service.long_press_keycode(key);
        if submission_result.is_err() {
            println!("Error: {:?}", submission_result);
        };
        println!("Second toggle shift and long press x");
    };

    button.connect_clicked(callback);

    // window.add(&win_title);
    window.add(&row);
    window.add(&button);

    // show everything
    window.show_all();
}

struct Data {
    description: String,
}

fn create_list_model() -> gtk::ListStore {
    let col_types: [glib::Type; 1] = [glib::Type::STRING];

    let data: [Data; 4] = [
        Data {
            description: "France".to_string(),
        },
        Data {
            description: "Italy".to_string(),
        },
        Data {
            description: "Sweden".to_string(),
        },
        Data {
            description: "Switzerland".to_string(),
        },
    ];
    let store = gtk::ListStore::new(&col_types);
    for d in data.iter() {
        let values: [(u32, &dyn ToValue); 1] = [(0, &d.description)];
        store.set(&store.append(), &values);
    }
    store
}
