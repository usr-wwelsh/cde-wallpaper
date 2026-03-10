mod gui;

use gtk4::prelude::*;
use gtk4::Application;

fn main() {
    let app = Application::builder()
        .application_id("com.github.cde-wallpaper")
        .build();

    app.connect_activate(|app| {
        gui::build_window(app);
    });

    app.run();
}
