use image::{imageops, RgbaImage, Rgba};
use crate::parser::{WallpaperData, XbmData, XpmData};

pub fn render(
    data: &WallpaperData,
    fg: [u8; 3],
    bg: [u8; 3],
    out_w: u32,
    out_h: u32,
    scale: bool,
    scale_factor: f32,
) -> RgbaImage {
    match data {
        WallpaperData::Xbm(xbm) => render_xbm(xbm, fg, bg, out_w, out_h, scale_factor),
        WallpaperData::Xpm(xpm) => render_xpm(xpm, fg, bg, out_w, out_h, scale, scale_factor),
    }
}

fn render_xbm(xbm: &XbmData, fg: [u8; 3], bg: [u8; 3], out_w: u32, out_h: u32, scale_factor: f32) -> RgbaImage {
    let mut src = RgbaImage::new(xbm.width, xbm.height);
    for y in 0..xbm.height {
        for x in 0..xbm.width {
            let color = if xbm.pixel(x, y) { fg } else { bg };
            src.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
        }
    }
    let src = apply_scale_factor(src, scale_factor, out_w, out_h);
    tile(&src, out_w, out_h)
}

fn apply_scale_factor(src: RgbaImage, factor: f32, max_w: u32, max_h: u32) -> RgbaImage {
    if (factor - 1.0).abs() < 0.001 {
        return src;
    }
    let new_w = ((src.width() as f32 * factor).round() as u32)
        .max(1)
        .min(max_w);
    let new_h = ((src.height() as f32 * factor).round() as u32)
        .max(1)
        .min(max_h);
    imageops::resize(&src, new_w, new_h, imageops::FilterType::Nearest)
}

fn render_xpm(xpm: &XpmData, fg: [u8; 3], bg: [u8; 3], out_w: u32, out_h: u32, scale: bool, scale_factor: f32) -> RgbaImage {
    // Remap symbolic (theme-driven) colors to fg/bg-derived shades
    let color_override: std::collections::HashMap<char, [u8; 3]> = xpm.symbolic_symbols.iter()
        .map(|(&sym, name)| (sym, symbolic_to_theme_color(name, fg, bg)))
        .collect();

    let mut src = RgbaImage::new(xpm.width, xpm.height);
    for y in 0..xpm.height {
        for x in 0..xpm.width {
            let c = if let Some(row) = xpm.pixels.get(y as usize) {
                if let Some(&sym) = row.get(x as usize) {
                    if let Some(&oc) = color_override.get(&sym) {
                        oc
                    } else {
                        xpm.colors.get(&sym).copied().unwrap_or([128, 128, 128])
                    }
                } else { [128, 128, 128] }
            } else { [128, 128, 128] };
            src.put_pixel(x, y, Rgba([c[0], c[1], c[2], 255]));
        }
    }
    let src = apply_scale_factor(src, scale_factor, out_w, out_h);
    if scale {
        imageops::resize(&src, out_w, out_h, imageops::FilterType::Lanczos3)
    } else {
        tile(&src, out_w, out_h)
    }
}

/// Map a CDE symbolic color name to a theme color derived from fg/bg.
fn symbolic_to_theme_color(name: &str, fg: [u8; 3], bg: [u8; 3]) -> [u8; 3] {
    match name {
        "background"        => bg,
        "foreground"        => fg,
        "topShadowColor"    => blend(bg, [255, 255, 255], 0.30),
        "selectColor"       => blend(bg, [0, 0, 0], 0.20),
        "bottomShadowColor" => blend(bg, [0, 0, 0], 0.35),
        _                   => bg,
    }
}

/// Blend `a` toward `b` by `t` (0.0 = all a, 1.0 = all b).
fn blend(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
    ]
}

fn tile(src: &RgbaImage, out_w: u32, out_h: u32) -> RgbaImage {
    let src_w = src.width();
    let src_h = src.height();
    let mut out = RgbaImage::new(out_w, out_h);
    for y in 0..out_h {
        for x in 0..out_w {
            let sx = x % src_w;
            let sy = y % src_h;
            out.put_pixel(x, y, *src.get_pixel(sx, sy));
        }
    }
    out
}

pub fn to_memory_texture(img: &RgbaImage) -> gtk4::gdk::MemoryTexture {
    let (w, h) = img.dimensions();
    gtk4::gdk::MemoryTexture::new(
        w as i32,
        h as i32,
        gtk4::gdk::MemoryFormat::R8g8b8a8,
        &gtk4::glib::Bytes::from(img.as_raw()),
        (w * 4) as usize,
    )
}
