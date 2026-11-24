//! What the pages say, in every language they speak: each language's file
//! in `locales`, as the build script writes it out. A table there is a
//! module here; a text that names no value is a constant holding it in
//! every language, for [`Lang::pick`](crate::lang::Lang::pick) to choose
//! from, and one that names values in braces is a function that writes it
//! with them.

// The file holds every text the languages carry, and the pages of this
// project read the ones they say: the rest are there for the pages to come,
// and are no dead weight to warn of.
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/texts.rs"));
