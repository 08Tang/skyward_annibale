//! Chinese input: AOSP's pinyin decoder (`third_party/aosp-pinyin`,
//! Apache-2.0).
//!
//! The dictionary is not carried in the repository: `build.rs` compiles the
//! `pinyinime_dictbuilder` AOSP ships, runs it once to press
//! `rawdict_utf16_65105_freq.txt` into `dict_pinyin.dat`, and carries that
//! inside the binary. The decoder reads a file and nothing else, so it is
//! written into a private temporary directory at the first use, and the user
//! dictionary lands there too and goes when the process does.
//!
//! Where it cannot be built -- the host has no POSIX headers, or
//! `pinyin-input` is off -- the candidates are always empty and a page still
//! types letters: with no candidate, the letters are written as they stand.

#[cfg(feature = "pinyin-input")]
use std::{
    ffi::CString,
    sync::{Mutex, OnceLock},
};

/// The decoder's limits: the longest spelling and the most characters one
/// candidate may hold, as its C adapter has them.
#[cfg(feature = "pinyin-input")]
const MAX_SPELLING: usize = 32;
#[cfg(feature = "pinyin-input")]
const MAX_CHARS: usize = 16;
#[cfg(feature = "pinyin-input")]
const MAX_CANDIDATES: usize = 24;

/// One candidate: the characters written into the field, and how much of
/// the spelling they take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub text: String,
    pub consumed: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    pub enabled: bool,
    pub spelling: String,
    pub page: usize,
    pub candidates: Vec<Candidate>,
}

impl Default for Composition {
    /// Off by default: a field takes English first, and the Chinese/English
    /// switch turns pinyin on. A password or an address would otherwise have
    /// its letters taken by the spelling (the keyboard is everywhere, so the
    /// letters cannot be collected by default).
    fn default() -> Self {
        Composition { enabled: false, spelling: String::new(), page: 0, candidates: Vec::new() }
    }
}

impl Composition {
    pub fn update(&mut self) {
        self.candidates = candidates(&self.spelling);
        self.page = 0;
    }
    pub fn clear(&mut self) {
        self.spelling.clear();
        self.candidates.clear();
        self.page = 0;
    }
    pub fn choose(&mut self, index: usize) -> Option<String> {
        let candidate = self.candidates.get(index)?.clone();
        self.spelling.drain(..candidate.consumed.min(self.spelling.len()));
        self.spelling = self.spelling.trim_start_matches('\'').into();
        self.update();
        Some(candidate.text)
    }
}

#[cfg(feature = "pinyin-input")]
unsafe extern "C" {
    fn im_open_decoder(system: *const std::ffi::c_char, user: *const std::ffi::c_char) -> bool;
    fn im_set_max_lens(spelling: usize, characters: usize);
    fn skyward_pinyin_query(input: *const std::ffi::c_char, index: usize, output: *mut u16, capacity: usize, consumed: *mut usize)
    -> usize;
}

#[cfg(feature = "pinyin-input")]
struct Engine {
    _files: tempfile::TempDir,
}

/// Why the engine did not open. A page only ever sees that the candidates
/// are empty; the reason is kept here, for the runtime to copy into kmsg,
/// where `adb shell dmesg | grep pinyin` finds it.
#[cfg(feature = "pinyin-input")]
static TROUBLE: Mutex<Option<String>> = Mutex::new(None);

#[cfg(feature = "pinyin-input")]
fn trouble(why: impl Into<String>) -> Option<Engine> {
    let why = why.into();
    if let Ok(mut slot) = TROUBLE.lock() {
        *slot = Some(why);
    }
    None
}

/// The temporary directory: `/tmp` first, which recovery's rootfs always
/// has and upstream uses too, then `TMPDIR` and the system default. Not a
/// bare `tempdir()`: on Android `TMPDIR` is often `/data/local/tmp`, and
/// `/data` is not mounted in recovery, so that directory cannot be made.
#[cfg(feature = "pinyin-input")]
fn temporary() -> Result<tempfile::TempDir, String> {
    let mut tried = Vec::new();
    for dir in ["/tmp", ""] {
        let mut builder = tempfile::Builder::new();
        builder.prefix("skyward-pinyin-");
        let made = if dir.is_empty() { builder.tempdir() } else { builder.tempdir_in(dir) };
        match made {
            Ok(files) => return Ok(files),
            Err(e) => tried.push(format!("{}: {e}", if dir.is_empty() { "TMPDIR" } else { dir })),
        }
    }
    Err(format!("no usable temporary directory（{}）", tried.join(";")))
}

#[cfg(feature = "pinyin-input")]
impl Engine {
    fn open() -> Option<Self> {
        let files = match temporary() {
            Ok(files) => files,
            Err(why) => return trouble(why),
        };
        let system = files.path().join("system.dat");
        let user = files.path().join("user.dat");
        let dictionary = include_bytes!(concat!(env!("OUT_DIR"), "/res/raw/dict_pinyin.dat"));
        if let Err(e) = std::fs::write(&system, dictionary) {
            return trouble(format!("writing the dictionary to {}: {e}", system.display()));
        }
        let (system, user) = (CString::new(system.to_str()?).ok()?, CString::new(user.to_str()?).ok()?);
        if !unsafe { im_open_decoder(system.as_ptr(), user.as_ptr()) } {
            return trouble("im_open_decoder refused the dictionary");
        }
        unsafe {
            im_set_max_lens(MAX_SPELLING, MAX_CHARS);
        }
        Some(Self { _files: files })
    }
}

#[cfg(feature = "pinyin-input")]
fn engine() -> &'static Mutex<Option<Engine>> {
    static ENGINE: OnceLock<Mutex<Option<Engine>>> = OnceLock::new();
    ENGINE.get_or_init(|| Mutex::new(Engine::open()))
}

/// Open the engine now, and bring back why it did not open, if it did not.
/// The runtime calls this once after boot, so that the log says whether the
/// dictionary, the temporary directory or the decoder itself is at fault,
/// without waiting for someone to press a key first.
#[cfg(feature = "pinyin-input")]
pub fn prepare() -> Option<String> {
    let _opened = engine().lock().ok()?;
    TROUBLE.lock().ok().and_then(|slot| slot.clone())
}

/// A build with no engine: there is never a reason to report.
#[cfg(not(feature = "pinyin-input"))]
pub fn prepare() -> Option<String> {
    None
}

#[cfg(feature = "pinyin-input")]
static LAST: Mutex<Option<(String, Vec<Candidate>)>> = Mutex::new(None);

#[cfg(not(feature = "pinyin-input"))]
pub fn candidates(_: &str) -> Vec<Candidate> {
    Vec::new()
}

#[cfg(feature = "pinyin-input")]
pub fn candidates(spelling: &str) -> Vec<Candidate> {
    if spelling.is_empty() || spelling.len() > MAX_SPELLING || !spelling.bytes().all(|c| c.is_ascii_lowercase() || c == b'\'') {
        return Vec::new();
    }
    // A frame asks more than once -- once for the page, once for the
    // layout -- and one search of the decoder is not cheap.
    if let Ok(last) = LAST.lock()
        && let Some((asked, picks)) = last.as_ref()
        && asked == spelling
    {
        return picks.clone();
    }
    let result = decode(spelling);
    if let Ok(mut last) = LAST.lock() {
        *last = Some((spelling.to_string(), result.clone()));
    }
    result
}

#[cfg(feature = "pinyin-input")]
fn decode(spelling: &str) -> Vec<Candidate> {
    let Ok(engine) = engine().lock() else {
        return Vec::new();
    };
    if engine.is_none() {
        return Vec::new();
    }
    let Ok(input) = CString::new(spelling) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for index in 0..MAX_CANDIDATES {
        let mut text = [0u16; MAX_CHARS + 1];
        let mut consumed = 0;
        let count = unsafe { skyward_pinyin_query(input.as_ptr(), index, text.as_mut_ptr(), text.len(), &mut consumed) };
        if index >= count {
            break;
        }
        let end = text.iter().position(|c| *c == 0).unwrap_or(text.len());
        if end > 0
            && consumed > 0
            && let Ok(text) = String::from_utf16(&text[..end])
        {
            result.push(Candidate { text, consumed });
        }
    }
    result
}

#[cfg(all(test, feature = "pinyin-input"))]
mod tests {
    use super::*;
    #[test]
    fn real_dictionary_decodes_chinese_and_keeps_a_bounded_candidate_list() {
        let result = candidates("nihao");
        assert!(result.iter().any(|c| c.text == "你好"), "{result:?}");
        assert!(result.len() <= MAX_CANDIDATES);
        for invalid in ["", "../../data", "你", "ni hao", "0123"] {
            assert!(candidates(invalid).is_empty());
        }
        assert!(candidates(&"a".repeat(33)).is_empty());
    }
    #[test]
    fn choosing_a_phrase_consumes_its_spelling_without_committing_raw_letters() {
        let mut composition = Composition { spelling: "nihao".into(), ..Composition::default() };
        composition.update();
        let index = composition.candidates.iter().position(|c| c.text == "你好").unwrap();
        assert_eq!(composition.choose(index).as_deref(), Some("你好"));
        assert!(composition.spelling.is_empty());
    }
}
