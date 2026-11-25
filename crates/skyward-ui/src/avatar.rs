//! The picture the phone's own system shows for its owner, read from the
//! user's photo and cut to the circle the pages draw it in.

use anyhow::{Context, Result, anyhow, ensure};
use std::path::Path;

/// A square picture, its pixels already cut to a circle: what lies outside the
/// circle is left transparent, so a page lays it over its card as it is.
#[derive(Clone, Debug)]
pub struct Avatar {
    /// The side of the square, in pixels.
    pub size: u32,
    /// `size * size` pixels of RGBA, row by row.
    pub rgba: Vec<u8>,
}

impl PartialEq for Avatar {
    /// Two of these are told apart without reading their pixels: a picture is
    /// read once, and the pages ask whether it has come, not what it shows.
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.rgba.len() == other.rgba.len() && self.rgba.as_ptr() == other.rgba.as_ptr()
    }
}

impl Avatar {
    /// Read the picture at `path` and cut it to a circle `size` pixels across.
    ///
    /// The phone keeps its owner's picture as a JPEG; the pages also take a
    /// PNG, since what a phone saves it as is not theirs to choose.
    pub fn read(path: &Path, size: u32) -> Result<Avatar> {
        ensure!(size > 0, "a picture of no width");
        let bytes = std::fs::read(path).with_context(|| format!("open {}", path.display()))?;
        let (pixels, w, h, channels) = match bytes.as_slice() {
            [0xFF, 0xD8, ..] => jpeg(&bytes, path)?,
            [0x89, b'P', b'N', b'G', ..] => png(&bytes, path)?,
            _ => anyhow::bail!("{} is neither a JPEG nor a PNG", path.display()),
        };
        let rgba = cut(&pixels, w, h, channels, size);
        Ok(Avatar { size, rgba })
    }
}

/// A PNG's pixels, normalized to eight bits a channel.
fn png(bytes: &[u8], path: &Path) -> Result<(Vec<u8>, u32, u32, usize)> {
    let _ = path;
    {
        let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        // The phone's own picture is whatever it was saved as; the pages want
        // RGBA and nothing else.
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder.read_info().with_context(|| format!("read {}", path.display()))?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).with_context(|| format!("read {}", path.display()))?;
        let (w, h) = (info.width, info.height);
        ensure!(w > 0 && h > 0, "{} is a picture of no size", path.display());
        let channels = match info.color_type {
            png::ColorType::Rgba => 4,
            png::ColorType::Rgb => 3,
            png::ColorType::GrayscaleAlpha => 2,
            png::ColorType::Grayscale => 1,
            other => anyhow::bail!("{} is a picture of {other:?}, which a page cannot show", path.display()),
        };
        Ok((buf, w, h, channels))
    }
}

/// A JPEG's pixels, as three channels.
fn jpeg(bytes: &[u8], path: &Path) -> Result<(Vec<u8>, u32, u32, usize)> {
    let mut decoder = zune_jpeg::JpegDecoder::new(bytes);
    // The decoder's own errors are not std ones, so they are said as they come.
    decoder.decode_headers().map_err(|e| anyhow!("read {}: {e:?}", path.display()))?;
    let (w, h) = decoder.dimensions().ok_or_else(|| anyhow!("{} is a picture of no size", path.display()))?;
    let mut pixels = decoder.decode().map_err(|e| anyhow!("read {}: {e:?}", path.display()))?;
    // A four-channel picture is cut down to three: a page wants a picture, not
    // the two bytes per pixel the phone's own saved alongside.
    if pixels.len() == w * h * 4 {
        let mut rgb = Vec::with_capacity(w * h * 3);
        for px in pixels.as_chunks::<4>().0 {
            rgb.extend_from_slice(&px[..3]);
        }
        pixels = rgb;
    }
    ensure!(pixels.len() == w * h * 3, "{} is a picture of {} bytes, which is not a size", path.display(), pixels.len());
    Ok((pixels, w as u32, h as u32, 3))
}

/// The picture scaled to `size` square, its corners taken off: each pixel of
/// the square is read from the one under it in the picture, and left clear when
/// it falls outside the circle.
fn cut(src: &[u8], w: u32, h: u32, channels: usize, size: u32) -> Vec<u8> {
    let mut out = vec![0u8; (size * size * 4) as usize];
    let radius = size as f32 / 2.0;
    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f32 + 0.5 - radius, y as f32 + 0.5 - radius);
            if dx * dx + dy * dy > radius * radius {
                continue;
            }
            // The picture is square in practice, but one that is not is read
            // across its own width and height.
            let sx = ((x as u64 * w as u64) / size as u64) as u32;
            let sy = ((y as u64 * h as u64) / size as u64) as u32;
            let at = ((sy.min(h - 1) * w + sx.min(w - 1)) as usize) * channels;
            let into = ((y * size + x) * 4) as usize;
            let (r, g, b, a) = match channels {
                4 => (src[at], src[at + 1], src[at + 2], src[at + 3]),
                3 => (src[at], src[at + 1], src[at + 2], 255),
                2 => (src[at], src[at], src[at], src[at + 1]),
                _ => (src[at], src[at], src[at], 255),
            };
            out[into] = r;
            out[into + 1] = g;
            out[into + 2] = b;
            out[into + 3] = a;
        }
    }
    out
}

/// The name the phone gives itself, out of the places its settings keep one.
///
/// The file is a compiled XML whose text lives in a shared string pool, so the
/// value is found through the element that carries it, not by the bytes that
/// follow the key; see `crate::secure`.
///
/// The settings' own "device name" is the model unless the owner changed it
/// (`Xiaomi 17` and the model are the same word), and a name that is only the
/// model is not a name: it says which phone this is, which the page already
/// says. The name the owner gave is in the Bluetooth name, which MIUI writes
/// when the name is set there. So the order is the settings' own name where it
/// was changed, then the Bluetooth name, then the settings' own name as a last
/// resort. Where nothing is kept at all, the caller falls back to the model.
pub fn device_name(global: &Path, secure: &Path, model: Option<&str>) -> Option<String> {
    let chosen = named(global, "device_name").filter(|name| Some(name.as_str()) != model);
    chosen
        .or_else(|| named(secure, "bluetooth_name"))
        .or_else(|| named(secure, "device_name"))
}

/// Whether `what` reads as something a phone is called.
///
/// The settings file holds more than the owner's name, and a value that is not
/// a name should not be shown as one: this phone once displayed
/// `com.xiaomi.bluetooth`, a package that sat beside the name in the file's
/// string pool. A value that is all lower case, digits, dots and underscores,
/// with a dot in it and no space, is a package rather than a name, and is
/// refused so the caller falls back to the model.
fn reads_as_a_name(what: &str) -> bool {
    if what.is_empty() || what.len() > 64 || what.chars().any(char::is_control) {
        return false;
    }
    let package = what.contains('.')
        && !what.contains(' ')
        && what.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_');
    !package
}

/// The value the settings file at `path` keeps for `key`.
///
/// A name with an emoji in it is not stored as text: the settings keep the
/// value as a type byte and then the value, and a name that does not fit the
/// plain type is kept as base64 of its UTF-16 big-endian bytes under type
/// `0x0a`, while plain ASCII and Chinese are kept as UTF-8 under `0x05`. So the
/// type says which, and guessing from the content does not: `MyPhoneName1` is
/// twelve letters and digits that would decode as base64 into nonsense.
fn named(path: &Path, key: &str) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let name = crate::secure::value(&bytes, key)?;
    let name = decode_name(&name);
    let name = name.trim().to_string();
    if reads_as_a_name(&name) { Some(name) } else { None }
}

/// The name `raw` holds, decoded the way the settings wrote it.
///
/// The pool hands the value back as text, so what is left to undo is the base64
/// layer: a value that is base64 of well-formed UTF-16 big-endian is that text,
/// and anything else is the name as it stands.
fn decode_name(raw: &str) -> String {
    unbase64(raw).and_then(|bytes| utf16be(&bytes)).unwrap_or_else(|| raw.to_string())
}

/// UTF-16 big-endian bytes as text: an odd length, or a lone surrogate, is not
/// a name.
fn utf16be(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(2) {
        return None;
    }
    let units: Vec<u16> = bytes.chunks(2).map(|p| u16::from_be_bytes([p[0], p[1]])).collect();
    String::from_utf16(&units).ok()
}

/// Standard base64, or nothing where `text` is not that.
///
/// Only text that decodes to UTF-16 big-endian the whole way is taken, so a
/// plain name that happens to be valid base64 is not decoded by mistake.
fn unbase64(text: &str) -> Option<Vec<u8>> {
    let s = text.as_bytes();
    if s.len() < 8 || !s.len().is_multiple_of(4) {
        return None;
    }
    let pad = s.iter().rev().take_while(|&&b| b == b'=').count();
    if pad > 2 || s[..s.len() - pad].contains(&b'=') {
        return None;
    }
    let digit = |b: u8| -> Option<u32> {
        Some(match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a') as u32 + 26,
            b'0'..=b'9' => (b - b'0') as u32 + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        })
    };
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    for group in s.chunks(4) {
        let mut bits = 0u32;
        let mut count = 0;
        for &b in group {
            if b == b'=' {
                break;
            }
            bits = (bits << 6) | digit(b)?;
            count += 1;
        }
        match count {
            4 => {
                out.push((bits >> 16) as u8);
                out.push((bits >> 8) as u8);
                out.push(bits as u8);
            }
            3 => {
                out.push((bits >> 10) as u8);
                out.push((bits >> 2) as u8);
            }
            2 => out.push((bits >> 4) as u8),
            _ => return None,
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_is_cut_to_a_circle_its_corners_left_clear() {
        // Four pixels, all of them white: the circle takes the middle of them.
        let src = vec![255u8; 4 * 4 * 4];
        let out = cut(&src, 4, 4, 4, 8);
        let at = |x: u32, y: u32| out[((y * 8 + x) * 4) as usize + 3];
        assert_eq!(at(0, 0), 0, "the corner is outside the circle");
        assert_eq!(at(7, 7), 0);
        assert_eq!(at(4, 4), 255, "the middle is in it");
        assert_eq!(at(4, 0), 255, "and so is the top of it");
        assert_eq!(at(0, 4), 255, "and its left");
    }

    /// A settings file as plain text XML, which the reader takes as well as the
    /// compiled form Android writes. Entries are given in the order the file
    /// keeps them, so a test can put a value anywhere in it.
    fn settings(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut out = String::from("<?xml version='1.0' encoding='utf-8' standalone='yes' ?>\n<map>\n");
        for (k, v) in entries {
            out.push_str(&format!("    <string name=\"{k}\">{v}</string>\n"));
        }
        out.push_str("</map>\n");
        out.into_bytes()
    }

    /// A settings file written to a temporary path, and that path.
    fn written(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let at = std::env::temp_dir().join(format!("skyward-avatar-{name}"));
        std::fs::write(&at, bytes).unwrap();
        at
    }

    #[test]
    fn the_settings_own_name_is_read_where_the_owner_changed_it() {
        // A phone whose owner renamed it: the settings' own name is the new one,
        // and it is not the model, so it is what the phone goes by.
        let global = written("global-renamed", &settings(&[("device_name", "Mari0us")]));
        let secure = written("secure-renamed", &settings(&[("bluetooth_name", "Mari0us")]));
        assert_eq!(device_name(&global, &secure, Some("Xiaomi 17")).as_deref(), Some("Mari0us"));
        let _ = std::fs::remove_file(&global);
        let _ = std::fs::remove_file(&secure);
    }

    #[test]
    fn a_settings_name_that_is_only_the_model_gives_way_to_the_bluetooth_one() {
        // The settings' own name defaults to the model, so one that matches it
        // word for word is the model left standing, not a name the owner chose:
        // the name the owner gave is the Bluetooth one.
        let global = written("global-model", &settings(&[("device_name", "Xiaomi 17")]));
        let secure = written("secure-model", &settings(&[("bluetooth_name", "Mari0us")]));
        assert_eq!(
            device_name(&global, &secure, Some("Xiaomi 17")).as_deref(),
            Some("Mari0us"),
            "the model left standing is not a name"
        );
        let _ = std::fs::remove_file(&global);
        let _ = std::fs::remove_file(&secure);
    }

    #[test]
    fn the_settings_own_name_is_read_where_the_bluetooth_one_is_not_kept() {
        let global = written("global-only", &settings(&[("device_name", "Xiaomi 17")]));
        let secure = written("secure-only", &settings(&[("device_name", "Xiaomi 17")]));
        assert_eq!(device_name(&global, &secure, Some("Xiaomi 17")).as_deref(), Some("Xiaomi 17"));
        let _ = std::fs::remove_file(&global);
        let _ = std::fs::remove_file(&secure);
    }

    #[test]
    fn the_model_is_compared_only_where_one_is_given() {
        // Nothing to compare against: the settings' name is taken as it stands.
        let global = written("global-nomodel", &settings(&[("device_name", "Xiaomi 17")]));
        let secure = written("secure-nomodel", &settings(&[("bluetooth_name", "Mari0us")]));
        assert_eq!(device_name(&global, &secure, None).as_deref(), Some("Xiaomi 17"));
        let _ = std::fs::remove_file(&global);
        let _ = std::fs::remove_file(&secure);
    }

    #[test]
    fn a_chinese_name_is_read_whole() {
        let global = written("global-chinese", &settings(&[("device_name", "碎屏飞牛")]));
        let secure = written("secure-chinese", &settings(&[("bluetooth_name", "碎屏飞牛")]));
        assert_eq!(device_name(&global, &secure, Some("Xiaomi 17")).as_deref(), Some("碎屏飞牛"));
        let _ = std::fs::remove_file(&global);
        let _ = std::fs::remove_file(&secure);
    }

    #[test]
    fn a_name_with_an_emoji_is_kept_as_base64_of_utf16() {
        // What the phone writes for a name it cannot keep as plain text:
        // `2D3eAG1Li9U=` is UTF-16 big-endian for the grinning face and the two
        // characters after it.
        let global = written("global-emoji", &settings(&[("device_name", "2D3eAG1Li9U=")]));
        let secure = written("secure-emoji", &settings(&[("bluetooth_name", "2D3eAG1Li9U=")]));
        assert_eq!(device_name(&global, &secure, Some("Xiaomi 17")).as_deref(), Some("😀测试"));
        let _ = std::fs::remove_file(&global);
        let _ = std::fs::remove_file(&secure);
    }

    #[test]
    fn a_plain_name_that_looks_like_base64_is_not_decoded() {
        // Twelve letters and digits decode as base64, but not into UTF-16 text:
        // guessing from the content would break this name.
        assert_eq!(decode_name("MyPhoneName1"), "MyPhoneName1");
        assert_eq!(decode_name("Mari0us"), "Mari0us");
        assert_eq!(decode_name("碎屏飞牛"), "碎屏飞牛");
        assert_eq!(decode_name("2D3eAG1Li9U="), "😀测试");
    }

    #[test]
    fn no_name_is_none() {
        let at = written("none", b"nothing here");
        let secure = written("none-secure", b"nothing here");
        assert_eq!(device_name(&at, &secure, Some("Xiaomi 17")), None);
        let _ = std::fs::remove_file(&at);
        let _ = std::fs::remove_file(&secure);
    }

    #[test]
    fn a_package_name_beside_the_key_is_not_read_as_the_name() {
        // What this phone showed: a package kept beside the name came out as
        // the phone's name. The value now comes from the element that holds it,
        // so the package is never looked at.
        let global = written("global-package", &settings(&[("device_name", "com.xiaomi.bluetooth")]));
        let secure = written("secure-package", &settings(&[("bluetooth_name", "com.xiaomi.bluetooth")]));
        assert_eq!(device_name(&global, &secure, Some("Xiaomi 17")), None, "a package is not what a phone is called");
        let _ = std::fs::remove_file(&global);
        let _ = std::fs::remove_file(&secure);
    }

    #[test]
    fn a_name_with_a_dot_and_a_space_is_still_a_name() {
        // Refusing every dotted value would lose names like `Mr. Smith`.
        assert!(reads_as_a_name("Mr. Smith"));
        assert!(reads_as_a_name("Xiaomi 17"));
        assert!(reads_as_a_name("碎屏飞牛"));
        assert!(!reads_as_a_name("com.xiaomi.bluetooth"));
        assert!(!reads_as_a_name("android.process.acore"));
        assert!(!reads_as_a_name(""));
        assert!(!reads_as_a_name(&"x".repeat(65)));
    }
}
