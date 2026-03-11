use gtk4::prelude::*;
use gtk4::{Label, ListBox, ListBoxRow, ScrolledWindow, SelectionMode};
use std::path::Path;

use cde_wallpaper::assets::DefaultWallpapers;
use cde_wallpaper::parser::is_skip_file;

pub fn build_file_list(wallpaper_dir: Option<&str>, hide_defaults: bool) -> (ScrolledWindow, ListBox) {
    let list = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .build();

    populate_list(&list, wallpaper_dir, hide_defaults);

    let scroll = ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .min_content_height(200)
        .build();

    (scroll, list)
}

pub fn populate_list(list: &ListBox, wallpaper_dir: Option<&str>, hide_defaults: bool) {
    // Remove all existing rows
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    // User files
    if let Some(dir_str) = wallpaper_dir {
        let dir = Path::new(dir_str);
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut names: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let ext = Path::new(&name)
                        .extension()
                        .and_then(|x| x.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if (ext == "bm" || ext == "xbm" || ext == "pm" || ext == "xpm")
                        && !is_skip_file(&name)
                    {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect();
            names.sort();

            for name in names {
                let label = Label::builder()
                    .label(&name)
                    .halign(gtk4::Align::Start)
                    .margin_start(8)
                    .margin_top(4)
                    .margin_bottom(4)
                    .build();
                let row = ListBoxRow::new();
                row.set_child(Some(&label));
                row.set_widget_name("user");
                list.append(&row);
            }
        }
    }

    // Embedded defaults
    if !hide_defaults {
        // Separator header row (non-selectable)
        let sep_label = Label::builder()
            .label("── Defaults ──")
            .halign(gtk4::Align::Center)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        let sep_row = ListBoxRow::new();
        sep_row.set_child(Some(&sep_label));
        sep_row.set_selectable(false);
        sep_row.set_activatable(false);
        sep_row.set_widget_name("separator");
        list.append(&sep_row);

        let mut names: Vec<String> = DefaultWallpapers::iter()
            .map(|s| s.to_string())
            .filter(|name| {
                let ext = Path::new(name)
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                (ext == "bm" || ext == "xbm" || ext == "pm" || ext == "xpm")
                    && !is_skip_file(name)
            })
            .collect();
        names.sort();

        for name in names {
            let label = Label::builder()
                .label(&name)
                .halign(gtk4::Align::Start)
                .margin_start(8)
                .margin_top(4)
                .margin_bottom(4)
                .build();
            let row = ListBoxRow::new();
            row.set_child(Some(&label));
            row.set_widget_name("embedded");
            list.append(&row);
        }
    }
}

/// Get the filename string from a ListBoxRow
pub fn row_filename(row: &ListBoxRow) -> Option<String> {
    let label = row.child()?.downcast::<Label>().ok()?;
    Some(label.text().to_string())
}

pub fn is_embedded_row(row: &ListBoxRow) -> bool {
    row.widget_name() == "embedded"
}

/// Select the row matching the given filename and embedded flag
pub fn select_by_name(list: &ListBox, name: &str, prefer_embedded: bool) {
    let mut i = 0;
    loop {
        let Some(row) = list.row_at_index(i) else { break };
        if row_filename(&row).as_deref() == Some(name) {
            let embedded = is_embedded_row(&row);
            if embedded == prefer_embedded || !prefer_embedded {
                list.select_row(Some(&row));
                break;
            }
        }
        i += 1;
    }
}
