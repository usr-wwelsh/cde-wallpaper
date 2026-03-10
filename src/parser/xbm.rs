use anyhow::{bail, Context, Result};

pub struct XbmData {
    pub width: u32,
    pub height: u32,
    pub bits: Vec<u8>,
}

pub fn parse(source: &str) -> Result<XbmData> {
    // Strip block comments
    let source = strip_block_comments(source);

    let width = parse_define(&source, "_width")
        .context("missing _width define")?;
    let height = parse_define(&source, "_height")
        .context("missing _height define")?;

    let bits = parse_bits(&source).context("failed to parse bits array")?;

    let stride = (width + 7) / 8;
    let expected = stride * height;
    if bits.len() < expected as usize {
        bail!(
            "XBM bits too short: got {}, expected {} ({}x{} stride={})",
            bits.len(), expected, width, height, stride
        );
    }

    Ok(XbmData { width, height, bits })
}

fn strip_block_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next(); // consume '*'
            // skip until */
            loop {
                match chars.next() {
                    Some('*') if chars.peek() == Some(&'/') => {
                        chars.next();
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_define(source: &str, suffix: &str) -> Option<u32> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#define") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 && parts[1].ends_with(suffix) {
                return parts[2].parse().ok();
            }
        }
    }
    None
}

fn parse_bits(source: &str) -> Result<Vec<u8>> {
    // Find the array: static char/unsigned char name_bits[] = { ... };
    let brace_start = source.find('{').context("no opening brace")?;
    let brace_end = source.rfind('}').context("no closing brace")?;
    let inner = &source[brace_start + 1..brace_end];

    let mut bits = Vec::new();
    for token in inner.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let val = if let Some(hex) = token.strip_prefix("0x").or_else(|| token.strip_prefix("0X")) {
            u8::from_str_radix(hex, 16)?
        } else {
            token.parse::<u8>()?
        };
        bits.push(val);
    }
    Ok(bits)
}

impl XbmData {
    /// Get the pixel value at (x, y): true = FG, false = BG
    pub fn pixel(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let stride = (self.width + 7) / 8;
        let byte_idx = (y * stride + x / 8) as usize;
        let bit = x % 8;
        (self.bits[byte_idx] >> bit) & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brickwall() {
        let path = "/home/wwelsh/Documents/cdewallpapers/raw/BrickWall.bm";
        let src = std::fs::read_to_string(path).expect("read BrickWall.bm");
        let data = parse(&src).expect("parse BrickWall.bm");
        assert_eq!(data.width, 50, "BrickWall width");
        assert_eq!(data.height, 50, "BrickWall height");
        let stride = (50u32 + 7) / 8; // 7
        assert_eq!(data.bits.len(), (stride * 50) as usize, "BrickWall bits length");
    }

    #[test]
    fn test_ankh() {
        let path = "/home/wwelsh/Documents/cdewallpapers/raw/Ankh.bm";
        let src = std::fs::read_to_string(path).expect("read Ankh.bm");
        let data = parse(&src).expect("parse Ankh.bm");
        assert_eq!(data.width, 261, "Ankh width");
        assert_eq!(data.height, 127, "Ankh height");
        // stride = (261+7)/8 = 33 bytes per row
        let stride = (261u32 + 7) / 8;
        assert_eq!(stride, 33);
        assert!(data.bits.len() >= (stride * 127) as usize);
    }
}
