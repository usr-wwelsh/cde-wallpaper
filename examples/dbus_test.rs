fn main() -> anyhow::Result<()> {
    let path = "/tmp/cde-wallpaper-current.png";
    println!("Setting KDE wallpaper to: {}", path);
    cde_wallpaper::kde::set_kde_wallpaper(path)?;
    println!("Success!");
    Ok(())
}
