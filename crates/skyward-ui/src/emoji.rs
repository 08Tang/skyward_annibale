//! Colour emoji: two typefaces on the phone between them draw them.
//!
//! * `NotoColorEmoji.ttf` is a COLR/CPAL vector font: one glyph a layer, one
//!   colour a layer, and each layer with a transform, a gradient and a clip of
//!   its own. `ab_glyph` only rasterizes an outline into grey coverage, so it
//!   draws neither the colour nor the transform. Here each layer's outline is
//!   taken, coloured, and painted into a small picture, which the text drawing
//!   then lays down.
//! * `NotoColorEmojiFlags.ttf` is a CBDT bitmap font, and the flags live in it:
//!   two regional indicators are joined into one flag by a GSUB ligature, and
//!   either of them alone is only a box.
//!
//! Neither has to be in the ramdisk: the copy that comes with the installed
//! system is under `/mnt/system`. A font that cannot be read changes nothing,
//! and text draws as it did.

use ab_glyph_rasterizer::{point, Rasterizer};
use crate::canvas::Canvas;
use crate::color::Rgba;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Where the coloured typeface is looked for, in the order tried: the copy the
/// ramdisk carries, which is always there, then the installed system as the
/// recovery mounts it, and the other partitions a phone might keep it on.
const COLOR_FONTS: [&str; 6] = [
    "/skyward/NotoColorEmoji.ttf",
    "/mnt/system/fonts/NotoColorEmoji.ttf",
    "/mnt/system_ext/fonts/NotoColorEmoji.ttf",
    "/system/fonts/NotoColorEmoji.ttf",
    "/vendor/fonts/NotoColorEmoji.ttf",
    "/mnt/mi_ext/fonts/NotoColorEmoji.ttf",
];

/// The flags are a typeface of their own: a bitmap one, whose glyphs are pairs
/// of regional indicators joined by a ligature.
const FLAG_FONTS: [&str; 5] = [
    "/skyward/NotoColorEmojiFlags.ttf",
    "/mnt/system/fonts/NotoColorEmojiFlags.ttf",
    "/mnt/system_ext/fonts/NotoColorEmojiFlags.ttf",
    "/system/fonts/NotoColorEmojiFlags.ttf",
    "/vendor/fonts/NotoColorEmojiFlags.ttf",
];

/// The files the typefaces would be read from, for the log: whether the phone
/// has them at all is the first thing to know when a name comes out with a box
/// where its emoji should be.
pub fn found() -> Vec<(String, u64)> {
    COLOR_FONTS
        .iter()
        .chain(FLAG_FONTS.iter())
        .filter_map(|path| std::fs::metadata(path).ok().map(|m| (path.to_string(), m.len())))
        .collect()
}

/// One emoji as it is drawn: a small picture of its own size, and where it
/// stands against the pen position and the baseline.
pub struct Art {
    pub canvas: Canvas,
    /// How far the picture's top-left corner lies from the pen, across, in
    /// pixels.
    pub dx: i32,
    /// How far it lies from the baseline, down the screen, in pixels: positive
    /// is downwards.
    pub dy: i32,
    /// How wide the emoji stands, in pixels: one em, which is what it takes in
    /// the system's own drawing.
    pub advance: f32,
}

/// The typefaces are read the first time an emoji is to be drawn. A first
/// reading that finds nothing is tried again a little later: the pages begin
/// drawing text before the phone's partitions are mounted.
#[derive(Default)]
pub struct Slot {
    loaded: RefCell<Loaded>,
}

#[derive(Default)]
struct Loaded {
    /// When the files were last looked for, whether or not they were found.
    at: Option<Instant>,
    emoji: Option<Rc<Emoji>>,
}

/// How long to wait before looking again for typefaces that were not there.
const RETRY: Duration = Duration::from_secs(2);

impl Slot {
    /// The typefaces, read now if they have not been read yet.
    pub fn get(&self) -> Option<Rc<Emoji>> {
        let mut loaded = self.loaded.borrow_mut();
        if let Some(emoji) = &loaded.emoji {
            return Some(emoji.clone());
        }
        // Looked for and not found: do not read those megabytes again every
        // frame.
        if loaded.at.is_some_and(|at| at.elapsed() < RETRY) {
            return None;
        }
        loaded.at = Some(Instant::now());
        loaded.emoji = Emoji::read();
        loaded.emoji.clone()
    }

    /// The picture of `ch` at `px`, where a typeface draws one.
    pub fn art(&self, ch: char, px: f32) -> Option<Rc<Art>> {
        self.get()?.art(ch, px)
    }

    /// The flag two regional indicators spell.
    pub fn flag(&self, first: char, second: char, px: f32) -> Option<Rc<Art>> {
        self.get()?.flag(first, second, px)
    }

    /// Hold `typeface` already, for a test or a preview that has one at hand
    /// rather than at one of the places above.
    pub fn keep(&self, typeface: Rc<Emoji>) {
        let mut loaded = self.loaded.borrow_mut();
        loaded.at = Some(Instant::now());
        loaded.emoji = Some(typeface);
    }
}

/// Whether `ch` is one of the characters an emoji typeface draws: the
/// pictographic blocks. The UI typeface has no shape for them by design, so the
/// test that every text the pages write is drawable leaves them to this face.
///
/// The regional indicators (U+1F1E6 to U+1F1FF) count too: neither is a
/// picture alone, and a pair of them is a flag.
pub fn depicted(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1F000..=0x1FAFF
            | 0x1F1E6..=0x1F1FF
            | 0x2190..=0x21FF
            | 0x2300..=0x23FF
            | 0x25A0..=0x27BF
            | 0x2900..=0x297F
            | 0x2B00..=0x2BFF
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0xFE00..=0xFE0F
            | 0x200D
            | 0x20E3
    )
}

/// A regional indicator: one of the two letters a flag is written with.
pub fn regional(ch: char) -> bool {
    matches!(ch as u32, 0x1F1E6..=0x1F1FF)
}

/// The emoji typefaces the phone has, and the pictures they have given already.
pub struct Emoji {
    fonts: Vec<Font>,
}

struct Font {
    /// The file's bytes, kept for the life of the process: the parsers borrow
    /// them, and a page may ask for a picture at any time.
    data: &'static [u8],
    /// A bitmap font (CBDT): the flags' typeface is one, and its glyphs are
    /// PNGs.
    bitmap: bool,
    /// Each picture drawn, by character (or ligature glyph) and em size.
    cache: RefCell<HashMap<(u32, u16), Option<Rc<Art>>>>,
}

impl Emoji {
    fn read() -> Option<Rc<Emoji>> {
        let mut fonts = Vec::new();
        if let Some(font) = COLOR_FONTS.iter().find_map(|path| Font::read(Path::new(path))) {
            fonts.push(font);
        }
        if let Some(font) = FLAG_FONTS.iter().find_map(|path| Font::read(Path::new(path))) {
            fonts.push(font);
        }
        (!fonts.is_empty()).then(|| {
            Rc::new(Emoji { fonts })
        })
    }

    /// The typeface in the file at `path`, where it draws colours at all.
    pub fn read_at(path: &Path) -> Option<Rc<Emoji>> {
        Font::read(path).map(|font| Rc::new(Emoji { fonts: vec![font] }))
    }

    /// The typefaces these files hold, for a preview that has them at hand.
    pub fn read_all(paths: &[&Path]) -> Option<Rc<Emoji>> {
        let fonts: Vec<Font> = paths.iter().filter_map(|p| Font::read(p)).collect();
        (!fonts.is_empty()).then(|| Rc::new(Emoji { fonts }))
    }

    pub fn art(&self, ch: char, px: f32) -> Option<Rc<Art>> {
        self.fonts.iter().find_map(|font| font.art(ch, px))
    }

    /// The flag two regional indicators spell: the font joins them into one
    /// glyph through a GSUB ligature.
    pub fn flag(&self, first: char, second: char, px: f32) -> Option<Rc<Art>> {
        for font in &self.fonts {
            let Ok(face) = ttf_parser::Face::parse(font.data, 0) else { continue };
            let (Some(a), Some(b)) = (face.glyph_index(first), face.glyph_index(second)) else { continue };
            let Some(glyph) = ligature(&face, a.0, b.0) else { continue };
            if let Some(art) = font.art_of(&face, glyph, px) {
                return Some(art);
            }
        }
        None
    }
}

/// The em size rounded to whole pixels, so that one size is drawn once for the
/// whole page.
fn ppem_of(px: f32) -> u16 {
    px.round().clamp(6.0, 128.0) as u16
}

impl Font {
    fn read(path: &Path) -> Option<Font> {
        let bytes = std::fs::read(path).ok()?;
        let data: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        let face = ttf_parser::Face::parse(data, 0).ok()?;
        let tables = face.tables();
        let bitmap = tables.colr.is_none();
        // A bitmap glyph hides in one of four tables: CBDT (which this flags'
        // typeface uses), EBDT, bdat and sbix.
        if bitmap && tables.cbdt.is_none() && tables.ebdt.is_none() && tables.bdat.is_none() && tables.sbix.is_none() {
            return None;
        }
        Some(Font { data, bitmap, cache: RefCell::new(HashMap::new()) })
    }

    fn art(&self, ch: char, px: f32) -> Option<Rc<Art>> {
        // This typeface also has a space, `#`, `*` and the digits, which the
        // keycaps are built on -- but those characters belong to the UI face:
        // drawing them from here made the digits thin and then gone
        // (2026-11-24).
        if !depicted(ch) {
            return None;
        }
        let face = ttf_parser::Face::parse(self.data, 0).ok()?;
        let glyph = face.glyph_index(ch)?;
        self.art_of(&face, glyph.0, px)
    }

    fn art_of(&self, face: &ttf_parser::Face, glyph: u16, px: f32) -> Option<Rc<Art>> {
        let ppem = ppem_of(px);
        // A ligature glyph and a character share one cache: the former is
        // numbered from high up, so it cannot collide with a code point.
        let key = (glyph as u32, ppem);
        if let Some(known) = self.cache.borrow().get(&key) {
            return known.clone();
        }
        let art = match self.bitmap {
            true => self.bitmap_art(face, glyph, ppem),
            false => self.color_art(face, glyph, ppem),
        }
        .map(Rc::new);
        self.cache.borrow_mut().insert(key, art.clone());
        art
    }

    /// The vector one: each layer's outline coloured from the palette, put
    /// where its transform says, and painted into a small picture.
    fn color_art(&self, face: &ttf_parser::Face, glyph: u16, ppem: u16) -> Option<Art> {
        let colr = face.tables().colr?;
        let mut painter = Layers::default();
        // Palette 0 is the font's own default colouring; the foreground
        // colour is only for layers that name no palette.
        let foreground = ttf_parser::RgbaColor { red: 0, green: 0, blue: 0, alpha: 255 };
        colr.paint(ttf_parser::GlyphId(glyph), 0, &mut painter, &[], foreground)?;
        if painter.shapes.is_empty() {
            return None;
        }
        let upem = face.units_per_em() as f32;
        // Drawn at the glyph's own size, one em. The picture is a little wider
        // than an em: the advance follows the picture's width, so it never
        // crowds the letter beside it. The picture is 1.2 em tall with the
        // baseline left below the middle, so that what reaches under the
        // baseline -- the peach's base, the whale's spout -- is not cut off
        // (the proportion settled on in 0.0.78).
        let scale = ppem as f32 / upem;
        let (fw, fh) = (ppem as f32 * 1.05, ppem as f32 * 1.2);
        let (ox, oy) = (ppem as f32 * 0.02, ppem as f32);
        let mut parts: Vec<(Vec<Seg>, Rgba, Option<(f32, f32, f32, f32)>)> = Vec::new();
        for shape in &painter.shapes {
            let m = shape.transform;
            // A transform read as broken -- NaN or infinite -- would only
            // overflow the rasterizer's whole-number conversion: skip it.
            if ![m.a, m.b, m.c, m.d, m.e, m.f].iter().all(|v| v.is_finite()) {
                continue;
            }
            let mut collect = Collect::default();
            if face.outline_glyph(ttf_parser::GlyphId(shape.glyph), &mut collect).is_none() {
                continue;
            }
            // Some layers' transforms say a scale and a rotation but no
            // translation: the font's own COLR rotates about the glyph's
            // centre, and without that centre the highlight is carried off the
            // fruit. Such a layer is given the translation about its own
            // centre; a layer that does carry one is put where it says.
            let (mut e, mut f) = (m.e, m.f);
            let turned = (m.a - 1.0).abs() > 0.01 || m.b.abs() > 0.01 || m.c.abs() > 0.01 || (m.d - 1.0).abs() > 0.01;
            if turned && e.abs() < 0.01 && f.abs() < 0.01 {
                let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                for (x, y) in collect.segs.iter().flat_map(|s| s.points()) {
                    x0 = x0.min(x);
                    x1 = x1.max(x);
                    y0 = y0.min(y);
                    y1 = y1.max(y);
                }
                let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                e = cx - (m.a * cx + m.c * cy);
                f = cy - (m.b * cx + m.d * cy);
            }
            let mut segs = Vec::with_capacity(collect.segs.len());
            for seg in collect.segs {
                let seg = seg.map(|x, y| {
                    let (x, y) = (m.a * x + m.c * y + e, m.b * x + m.d * y + f);
                    match shape.clip {
                        Some((a, b, c, d)) => (x.clamp(a, c), y.clamp(b, d)),
                        None => (x, y),
                    }
                });
                segs.push(seg);
            }
            parts.push((segs, shape.color, shape.clip));
        }
        if parts.is_empty() {
            return None;
        }
        let (left, bottom) = (0.0f32, 0.0f32);
        let (w, h) = (fw.ceil() as u32, fh.ceil() as u32);
        let mut canvas = Canvas::new(w, h);
        let (wf, hf) = (w as f32, h as f32);
        // The rasterizer draws only inside the picture: a point has to fall
        // within 0..width, and one outside is pushed to the edge (it would be
        // clipped anyway). A little past the edge overflows its index.
        let fix = |v: f32, hi: f32| if v.is_finite() { v.clamp(0.0, hi) } else { 0.0 };
        for (segs, color, clip) in parts {
            let clip_px = clip.map(|(x0, y0, x1, y1)| {
                let a = (ox + x0 * scale, oy - y1 * scale);
                let b = (ox + x1 * scale, oy - y0 * scale);
                (a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1))
            });
            let mut r = Rasterizer::new(w as usize, h as usize);
            let (mut cursor, mut start) = ((0.0f32, 0.0f32), (0.0f32, 0.0f32));
            let unit = |x: f32, y: f32| (fix(ox + x * scale, wf), fix(oy - y * scale, hf));
            for seg in segs {
                match seg {
                    Seg::Move(x, y) => {
                        cursor = unit(x, y);
                        start = cursor;
                    }
                    Seg::Line(x, y) => {
                        let to = unit(x, y);
                        // A "line" between one point and itself gives the
                        // rasterizer an infinite slope: skip it.
                        if (to.0 - cursor.0).abs() > 0.01 || (to.1 - cursor.1).abs() > 0.01 {
                            r.draw_line(point(cursor.0, cursor.1), point(to.0, to.1));
                        }
                        cursor = to;
                    }
                    Seg::Quad(x1, y1, x, y) => {
                        let (c, to) = (unit(x1, y1), unit(x, y));
                        let on = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01;
                        if on(c, cursor) || on(c, to) {
                            if !on(cursor, to) {
                                r.draw_line(point(cursor.0, cursor.1), point(to.0, to.1));
                            }
                        } else {
                            r.draw_quad(point(cursor.0, cursor.1), point(c.0, c.1), point(to.0, to.1));
                        }
                        cursor = to;
                    }
                    Seg::Cubic(x1, y1, x2, y2, x, y) => {
                        let (c1, c2, to) = (unit(x1, y1), unit(x2, y2), unit(x, y));
                        let on = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01;
                        // A cubic whose control point meets an end has
                        // degenerated to a line, and worked as a curve it
                        // divides by zero.
                        if on(c1, cursor) || on(c2, to) {
                            if !on(cursor, to) {
                                r.draw_line(point(cursor.0, cursor.1), point(to.0, to.1));
                            }
                        } else {
                            r.draw_cubic(point(cursor.0, cursor.1), point(c1.0, c1.1), point(c2.0, c2.1), point(to.0, to.1));
                        }
                        cursor = to;
                    }
                    // An outline has to close to hold an inside: several
                    // emoji's layers are open subpaths.
                    Seg::Close => {
                        if (cursor.0 - start.0).abs() > 0.01 || (cursor.1 - start.1).abs() > 0.01 {
                            r.draw_line(point(cursor.0, cursor.1), point(start.0, start.1));
                        }
                        cursor = start;
                    }
                }
            }
            r.for_each_pixel_2d(|x, y, cov| {
                if let Some((x0, y0, x1, y1)) = clip_px {
                    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                    if px < x0 || px > x1 || py < y0 || py > y1 {
                        return;
                    }
                }
                canvas.blend(x as i32, y as i32, color, cov)
            });
        }
        let _ = (left, bottom);
        // The advance is the picture's own width, so the letter beside it is
        // not crowded.
        Some(Art { canvas, dx: -ox.round() as i32, dy: -oy.round() as i32, advance: fw })
    }

    /// The bitmap one (the flags): the glyph is a PNG, decoded and scaled to
    /// an em and laid into the picture.
    fn bitmap_art(&self, face: &ttf_parser::Face, glyph: u16, ppem: u16) -> Option<Art> {
        let image = face.glyph_raster_image(ttf_parser::GlyphId(glyph), ppem)?;
        if image.format != ttf_parser::RasterImageFormat::PNG {
            return None;
        }
        let mut decoder = png::Decoder::new(std::io::Cursor::new(image.data));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
        let mut reader = decoder.read_info().ok()?;
        let mut pixels = vec![0u8; reader.output_buffer_size()];
        let frame = reader.next_frame(&mut pixels).ok()?;
        let (iw, ih) = (frame.width as usize, frame.height as usize);
        if iw == 0 || ih == 0 {
            return None;
        }
        let bytes = &pixels[..frame.buffer_size()];
        let pixel = |i: usize| -> Option<Rgba> {
            Some(match frame.color_type {
                png::ColorType::Rgba => Rgba { r: *bytes.get(i * 4)?, g: *bytes.get(i * 4 + 1)?, b: *bytes.get(i * 4 + 2)?, a: *bytes.get(i * 4 + 3)? },
                png::ColorType::Rgb => Rgba { r: *bytes.get(i * 3)?, g: *bytes.get(i * 3 + 1)?, b: *bytes.get(i * 3 + 2)?, a: 255 },
                png::ColorType::GrayscaleAlpha => {
                    let v = *bytes.get(i * 2)?;
                    Rgba { r: v, g: v, b: v, a: *bytes.get(i * 2 + 1)? }
                }
                png::ColorType::Grayscale => {
                    let v = *bytes.get(i)?;
                    Rgba { r: v, g: v, b: v, a: 255 }
                }
                png::ColorType::Indexed => return None,
            })
        };
        let (fw, fh) = (ppem as f32 * 1.05, ppem as f32 * 1.2);
        let (w, h) = (fw.ceil() as u32, fh.ceil() as u32);
        let mut canvas = Canvas::new(w, h);
        // The bitmap is laid at 0.95 em, centred across, with 0.11 em left
        // above it so that it lines up with what the vector one draws -- a
        // flag standing taller than the emoji beside it was a flag laid too
        // high (2026-11-24).
        let side = ppem as f32 * 0.95;
        let (ox, oy) = (ppem as f32 * 0.05, ppem as f32 * 0.11);
        for y in 0..side as u32 {
            for x in 0..side as u32 {
                let sx = ((x as f32 / side) * iw as f32) as usize;
                let sy = ((y as f32 / side) * ih as f32) as usize;
                let Some(color) = pixel(sy.min(ih - 1) * iw + sx.min(iw - 1)) else { continue };
                canvas.blend((ox as u32 + x) as i32, (oy as u32 + y) as i32, color, 1.0);
            }
        }
        Some(Art { canvas, dx: -ox.round() as i32, dy: -(ppem as f32).round() as i32, advance: fw })
    }
}

/// Whether two glyphs join into one ligature: the GSUB ligature tables are
/// walked for it.
fn ligature(face: &ttf_parser::Face, first: u16, second: u16) -> Option<u16> {
    let gsub = face.tables().gsub?;
    for lookup in gsub.lookups {
        for sub in lookup.subtables.into_iter::<ttf_parser::gsub::SubstitutionSubtable>() {
            let ttf_parser::gsub::SubstitutionSubtable::Ligature(lig) = sub else { continue };
            let Some(index) = lig.coverage.get(ttf_parser::GlyphId(first)) else { continue };
            let Some(set) = lig.ligature_sets.get(index) else { continue };
            for item in set {
                if item.components.len() == 1 && item.components.get(0).map(|g| g.0) == Some(second) {
                    return Some(item.glyph.0);
                }
            }
        }
    }
    None
}

/// One piece of an outline: both TrueType's quadratics and OpenType's cubics
/// are kept.
#[derive(Clone, Copy)]
enum Seg {
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

impl Seg {
    /// Every point through `f`: this is how a transform reaches an outline.
    fn map(self, f: impl Fn(f32, f32) -> (f32, f32)) -> Seg {
        let m = |x: f32, y: f32| f(x, y);
        match self {
            Seg::Move(x, y) => {
                let (x, y) = m(x, y);
                Seg::Move(x, y)
            }
            Seg::Line(x, y) => {
                let (x, y) = m(x, y);
                Seg::Line(x, y)
            }
            Seg::Quad(x1, y1, x, y) => {
                let (x1, y1) = m(x1, y1);
                let (x, y) = m(x, y);
                Seg::Quad(x1, y1, x, y)
            }
            Seg::Cubic(x1, y1, x2, y2, x, y) => {
                let (x1, y1) = m(x1, y1);
                let (x2, y2) = m(x2, y2);
                let (x, y) = m(x, y);
                Seg::Cubic(x1, y1, x2, y2, x, y)
            }
            Seg::Close => Seg::Close,
        }
    }

    /// Which points this piece falls on (control points count too; the bounds
    /// measured are only a little large).
    fn points(self) -> Vec<(f32, f32)> {
        match self {
            Seg::Move(x, y) | Seg::Line(x, y) => vec![(x, y)],
            Seg::Quad(x1, y1, x, y) => vec![(x1, y1), (x, y)],
            Seg::Cubic(x1, y1, x2, y2, x, y) => vec![(x1, y1), (x2, y2), (x, y)],
            Seg::Close => Vec::new(),
        }
    }
}

#[derive(Default)]
struct Collect {
    segs: Vec<Seg>,
}

impl ttf_parser::OutlineBuilder for Collect {
    fn move_to(&mut self, x: f32, y: f32) {
        self.segs.push(Seg::Move(x, y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.segs.push(Seg::Line(x, y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.segs.push(Seg::Quad(x1, y1, x, y));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.segs.push(Seg::Cubic(x1, y1, x2, y2, x, y));
    }
    fn close(&mut self) {
        self.segs.push(Seg::Close);
    }
}

/// One layer: which glyph it draws, in what colour, and under which
/// transform.
struct Shape {
    glyph: u16,
    color: Rgba,
    transform: ttf_parser::Transform,
    /// The box this layer is clipped to, in font units; `None` is no clip.
    clip: Option<(f32, f32, f32, f32)>,
}

/// Collects the COLR layers. `outline_glyph` records which glyph the layer
/// draws, and the `paint` that follows says its colour; `push_transform` and
/// `pop_transform` fence the layer's transform.
#[derive(Default)]
struct Layers {
    drawn: Option<u16>,
    shapes: Vec<Shape>,
    stack: Vec<ttf_parser::Transform>,
    clips: Vec<ttf_parser::colr::ClipBox>,
    /// How many `push_layer` levels deep: those layers are for compositing (a
    /// mask and the like), not shapes drawn down, and stacking them as shapes
    /// only adds what should not show.
    layers: usize,
}

impl Layers {
    /// The clip boxes in the stack put together: where there are several, only
    /// their intersection counts.
    fn clip(&self) -> Option<(f32, f32, f32, f32)> {
        self.clips.iter().fold(None, |acc: Option<(f32, f32, f32, f32)>, c| {
            let r = (c.x_min, c.y_min, c.x_max, c.y_max);
            Some(match acc {
                None => r,
                Some(a) => (a.0.max(r.0), a.1.max(r.1), a.2.min(r.2), a.3.min(r.3)),
            })
        })
    }

    /// The transforms in the stack put together: the outer first, the inner
    /// after.
    fn transform(&self) -> ttf_parser::Transform {
        let mut all = ttf_parser::Transform::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        for t in &self.stack {
            all = ttf_parser::Transform::combine(all, *t);
        }
        all
    }
}

impl<'a> ttf_parser::colr::Painter<'a> for Layers {
    fn outline_glyph(&mut self, glyph_id: ttf_parser::GlyphId) {
        self.drawn = Some(glyph_id.0);
    }

    fn paint(&mut self, paint: ttf_parser::colr::Paint<'a>) {
        // One glyph to a layer: it is cleared once the layer is done, so that
        // a later `paint` does not use the glyph before it.
        let Some(glyph) = self.drawn.take() else { return };
        if self.layers > 0 {
            return;
        }
        let color = match &paint {
            ttf_parser::colr::Paint::Solid(c) => Some(rgba(*c)),
            // A gradient at this size is one colour anyway: take its first
            // stop.
            ttf_parser::colr::Paint::LinearGradient(g) => first_stop(g.stops(0, &[])),
            ttf_parser::colr::Paint::RadialGradient(g) => first_stop(g.stops(0, &[])),
            ttf_parser::colr::Paint::SweepGradient(g) => first_stop(g.stops(0, &[])),
        };
        let Some(color) = color else { return };
        let transform = self.transform();
        let clip = self.clip();
        self.shapes.push(Shape { glyph, color, transform, clip });
    }

    fn push_clip(&mut self) {}
    fn push_clip_box(&mut self, clipbox: ttf_parser::colr::ClipBox) {
        self.clips.push(clipbox);
    }
    fn pop_clip(&mut self) {
        self.clips.pop();
    }
    fn push_layer(&mut self, _mode: ttf_parser::colr::CompositeMode) {
        self.layers += 1;
    }
    fn pop_layer(&mut self) {
        self.layers = self.layers.saturating_sub(1);
    }
    fn push_transform(&mut self, transform: ttf_parser::Transform) {
        self.stack.push(transform);
    }
    fn pop_transform(&mut self) {
        self.stack.pop();
    }
}

fn rgba(c: ttf_parser::RgbaColor) -> Rgba {
    Rgba { r: c.red, g: c.green, b: c.blue, a: c.alpha }
}

fn first_stop(stops: impl Iterator<Item = ttf_parser::colr::ColorStop>) -> Option<Rgba> {
    stops.map(|s| rgba(s.color)).next()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One character from its code point. The typeface-coverage test reads
    /// every literal the source writes and holds it to MiSans, which draws no
    /// emoji; built here, they never reach it.
    fn emoji_at(cp: u32) -> char {
        char::from_u32(cp).expect("a code point that is a character")
    }

    const COLOR: &str = "/data/local/tmp/NotoColorEmoji.ttf";
    const FLAGS: &str = "/data/local/tmp/NotoColorEmojiFlags.ttf";

    #[test]
    fn a_phone_without_the_typeface_draws_no_emoji_and_breaks_nothing() {
        let slot = Slot::default();
        // The host usually has neither typeface; where it has one, this only
        // draws, and does not fail.
        let _ = slot.art(emoji_at(0x1f600), 24.0);
        let _ = slot.flag(emoji_at(0x1f1e8), emoji_at(0x1f1f3), 24.0);
    }

    #[test]
    fn every_emoji_the_typeface_holds_draws_a_coloured_picture() {
        // The phone's two typefaces can be copied to /data/local/tmp before
        // the test; where they are not, it is skipped.
        if !Path::new(COLOR).exists() {
            return;
        }
        let emoji = Emoji::read_at(Path::new(COLOR)).expect("the typeface reads");
        // The digits, `#` and `*` have glyphs in this typeface (the keycaps
        // use them), but they are not for it to draw.
        for ch in ['0', '1', '5', '9', '#', '*'] {
            assert!(emoji.art(ch, 32.0).is_none(), "{ch:?} must stay with the UI face");
        }
        for cp in [0x1f600u32, 0x1f433, 0x1f351] {
            let ch = emoji_at(cp);
            let art = emoji.art(ch, 32.0).unwrap_or_else(|| panic!("{ch:?} draws"));
            assert!(art.canvas.w >= 16 && art.canvas.h >= 16, "{ch:?}: {}×{}", art.canvas.w, art.canvas.h);
            let coloured = art.canvas.data.as_chunks::<4>().0.iter().filter(|p| p[3] > 128).count();
            assert!(coloured > 50, "{ch:?}: only {coloured} pixels painted");
        }
        // Every pictorial character in the typeface has to be drawable: not
        // one box.
        let face = ttf_parser::Face::parse(emoji.fonts[0].data, 0).unwrap();
        let mut missing: Vec<u32> = Vec::new();
        let mut total = 0;
        for table in face.tables().cmap.unwrap().subtables {
            if !table.is_unicode() {
                continue;
            }
            table.codepoints(|cp| {
                // The zero-width joiner and the variation selectors only
                // combine; alone they are no picture.
                if cp == 0x200D || (0xFE00..=0xFE0F).contains(&cp) || cp == 0x20E3 {
                    return;
                }
                if !depicted(char::from_u32(cp).unwrap_or('\u{0}')) || cp < 0x2000 {
                    return;
                }
                total += 1;
                if emoji.art(char::from_u32(cp).unwrap(), 20.0).is_none() {
                    missing.push(cp);
                }
            });
        }
        assert!(total > 2000, "only {total} pictures in the typeface");
        assert!(missing.is_empty(), "{} of {total} draw nothing: {:?}", missing.len(), &missing[..missing.len().min(10)]);
    }

    #[test]
    fn a_flag_is_two_regional_indicators_joined_into_one_picture() {
        if !Path::new(FLAGS).exists() {
            return;
        }
        let emoji = Emoji::read_at(Path::new(FLAGS)).expect("the flag typeface reads");
        // A single regional indicator has no picture here: a pair of them is
        // a flag.
        let art = emoji.flag(emoji_at(0x1f1e8), emoji_at(0x1f1f3), 32.0).expect("China's flag draws");
        assert!(art.canvas.w >= 16 && art.canvas.h >= 16, "{}×{}", art.canvas.w, art.canvas.h);
        let coloured = art.canvas.data.as_chunks::<4>().0.iter().filter(|p| p[3] > 128).count();
        assert!(coloured > 100, "only {coloured} pixels painted");
        assert!(emoji.flag(emoji_at(0x1f1e8), emoji_at(0x1f3f3), 32.0).is_none(), "there is no such flag");
    }
}
