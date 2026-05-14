# cde-wallpaper

![Demo](demo.png)

A Rust/GTK4 wallpaper picker for Wayland that reads authentic CDE (Common Desktop Environment) wallpaper files (`.xbm`, `.xpm`) and applies them via KDE Plasma's D-Bus API or Hyprland's `hyprctl`/`swww`/`awww`.

## Features

- Parses original CDE `.xbm` (X BitMap) and `.xpm` (X PixMap) wallpaper files
- Tiled rendering for bitmap patterns; scaled rendering for full-screen XPM images
- Built-in CDE color palettes with foreground/background color pickers
- Live preview before applying
- Sets wallpaper on KDE Plasma (D-Bus) or Hyprland (swww/awww)
- GUI + scriptable CLI in a single binary
- Config persisted to `~/.config/cde-wallpaper/config.toml`

## Requirements

- Wayland compositor (tested on KDE Plasma 6)
- GTK 4.12+
- D-Bus session (for KDE wallpaper apply)
- CDE wallpaper files (`.xbm` / `.xpm`) — default set included (see below)

## Dependencies

```toml
gtk4 = "0.9"       # GUI + GDK texture
zbus = "5"         # D-Bus (KDE PlasmaShell)
image = "0.25"     # PNG rendering
serde + toml       # Config serialization
anyhow             # Error handling
```

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

A default set of CDE wallpapers is embedded in the binary and shown on startup. Optionally click **Add folder…** to include your own wallpaper directory (files appear above the built-in defaults). Use the **Hide defaults** checkbox to hide the embedded set. Select a wallpaper, choose foreground/background colors from the CDE palette or a custom color picker, preview it, then click **Apply**.

## CLI

The same `cde-wallpaper` binary exposes a CLI for scripting — passing any argument skips GTK and runs headlessly. Invoke with no args to launch the GUI.

```
cde-wallpaper                       launch GUI
cde-wallpaper apply [opts]          render + set wallpaper headlessly
cde-wallpaper show                  print active config
cde-wallpaper help                  print usage
```

### `apply` options

| Flag | Description |
|---|---|
| `--file NAME` | Wallpaper to render. Resolves as: absolute/relative path (contains `/`), basename in your saved `wallpaper_dir`, then embedded asset name (e.g. `abicus.xbm`). |
| `--fg RRGGBB` | Foreground color (hex, `#` optional). |
| `--bg RRGGBB` | Background color. |
| `--scale F` | Scale factor for tiled bitmaps (e.g. `1.5`). |
| `--no-save` | Render and apply without persisting overrides to `config.toml`. |

Without any overrides, `apply` re-renders the saved selection at the current screen size — useful as a startup hook after resolution changes.

Screen size is detected via `hyprctl monitors` when on Hyprland; falls back to 1920×1080. The compositor target is chosen by `HYPRLAND_INSTANCE_SIGNATURE` (Hyprland) or KDE Plasma D-Bus otherwise.

### Examples

```bash
# Apply an embedded wallpaper with custom colors, persist selection
cde-wallpaper apply --file abicus.xbm --fg ff8800 --bg 220033

# One-shot preview without touching config
cde-wallpaper apply --file Block.xbm --no-save

# Apply a file by absolute path
cde-wallpaper apply --file ~/wallpapers/custom.xbm --scale 2

# Re-apply current selection (e.g. on login)
cde-wallpaper apply

# Inspect saved config
cde-wallpaper show
```

Exits non-zero on errors (unknown flags, bad hex, missing wallpaper) with a message on stderr.

## How It Works

1. **Parser** — reads `.xbm` (1-bit bitmap) and `.xpm` (indexed color) formats
2. **Renderer** — tiles bitmaps across the screen resolution; scales select XPM images (Concave, Convex, SkyDark, SkyLight) to fill
3. **GUI** — GTK4 window with file list, CDE palette swatches, live preview
4. **Apply** — renders to a temporary PNG, then calls KDE Plasma's `evaluateScript` D-Bus method to set it as the wallpaper

## CDE Palettes

The built-in palette list mirrors the original CDE palette set (Broica, Cactus, Default, Desert, EarthTones, Galactic, GrassyMeadow, Ivory, Maple, Monsoon, Ocean, Pumpkin, Sandstone, Slate, Spring, Sulphur, Sunshine, Tropical, Tundra, Wheat).

## Bundled Wallpapers

The default wallpaper files embedded in this binary (`.bm`, `.xbm`, `.pm`, `.xpm` files in `assets/wallpapers/`) originate from the **Common Desktop Environment (CDE)** and are licensed under the **Creative Commons Attribution-ShareAlike 3.0 (CC BY-SA 3.0)** license. These files were not created by the author of this project. Source: [CDE on SourceForge](https://sourceforge.net/projects/cdesktopenv/).

The color palettes are also derived from CDE and are not original works of this project's author.

## License

MIT — see [LICENSE](LICENSE)
