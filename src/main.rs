mod cli;
mod gui;

use gtk4::prelude::*;
use gtk4::Application;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // CLI dispatch — anything that isn't a bare invocation skips GTK init.
    if args.len() > 1 {
        if let Err(e) = cli::run(&args[1..]) {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
        return;
    }

    let app = Application::builder()
        .application_id("com.github.cde-wallpaper")
        .build();

    app.connect_activate(|app| {
        gui::build_window(app);
    });

    app.run();
}
