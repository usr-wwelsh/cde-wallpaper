use anyhow::{bail, Context, Result};
use std::collections::HashMap;

pub struct XpmData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec<char>>,  // pixels[y][x] = symbol char
    pub colors: HashMap<char, [u8; 3]>,
}

pub fn parse(source: &str) -> Result<XpmData> {
    let strings = extract_quoted_strings(source);
    if strings.is_empty() {
        bail!("no quoted strings found in XPM");
    }

    // First string: header
    let header = &strings[0];
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() < 4 {
        bail!("XPM header too short: {:?}", header);
    }
    let width: u32 = parts[0].parse().context("parse width")?;
    let height: u32 = parts[1].parse().context("parse height")?;
    let ncolors: usize = parts[2].parse().context("parse ncolors")?;
    let cpp: usize = parts[3].parse().context("parse cpp")?;

    if cpp != 1 {
        bail!("only cpp=1 supported, got {}", cpp);
    }

    // Next ncolors strings: color entries
    if strings.len() < 1 + ncolors {
        bail!("not enough strings for colors: need {}, have {}", 1 + ncolors, strings.len());
    }

    let mut colors = HashMap::new();
    for i in 0..ncolors {
        let entry = &strings[1 + i];
        if entry.is_empty() {
            continue;
        }
        let symbol = entry.chars().next().unwrap();
        let rest = &entry[1..].trim().to_string();
        let color = parse_color_entry(rest);
        colors.insert(symbol, color);
    }

    // Remaining height strings: pixel rows
    let row_start = 1 + ncolors;
    if strings.len() < row_start + height as usize {
        bail!(
            "not enough pixel rows: need {}, have {} (total strings={})",
            height, strings.len() - row_start, strings.len()
        );
    }

    let mut pixels = Vec::with_capacity(height as usize);
    for row_idx in 0..height as usize {
        let row_str = &strings[row_start + row_idx];
        let row: Vec<char> = row_str.chars().collect();
        pixels.push(row);
    }

    Ok(XpmData { width, height, pixels, colors })
}

/// Extract all doubly-quoted string literals from the source.
/// Handles C escape sequences: \\ → \, \" → ", \n → newline, etc.
fn extract_quoted_strings(source: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            let mut s = String::new();
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 1;
                    match bytes[i] {
                        b'\\' => s.push('\\'),
                        b'"' => s.push('"'),
                        b'n' => s.push('\n'),
                        b't' => s.push('\t'),
                        b'r' => s.push('\r'),
                        other => {
                            s.push('\\');
                            s.push(other as char);
                        }
                    }
                } else {
                    s.push(bytes[i] as char);
                }
                i += 1;
            }
            if i < bytes.len() {
                i += 1; // closing "
            }
            strings.push(s);
        } else {
            i += 1;
        }
    }
    strings
}

/// Parse a color entry's key-value pairs and return the best RGB color.
///
/// Entry format (after the 1-char symbol): key value [key value ...]
/// Keys: s (symbolic), m (mono), c (color), g (grey), g4 (4-level grey)
/// Prefer c; fall back to m; fall back to symbolic name mapping.
fn parse_color_entry(rest: &str) -> [u8; 3] {
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let mut c_value: Option<&str> = None;
    let mut m_value: Option<&str> = None;
    let mut s_value: Option<&str> = None;

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "c" if i + 1 < tokens.len() => { c_value = Some(tokens[i + 1]); i += 2; }
            "m" if i + 1 < tokens.len() => { m_value = Some(tokens[i + 1]); i += 2; }
            "s" if i + 1 < tokens.len() => { s_value = Some(tokens[i + 1]); i += 2; }
            "g" | "g4" if i + 1 < tokens.len() => { i += 2; }
            _ => { i += 1; }
        }
    }

    if let Some(v) = c_value {
        return parse_color_value(v);
    }
    if let Some(v) = m_value {
        let parsed = parse_color_value(v);
        // If the mono value is meaningful (not a fallback grey), use it
        return parsed;
    }
    if let Some(sym) = s_value {
        return symbolic_color(sym);
    }
    [128, 128, 128]
}

fn symbolic_color(name: &str) -> [u8; 3] {
    match name {
        "topShadowColor" => [189, 189, 189],
        "background"     => [148, 148, 148],
        "selectColor"    => [115, 115, 115],
        "bottomShadowColor" => [99, 99, 99],
        _ => [128, 128, 128],
    }
}

pub fn parse_color_value(s: &str) -> [u8; 3] {
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            12 => {
                // #RRRRGGGGBBBB — take top byte of each 16-bit component
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
                let g = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
                let b = u8::from_str_radix(&hex[8..10], 16).unwrap_or(128);
                [r, g, b]
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
                [r, g, b]
            }
            _ => [128, 128, 128],
        }
    } else {
        match s.to_lowercase().as_str() {
            "black"   => [0, 0, 0],
            "white"   => [255, 255, 255],
            "red"     => [255, 0, 0],
            "green"   => [0, 128, 0],
            "blue"    => [0, 0, 255],
            "yellow"  => [255, 255, 0],
            "cyan"    => [0, 255, 255],
            "magenta" => [255, 0, 255],
            _         => [128, 128, 128],
        }
    }
}

impl XpmData {
    pub fn pixel_color(&self, x: u32, y: u32) -> [u8; 3] {
        if y as usize >= self.pixels.len() { return [128, 128, 128]; }
        let row = &self.pixels[y as usize];
        if x as usize >= row.len() { return [128, 128, 128]; }
        let sym = row[x as usize];
        self.colors.get(&sym).copied().unwrap_or([128, 128, 128])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concave() {
        let path = "/home/wwelsh/Documents/cdewallpapers/raw/Concave.pm";
        let src = std::fs::read_to_string(path).expect("read Concave.pm");
        let data = parse(&src).expect("parse Concave.pm");
        assert_eq!(data.width, 8);
        assert_eq!(data.height, 1024);
        assert_eq!(data.pixels.len(), 1024);
        assert_eq!(data.pixels[0].len(), 8);
    }

    #[test]
    fn test_skydark_colors_not_all_black() {
        let path = "/home/wwelsh/Documents/cdewallpapers/raw/SkyDark.pm";
        let src = std::fs::read_to_string(path).expect("read SkyDark.pm");
        let data = parse(&src).expect("parse SkyDark.pm");
        // Should have 4 distinct colors, none should be the default grey [128,128,128]
        let unique: std::collections::HashSet<[u8;3]> = data.colors.values().copied().collect();
        assert!(unique.len() >= 2, "SkyDark should have multiple colors, got {:?}", unique);
        // Check none are pure [128,128,128] fallback
        assert!(!unique.contains(&[128u8, 128u8, 128u8]),
            "SkyDark colors should not be fallback grey: {:?}", unique);
    }

    #[test]
    fn test_lattice() {
        let path = "/home/wwelsh/Documents/cdewallpapers/raw/Lattice.pm";
        let src = std::fs::read_to_string(path).expect("read Lattice.pm");
        let data = parse(&src).expect("parse Lattice.pm");
        assert!(data.width > 0);
        assert!(data.height > 0);
    }
}
