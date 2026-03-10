pub mod xbm;
pub mod xpm;

use anyhow::{bail, Result};
use std::path::Path;

pub use xbm::XbmData;
pub use xpm::XpmData;

pub enum WallpaperData {
    Xbm(XbmData),
    Xpm(XpmData),
}

pub fn parse_file(path: &Path) -> Result<WallpaperData> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let source = std::fs::read_to_string(path)?;

    match ext.as_str() {
        "bm" | "xbm" => Ok(WallpaperData::Xbm(xbm::parse(&source)?)),
        "pm" | "xpm" => Ok(WallpaperData::Xpm(xpm::parse(&source)?)),
        _ => bail!("unknown extension: {}", ext),
    }
}

/// Files to skip in the browser (solid color utilities, not real wallpapers)
pub fn is_skip_file(name: &str) -> bool {
    matches!(name, "Background.bm" | "Foreground.bm")
}

/// Files that should be scaled to fill the screen rather than tiled
pub fn is_scale_file(name: &str) -> bool {
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name);
    matches!(stem, "Concave" | "Convex" | "SkyDark" | "SkyLight")
}
