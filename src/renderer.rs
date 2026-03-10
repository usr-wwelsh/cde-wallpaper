use image::{imageops, RgbaImage, Rgba};
use crate::parser::{WallpaperData, XbmData, XpmData};

pub fn render(
    data: &WallpaperData,
    fg: [u8; 3],
    bg: [u8; 3],
    out_w: u32,
    out_h: u32,
    scale: bool,
) -> RgbaImage {
    match data {
        WallpaperData::Xbm(xbm) => render_xbm(xbm, fg, bg, out_w, out_h),
        WallpaperData::Xpm(xpm) => render_xpm(xpm, out_w, out_h, scale),
    }
}

fn render_xbm(xbm: &XbmData, fg: [u8; 3], bg: [u8; 3], out_w: u32, out_h: u32) -> RgbaImage {
    // First render the native-size image
    let mut src = RgbaImage::new(xbm.width, xbm.height);
    for y in 0..xbm.height {
        for x in 0..xbm.width {
            let color = if xbm.pixel(x, y) { fg } else { bg };
            src.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
        }
    }
    tile(&src, out_w, out_h)
}

fn render_xpm(xpm: &XpmData, out_w: u32, out_h: u32, scale: bool) -> RgbaImage {
    let mut src = RgbaImage::new(xpm.width, xpm.height);
    for y in 0..xpm.height {
        for x in 0..xpm.width {
            let c = xpm.pixel_color(x, y);
            src.put_pixel(x, y, Rgba([c[0], c[1], c[2], 255]));
        }
    }
    if scale {
        imageops::resize(&src, out_w, out_h, imageops::FilterType::Lanczos3)
    } else {
        tile(&src, out_w, out_h)
    }
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
