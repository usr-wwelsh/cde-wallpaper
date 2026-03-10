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

    // Concave — XPM scaled
    {
        let path = Path::new("/home/wwelsh/Documents/cdewallpapers/raw/Concave.pm");
        let data = cde_wallpaper::parser::parse_file(path)?;
        let fg = [43u8, 80, 115];
        let bg = [148u8, 148, 148];
        let img = cde_wallpaper::renderer::render(&data, fg, bg, 800, 600, true);
        img.save("/tmp/concave_test.png")?;
        println!("Saved /tmp/concave_test.png");
    }

    Ok(())
}
