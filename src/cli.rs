use anyhow::{anyhow, bail, Result};
use std::path::Path;
use std::process::Command;

use cde_wallpaper::assets::DefaultWallpapers;
use cde_wallpaper::config::Config;
use cde_wallpaper::hyprland::set_hyprland_wallpaper;
use cde_wallpaper::kde::set_kde_wallpaper;
use cde_wallpaper::parser::{is_scale_file, parse_file, parse_str, WallpaperData};
use cde_wallpaper::renderer::render;

pub fn run(args: &[String]) -> Result<()> {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");
    match cmd {
        "" | "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        "apply" => apply(&args[1..]),
        "show" => show(),
        other => bail!("unknown command: {other} (try `cde-wallpaper help`)"),
    }
}

fn print_help() {
    println!(
"cde-wallpaper — retro bitmap wallpaper picker

usage:
  cde-wallpaper                       launch GUI
  cde-wallpaper apply [opts]          render + set wallpaper headlessly
  cde-wallpaper show                  print active config
  cde-wallpaper help

apply options:
  --fg RRGGBB     override foreground color (hex, # optional)
  --bg RRGGBB     override background color
  --file NAME     wallpaper file (basename in wallpaper_dir, embedded name, or absolute path)
  --scale F       override scale factor (e.g. 1.5)
  --no-save       don't persist overrides to config.toml

without overrides, `apply` re-renders the saved selection at current screen size.");
}

fn show() -> Result<()> {
    let c = Config::load();
    println!("wallpaper_dir   = {:?}", c.wallpaper_dir);
    println!("selected_file   = {:?}", c.selected_file);
    println!("selected_embed  = {}", c.selected_is_embedded);
    println!("fg_color        = #{:02x}{:02x}{:02x}", c.fg_color[0], c.fg_color[1], c.fg_color[2]);
    println!("bg_color        = #{:02x}{:02x}{:02x}", c.bg_color[0], c.bg_color[1], c.bg_color[2]);
    println!("scale_factor    = {}", c.scale_factor);
    Ok(())
}

fn apply(args: &[String]) -> Result<()> {
    let mut config = Config::load();
    let mut file_override: Option<String> = None;
    let mut save = true;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--fg" => {
                i += 1;
                config.fg_color = parse_hex(args.get(i).ok_or_else(|| anyhow!("--fg needs value"))?)?;
            }
            "--bg" => {
                i += 1;
                config.bg_color = parse_hex(args.get(i).ok_or_else(|| anyhow!("--bg needs value"))?)?;
            }
            "--file" => {
                i += 1;
                file_override = Some(args.get(i).ok_or_else(|| anyhow!("--file needs value"))?.clone());
            }
            "--scale" => {
                i += 1;
                config.scale_factor = args.get(i).ok_or_else(|| anyhow!("--scale needs value"))?
                    .parse().map_err(|e| anyhow!("bad --scale: {e}"))?;
            }
            "--no-save" => save = false,
            other => bail!("unknown apply option: {other}"),
        }
        i += 1;
    }

    let (data, name, is_embedded) = load_wallpaper(&config, file_override.as_deref())?;
    let (out_w, out_h) = screen_size();
    let scale = is_scale_file(&name);
    let img = render(&data, config.fg_color, config.bg_color, out_w, out_h, scale, config.scale_factor);

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let out_dir = format!("{}/.local/share/cde-wallpaper", home);
    std::fs::create_dir_all(&out_dir)?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let tmp_path = format!("{}/wallpaper-{}.png", out_dir, ts);
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    img.save(&tmp_path)?;

    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        set_hyprland_wallpaper(&tmp_path)?;
    } else {
        set_kde_wallpaper(&tmp_path)?;
    }

    if save {
        config.selected_file = Some(name);
        config.selected_is_embedded = is_embedded;
        config.save();
    }
    Ok(())
}

fn parse_hex(s: &str) -> Result<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        bail!("hex color must be 6 chars (got {:?})", s);
    }
    Ok([
        u8::from_str_radix(&s[0..2], 16).map_err(|e| anyhow!("bad hex r: {e}"))?,
        u8::from_str_radix(&s[2..4], 16).map_err(|e| anyhow!("bad hex g: {e}"))?,
        u8::from_str_radix(&s[4..6], 16).map_err(|e| anyhow!("bad hex b: {e}"))?,
    ])
}

fn load_wallpaper(config: &Config, file_override: Option<&str>) -> Result<(WallpaperData, String, bool)> {
    let requested = file_override
        .map(|s| s.to_string())
        .or_else(|| config.selected_file.clone())
        .ok_or_else(|| anyhow!("no wallpaper selected (pass --file or set one in the GUI first)"))?;

    // Absolute / contains slash → treat as filesystem path.
    if requested.contains('/') {
        let p = Path::new(&requested);
        let data = parse_file(p)?;
        let basename = p.file_name().and_then(|n| n.to_str()).unwrap_or(&requested).to_string();
        return Ok((data, basename, false));
    }

    // Try wallpaper_dir/basename.
    if let Some(dir) = &config.wallpaper_dir {
        let p = Path::new(dir).join(&requested);
        if p.exists() {
            let data = parse_file(&p)?;
            return Ok((data, requested, false));
        }
    }

    // Fall back to embedded.
    if let Some(emb) = DefaultWallpapers::get(&requested) {
        let source = std::str::from_utf8(emb.data.as_ref())
            .map_err(|e| anyhow!("embedded wallpaper not utf-8: {e}"))?;
        let ext = Path::new(&requested)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("xbm");
        let data = parse_str(source, ext)?;
        return Ok((data, requested, true));
    }

    bail!("could not find wallpaper: {}", requested)
}

fn screen_size() -> (u32, u32) {
    if let Ok(out) = Command::new("hyprctl").arg("monitors").output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let t = line.trim();
                // line format: "1920x1080@60.00000 at 0x0"
                if let Some(at) = t.find('@') {
                    let dim = &t[..at];
                    if let Some((w, h)) = dim.split_once('x') {
                        if let (Ok(w), Ok(h)) = (w.trim().parse::<u32>(), h.trim().parse::<u32>()) {
                            return (w, h);
                        }
                    }
                }
            }
        }
    }
    (1920, 1080)
}
