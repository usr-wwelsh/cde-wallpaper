use gtk4::prelude::*;
use gtk4::{Label, ListBox, ScrolledWindow, SelectionMode};
use std::path::Path;

use cde_wallpaper::parser::is_skip_file;

pub fn build_file_list(wallpaper_dir: &str) -> (ScrolledWindow, ListBox) {
    let list = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .build();

    populate_list(&list, wallpaper_dir);

    let scroll = ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .min_content_height(200)
        .build();

    (scroll, list)
}

pub fn populate_list(list: &ListBox, wallpaper_dir: &str) {
    // Remove all existing rows
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let dir = Path::new(wallpaper_dir);
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let ext = Path::new(&name)
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_lowercase();
            if (ext == "bm" || ext == "xbm" || ext == "pm" || ext == "xpm") && !is_skip_file(&name) {
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
        list.append(&label);
    }
}

/// Get the filename string from a ListBoxRow
pub fn row_filename(row: &gtk4::ListBoxRow) -> Option<String> {
    let label = row.child()?.downcast::<Label>().ok()?;
    Some(label.text().to_string())
}

/// Select the row matching the given filename
pub fn select_by_name(list: &ListBox, name: &str) {
    let mut i = 0;
    loop {
        let Some(row) = list.row_at_index(i) else { break };
        if row_filename(&row).as_deref() == Some(name) {
            list.select_row(Some(&row));
            break;
        }
        i += 1;
    }
}
