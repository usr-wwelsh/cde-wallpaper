use std::path::Path;

fn main() -> anyhow::Result<()> {
    // BrickWall — XBM tiled
    {
        let path = Path::new("/home/wwelsh/Documents/cdewallpapers/raw/BrickWall.bm");
        let data = cde_wallpaper::parser::parse_file(path)?;
        let fg = [43u8, 80, 115];
        let bg = [148u8, 148, 148];
        let img = cde_wallpaper::renderer::render(&data, fg, bg, 800, 600, false);
        img.save("/tmp/brickwall_test.png")?;
        println!("Saved /tmp/brickwall_test.png");
    }

    // Ankh from nscdeWallpapers — verify symbolic color remapping
    {
        let path = Path::new("/home/wwelsh/Documents/nscdeWallpapers/Ankh.pm");
        let data = cde_wallpaper::parser::parse_file(path)?;

        // Render with grey bg (default-ish)
        let img_grey = cde_wallpaper::renderer::render(&data, [0u8,0,0], [148u8,148,148], 50, 50, false);
        // Render with bright red bg
        let img_red  = cde_wallpaper::renderer::render(&data, [0u8,0,0], [255u8,0,0],     50, 50, false);

        let p_grey = img_grey.get_pixel(0, 0);
        let p_red  = img_red.get_pixel(0, 0);
        println!("Ankh pixel(0,0) grey bg: {:?}", p_grey);
        println!("Ankh pixel(0,0) red  bg: {:?}", p_red);

        if p_grey != p_red {
            println!("SUCCESS: symbolic color remapping is working");
        } else {
            println!("FAIL: pixels are the same — symbolic colors not being remapped");

            // Debug: show symbolic_symbols
            if let cde_wallpaper::parser::WallpaperData::Xpm(ref xpm) = data {
                println!("  symbolic_symbols count: {}", xpm.symbolic_symbols.len());
                for (sym, name) in &xpm.symbolic_symbols {
                    println!("  {:?} -> {:?}", sym, name);
                }
                println!("  colors count: {}", xpm.colors.len());
            }
        }

        img_red.save("/tmp/ankh_red.png")?;
        println!("Saved /tmp/ankh_red.png");
    }

    Ok(())
}
