pub mod file_list;
pub mod palette;
pub mod preview;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, ColorDialogButton,
    DropDown, FileDialog, Label, Orientation, Separator, StringList,
};

use cde_wallpaper::assets::DefaultWallpapers;
use cde_wallpaper::config::Config;
use cde_wallpaper::hyprland::set_hyprland_wallpaper;
use cde_wallpaper::kde::set_kde_wallpaper;
use cde_wallpaper::parser::{is_scale_file, parse_file, parse_str, WallpaperData};
use cde_wallpaper::renderer::render;

use crate::gui::file_list::{build_file_list, is_embedded_row, populate_list, row_filename, select_by_name};
use crate::gui::palette::{build_palette, get_palette_colors, populate_flow, CDE_PALETTES};
use crate::gui::preview::{build_preview, update_preview};

pub struct AppState {
    pub config: Config,
    pub current_data: Option<WallpaperData>,
    pub current_name: Option<String>,
    pub current_is_embedded: bool,
}

pub fn build_window(app: &Application) {
    let config = Config::load();

    let state = Rc::new(RefCell::new(AppState {
        current_data: None,
        current_name: config.selected_file.clone(),
        current_is_embedded: config.selected_is_embedded,
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
    let hide_defaults = state.borrow().config.hide_defaults;
    let (scroll, file_list) = build_file_list(wallpaper_dir.as_deref(), hide_defaults);
    left.append(&scroll);

    // "Hide defaults" checkbox
    let hide_chk = CheckButton::with_label("Hide defaults");
    hide_chk.set_active(hide_defaults);
    left.append(&hide_chk);

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

    // Swap FG/BG button
    let swap_btn = Button::with_label("⇅ Swap");
    left.append(&swap_btn);

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

    // ── Swap button signal ────────────────────────────────────────────────────
    {
        let state_swap = Rc::clone(&state);
        let cd_fg_swap = cd_fg.clone();
        let cd_bg_swap = cd_bg.clone();
        let update_swap = Rc::clone(&update_fn);
        swap_btn.connect_clicked(move |_| {
            let (new_fg, new_bg) = {
                let mut s = state_swap.borrow_mut();
                let tmp = s.config.fg_color;
                s.config.fg_color = s.config.bg_color;
                s.config.bg_color = tmp;
                (s.config.fg_color, s.config.bg_color)
            };
            let [r, g, b] = new_fg;
            cd_fg_swap.set_rgba(&gtk4::gdk::RGBA::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0));
            let [r, g, b] = new_bg;
            cd_bg_swap.set_rgba(&gtk4::gdk::RGBA::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0));
            update_swap();
        });
    }

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
    let browse_btn = Button::with_label("Add folder…");
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

        let data = if is_embedded_row(row) {
            let ext = Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            match DefaultWallpapers::get(&name) {
                Some(file) => match std::str::from_utf8(file.data.as_ref()) {
                    Ok(source) => parse_str(source, &ext),
                    Err(e) => { eprintln!("UTF-8 error for {}: {}", name, e); return; }
                },
                None => { eprintln!("Embedded file not found: {}", name); return; }
            }
        } else {
            let dir = state_list.borrow().config.wallpaper_dir.clone().unwrap_or_default();
            parse_file(&Path::new(&dir).join(&name))
        };

        let embedded = is_embedded_row(row);
        match data {
            Ok(data) => {
                {
                    let mut s = state_list.borrow_mut();
                    s.current_data = Some(data);
                    s.current_name = Some(name);
                    s.current_is_embedded = embedded;
                }
                update_preview(&picture_list, &state_list.borrow());
            }
            Err(e) => eprintln!("Error parsing {}: {}", name, e),
        }
    });

    // ── Hide defaults checkbox signal ─────────────────────────────────────────
    let state_chk = Rc::clone(&state);
    let list_chk  = file_list.clone();
    hide_chk.connect_toggled(move |chk| {
        let hide = chk.is_active();
        state_chk.borrow_mut().config.hide_defaults = hide;
        let dir = state_chk.borrow().config.wallpaper_dir.clone();
        populate_list(&list_chk, dir.as_deref(), hide);
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
                    state_cb.borrow_mut().config.wallpaper_dir = Some(dir.clone());
                    let hide = state_cb.borrow().config.hide_defaults;
                    populate_list(&list_cb, Some(&dir), hide);
                }
            }
        });
    });

    // ── Apply logic (shared by button and Enter key) ──────────────────────────
    let apply_fn: Rc<dyn Fn()> = Rc::new({
        let state  = Rc::clone(&state);
        let window = window.clone();
        move || {
            let (has_data, fg, bg, name_opt) = {
                let s = state.borrow();
                (s.current_data.is_some(), s.config.fg_color, s.config.bg_color, s.current_name.clone())
            };
            if !has_data { return; }

            let (out_w, out_h) = get_screen_size(&window);
            let img = {
                let s     = state.borrow();
                let data  = s.current_data.as_ref().unwrap();
                let scale = is_scale_file(name_opt.as_deref().unwrap_or(""));
                render(data, fg, bg, out_w, out_h, scale)
            };

            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            let out_dir = format!("{}/.local/share/cde-wallpaper", home);
            let _ = std::fs::create_dir_all(&out_dir);
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs()).unwrap_or(0);
            let tmp_path = format!("{}/wallpaper-{}.png", out_dir, ts);
            if let Ok(entries) = std::fs::read_dir(&out_dir) {
                for entry in entries.flatten() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
            if let Err(e) = img.save(&tmp_path) { eprintln!("Failed to save PNG: {}", e); return; }
            if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
                if let Err(e) = set_hyprland_wallpaper(&tmp_path) {
                    eprintln!("Hyprland wallpaper error: {}", e);
                }
            } else if let Err(e) = set_kde_wallpaper(&tmp_path) {
                eprintln!("KDE DBus error: {}", e);
            }

            let mut s = state.borrow_mut();
            s.config.selected_file = name_opt;
            s.config.selected_is_embedded = s.current_is_embedded;
            s.config.save();
        }
    });

    // ── Apply button ─────────────────────────────────────────────────────────
    let apply_fn_btn = Rc::clone(&apply_fn);
    apply_btn.connect_clicked(move |_| apply_fn_btn());

    // ── Enter key on file list applies the wallpaper ──────────────────────────
    let key_ctrl = gtk4::EventControllerKey::new();
    let apply_fn_key = Rc::clone(&apply_fn);
    key_ctrl.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Return || key == gtk4::gdk::Key::KP_Enter {
            apply_fn_key();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    file_list.add_controller(key_ctrl);

    // ── Restore last selection ────────────────────────────────────────────────
    let (restore_name, restore_dir, restore_embedded) = {
        let s = state.borrow();
        (
            s.config.selected_file.clone(),
            s.config.wallpaper_dir.clone(),
            s.config.selected_is_embedded,
        )
    };
    if let Some(name) = restore_name {
        let data = if restore_embedded {
            let ext = Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            DefaultWallpapers::get(&name)
                .and_then(|f| std::str::from_utf8(f.data.as_ref()).ok().map(|s| s.to_string()))
                .and_then(|src| parse_str(&src, &ext).ok())
        } else {
            let dir = restore_dir.unwrap_or_default();
            parse_file(&Path::new(&dir).join(&name)).ok()
        };

        if let Some(data) = data {
            {
                let mut s = state.borrow_mut();
                s.current_data = Some(data);
                s.current_name = Some(name.clone());
            }
            update_preview(&picture, &state.borrow());
        }
        select_by_name(&file_list, &name, restore_embedded);
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
