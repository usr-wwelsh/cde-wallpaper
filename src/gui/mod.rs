pub mod file_list;
pub mod palette;
pub mod preview;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, ColorDialogButton,
    DropDown, FileDialog, Label, Orientation, Separator, StringList,
};

use cde_wallpaper::config::Config;
use cde_wallpaper::kde::set_kde_wallpaper;
use cde_wallpaper::parser::{is_scale_file, parse_file, WallpaperData};
use cde_wallpaper::renderer::render;

use crate::gui::file_list::{build_file_list, populate_list, row_filename, select_by_name};
use crate::gui::palette::{build_palette, get_palette_colors, populate_flow, CDE_PALETTES};
use crate::gui::preview::{build_preview, update_preview};

pub struct AppState {
    pub config: Config,
    pub current_data: Option<WallpaperData>,
    pub current_name: Option<String>,
}

pub fn build_window(app: &Application) {
    let config = Config::load();

    let state = Rc::new(RefCell::new(AppState {
        current_data: None,
        current_name: config.selected_file.clone(),
        config,
    }));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("CDE Wallpaper")
        .default_width(900)
        .default_height(600)
        .build();

    // ── Right panel preview (built early for closure captures) ───────────────
    let picture = build_preview();

    // Shared update-preview callback
    let update_fn: Rc<dyn Fn()> = Rc::new({
        let state = Rc::clone(&state);
        let picture = picture.clone();
        move || update_preview(&picture, &state.borrow())
    });

    // ── Left panel ──────────────────────────────────────────────────────────
    let left = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .width_request(260)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(4)
        .build();

    let wallpaper_dir = state.borrow().config.wallpaper_dir.clone();
    let (scroll, file_list) = build_file_list(&wallpaper_dir);
    left.append(&scroll);
    left.append(&Separator::new(Orientation::Horizontal));

    // Palette dropdown
    let palette_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .margin_top(4)
        .build();
    palette_row.append(&Label::builder().label("Palette:").build());

    let names: Vec<&str> = CDE_PALETTES.iter().map(|(n, _)| *n).collect();
    let model = StringList::new(&names);
    let palette_dd = DropDown::new(Some(model), None::<gtk4::Expression>);
    palette_dd.set_selected(12); // "Default"
    palette_row.append(&palette_dd);
    left.append(&palette_row);

    // FG palette
    left.append(
        &Label::builder()
            .label("Foreground")
            .halign(gtk4::Align::Start)
            .build(),
    );
    let initial_colors = get_palette_colors("Default");
    let fg_flow = build_palette(initial_colors, Rc::clone(&state), true, Rc::clone(&update_fn));
    left.append(&fg_flow);

    // FG custom color button
    let cd_fg = ColorDialogButton::new(Some(
        gtk4::ColorDialog::builder().with_alpha(false).build(),
    ));
    {
        let s = state.borrow();
        let [r, g, b] = s.config.fg_color;
        cd_fg.set_rgba(&gtk4::gdk::RGBA::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0));
    }
    let update_cd_fg = Rc::clone(&update_fn);
    let state_cd_fg = Rc::clone(&state);
    cd_fg.connect_rgba_notify(move |btn| {
        let rgba = btn.rgba();
        state_cd_fg.borrow_mut().config.fg_color = [
            (rgba.red() * 255.0) as u8,
            (rgba.green() * 255.0) as u8,
            (rgba.blue() * 255.0) as u8,
        ];
        update_cd_fg();
    });
    left.append(&cd_fg);

    // BG palette
    left.append(
        &Label::builder()
            .label("Background")
            .halign(gtk4::Align::Start)
            .build(),
    );
    let bg_flow = build_palette(initial_colors, Rc::clone(&state), false, Rc::clone(&update_fn));
    left.append(&bg_flow);

    // BG custom color button
    let cd_bg = ColorDialogButton::new(Some(
        gtk4::ColorDialog::builder().with_alpha(false).build(),
    ));
    {
        let s = state.borrow();
        let [r, g, b] = s.config.bg_color;
        cd_bg.set_rgba(&gtk4::gdk::RGBA::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0));
    }
    let update_cd_bg = Rc::clone(&update_fn);
    let state_cd_bg = Rc::clone(&state);
    cd_bg.connect_rgba_notify(move |btn| {
        let rgba = btn.rgba();
        state_cd_bg.borrow_mut().config.bg_color = [
            (rgba.red() * 255.0) as u8,
            (rgba.green() * 255.0) as u8,
            (rgba.blue() * 255.0) as u8,
        ];
        update_cd_bg();
    });
    left.append(&cd_bg);

    // ── Palette dropdown signal ───────────────────────────────────────────────
    let fg_flow_dd = fg_flow.clone();
    let bg_flow_dd = bg_flow.clone();
    let state_dd = Rc::clone(&state);
    let update_dd = Rc::clone(&update_fn);
    palette_dd.connect_selected_notify(move |dd| {
        let idx = dd.selected() as usize;
        if idx < CDE_PALETTES.len() {
            let colors = CDE_PALETTES[idx].1;
            populate_flow(&fg_flow_dd, colors, Rc::clone(&state_dd), true,  Rc::clone(&update_dd));
            populate_flow(&bg_flow_dd, colors, Rc::clone(&state_dd), false, Rc::clone(&update_dd));
        }
    });

    // ── Right panel ──────────────────────────────────────────────────────────
    let right = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(4)
        .margin_end(8)
        .build();

    right.append(&picture);

    let bottom_bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let browse_btn = Button::with_label("Browse…");
    let apply_btn  = Button::with_label("Apply");
    bottom_bar.append(&browse_btn);
    bottom_bar.append(&apply_btn);
    right.append(&bottom_bar);

    // ── Root layout ──────────────────────────────────────────────────────────
    let root = GtkBox::builder().orientation(Orientation::Horizontal).build();
    root.append(&left);
    root.append(&right);
    window.set_child(Some(&root));

    // ── File list signal ─────────────────────────────────────────────────────
    let picture_list = picture.clone();
    let state_list   = Rc::clone(&state);
    file_list.connect_row_selected(move |_, row| {
        let Some(row)  = row else { return };
        let Some(name) = row_filename(row) else { return };
        let dir = state_list.borrow().config.wallpaper_dir.clone();
        let path = Path::new(&dir).join(&name);
        match parse_file(&path) {
            Ok(data) => {
                {
                    let mut s = state_list.borrow_mut();
                    s.current_data = Some(data);
                    s.current_name = Some(name);
                }
                update_preview(&picture_list, &state_list.borrow());
            }
            Err(e) => eprintln!("Error parsing {:?}: {}", path, e),
        }
    });

    // ── Browse button ─────────────────────────────────────────────────────────
    let state_browse     = Rc::clone(&state);
    let file_list_browse = file_list.clone();
    let window_browse    = window.clone();
    browse_btn.connect_clicked(move |_| {
        let dialog   = FileDialog::builder().title("Choose wallpaper directory").build();
        let state_cb = Rc::clone(&state_browse);
        let list_cb  = file_list_browse.clone();
        dialog.select_folder(Some(&window_browse), gtk4::gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let dir = path.to_string_lossy().to_string();
                    state_cb.borrow_mut().config.wallpaper_dir = dir.clone();
                    populate_list(&list_cb, &dir);
                }
            }
        });
    });

    // ── Apply button ─────────────────────────────────────────────────────────
    let state_apply  = Rc::clone(&state);
    let window_apply = window.clone();
    apply_btn.connect_clicked(move |_| {
        let (has_data, fg, bg, name_opt) = {
            let s = state_apply.borrow();
            (s.current_data.is_some(), s.config.fg_color, s.config.bg_color, s.current_name.clone())
        };
        if !has_data { return; }

        let (out_w, out_h) = get_screen_size(&window_apply);
        let img = {
            let s     = state_apply.borrow();
            let data  = s.current_data.as_ref().unwrap();
            let scale = is_scale_file(name_opt.as_deref().unwrap_or(""));
            render(data, fg, bg, out_w, out_h, scale)
        };

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);
        let tmp_path = format!("/tmp/cde-wallpaper-{}.png", ts);
        if let Err(e) = img.save(&tmp_path) { eprintln!("Failed to save PNG: {}", e); return; }
        if let Err(e) = set_kde_wallpaper(&tmp_path) { eprintln!("KDE DBus error: {}", e); }

        let mut s = state_apply.borrow_mut();
        s.config.selected_file = name_opt;
        s.config.save();
    });

    // ── Restore last selection ────────────────────────────────────────────────
    // Extract from state first so the Ref is dropped before any borrow_mut below
    let (restore_name, restore_dir) = {
        let s = state.borrow();
        (s.config.selected_file.clone(), s.config.wallpaper_dir.clone())
    };
    if let Some(name) = restore_name {
        let path = Path::new(&restore_dir).join(&name);
        if let Ok(data) = parse_file(&path) {
            {
                let mut s = state.borrow_mut();
                s.current_data = Some(data);
                s.current_name = Some(name.clone());
            }
            update_preview(&picture, &state.borrow());
        }
        select_by_name(&file_list, &name);
    }

    window.present();
}

fn get_screen_size(window: &ApplicationWindow) -> (u32, u32) {
    use gtk4::prelude::WidgetExt;
    let display = WidgetExt::display(window);
    if let Some(surface) = window.surface() {
        if let Some(monitor) = display.monitor_at_surface(&surface) {
            let g: gtk4::gdk::Rectangle = monitor.geometry();
            return (g.width() as u32, g.height() as u32);
        }
    }
    if let Some(obj) = display.monitors().item(0) {
        if let Ok(monitor) = obj.downcast::<gtk4::gdk::Monitor>() {
            let g: gtk4::gdk::Rectangle = monitor.geometry();
            return (g.width() as u32, g.height() as u32);
        }
    }
    (1920, 1080)
}
