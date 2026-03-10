use anyhow::Result;
use zbus::blocking::Connection;

pub fn set_kde_wallpaper(path: &str) -> Result<()> {
    let script = format!(
        r#"var allDesktops = desktops();
for (i=0; i<allDesktops.length; i++) {{
    d = allDesktops[i];
    d.wallpaperPlugin = "org.kde.image";
    d.currentConfigGroup = Array("Wallpaper", "org.kde.image", "General");
    d.writeConfig("Image", "file://{}");
    d.writeConfig("FillMode", 2);
}}"#,
        path
    );

    let conn = Connection::session()?;
    conn.call_method(
        Some("org.kde.plasmashell"),
        "/PlasmaShell",
        Some("org.kde.PlasmaShell"),
        "evaluateScript",
        &(script,),
    )?;
    Ok(())
}
