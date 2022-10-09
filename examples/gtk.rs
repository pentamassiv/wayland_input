use gtk::prelude::*;
use gtk::{gio, glib};
use wayland_input::DummyConnector;

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

    let mut vk_service = wayland_input::InputService::new::<DummyConnector>(None);

    let mut callback = move |_| {
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
        let release = wayland_input::KeyState::Released;
        let submission_result = vk_service.send_key(keycode, press);
        if submission_result.is_err() {
            println!("Error: {:?}", submission_result);
        };
        //let submission_result = vk_service.send_key(keycode, release);
        //let submission_result = vk_service.send_key(keycode, release);
        if submission_result.is_err() {
            println!("Error: {:?}", submission_result);
        };
        println!("Second toggle shift and long press x");

        vk_service.sync_eventqueue();
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
