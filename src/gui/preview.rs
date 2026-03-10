use gtk4::prelude::*;
use gtk4::{ContentFit, Picture};

use crate::gui::AppState;
use cde_wallpaper::parser::is_scale_file;
use cde_wallpaper::renderer::{render, to_memory_texture};

pub fn update_preview(picture: &Picture, state: &AppState) {
    if let Some(data) = &state.current_data {
        let scale = is_scale_file(state.current_name.as_deref().unwrap_or(""));
        let img = render(data, state.config.fg_color, state.config.bg_color, 400, 225, scale);
        let texture = to_memory_texture(&img);
        picture.set_paintable(Some(&texture));
    } else {
        picture.set_paintable(gtk4::gdk::Paintable::NONE);
    }
}

pub fn build_preview() -> Picture {
    let picture = Picture::new();
    picture.set_can_shrink(true);
    picture.set_content_fit(ContentFit::Fill);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture
}
