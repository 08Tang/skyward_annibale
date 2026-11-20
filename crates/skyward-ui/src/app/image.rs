//! The image backup page: a partition written out whole as an image file, an
//! image written back over a partition, and the images kept that are no
//! longer wanted.
//!
//! An image is the partition's own bytes, start to end, with nothing added:
//! writing one back puts the partition in exactly the state it was in when
//! it was taken. The digest of the whole image is taken as it is written, so
//! an image written back is checked against the one that was saved before a
//! single byte of the partition is touched.

use super::*;

/// A partition an image can be taken from or written to, as the runtime
/// found it: its name, and how large the partition is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImagePart {
    pub name: String,
    /// The partition's size in bytes, which is the image's size too.
    pub size: u64,
}

/// An image already saved, as the page lists it: its file, the partition it
/// was taken from, and how large it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageFile {
    /// The file's name, which is also what it is listed by.
    pub name: String,
    /// The partition the image was taken from, where the name still says it.
    pub partition: String,
    /// Where the file sits, for the runtime to write or delete.
    pub path: String,
    /// The digest of the whole image, as it was taken: what writing it back
    /// is checked against before the partition is touched.
    pub sha256: String,
    pub size: u64,
    /// When it was written, in seconds since the epoch.
    pub made: i64,
}

/// What the image backup page is doing, or how it last ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageJob {
    /// Whether this is an image being written out or written back.
    pub kind: ImageFlow,
    /// The partition the work is on, and how many of its bytes are done.
    pub partition: String,
    pub done: u64,
    pub of: u64,
    /// Asked to stop, and not yet ended.
    pub stopping: bool,
    /// How it ended, once it has.
    pub outcome: Option<ImageEnd>,
}

/// Whether an image is being taken from a partition or put back over one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFlow {
    /// Written out to a file.
    Export,
    /// Written back over the partition from a file.
    Import,
}

/// How work on an image ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageEnd {
    /// The image was written, at this path.
    Exported(String),
    /// The image was written back over the partition.
    Imported(String),
    /// The image was deleted.
    Deleted(String),
    /// Stopped as asked, with the half-written image taken away again.
    Stopped,
    /// The image does not hold what it should, so nothing was written.
    Mismatched { wanted: String, found: String },
    Failed(String),
}

impl App {
    /// The runtime read the partitions an image can be taken from, and the
    /// images already saved, or could not: keep them and show them.
    pub fn set_images(&mut self, parts: Result<Vec<ImagePart>, String>, files: Result<Vec<ImageFile>, String>) {
        if self.state.image_parts != parts {
            self.state.image_parts = parts;
            self.refresh();
        }
        if self.state.image_files != files {
            self.state.image_files = files;
            self.refresh();
        }
    }

    /// Work on an image has begun, or has moved on: how far it has come.
    pub fn image_at(&mut self, kind: ImageFlow, partition: &str, done: u64, of: u64) {
        self.state.image_job = Some(ImageJob { kind, partition: partition.to_owned(), done, of, stopping: false, outcome: None });
        self.refresh();
    }

    /// Work on an image has ended, and how.
    pub fn image_ended(&mut self, end: ImageEnd) {
        if let Some(job) = self.state.image_job.as_mut() {
            job.outcome = Some(end);
            // A finished job that went through leaves the list of images
            // saying what it was, so the runtime reads it again.
            self.refresh();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_size_of_an_image_is_the_size_of_its_partition() {
        let parts = vec![ImagePart { name: "boot_a".into(), size: 100 * 1024 * 1024 }];
        let mut app = App::new(Screen::ImageBackup, 1440, 3200, 3.5);
        app.set_images(Ok(parts.clone()), Ok(Vec::new()));
        assert_eq!(app.state.image_parts, Ok(parts), "the partitions read are the ones shown");
        assert!(app.state.image_files == Ok(Vec::new()), "and no images are saved yet");
    }

    #[test]
    fn a_job_says_how_far_it_has_come_and_how_it_ended() {
        let mut app = App::new(Screen::ImageBackup, 1440, 3200, 3.5);
        app.image_at(ImageFlow::Export, "boot_a", 0, 1000);
        let job = app.state.image_job.as_ref().expect("a job under way");
        assert_eq!((job.done, job.of), (0, 1000), "nothing written yet");
        assert!(job.outcome.is_none(), "and it has not ended");
        assert!(!job.stopping, "and nothing has asked it to stop");
        app.image_at(ImageFlow::Export, "boot_a", 400, 1000);
        assert_eq!(app.state.image_job.as_ref().unwrap().done, 400, "it moves on");
        app.image_ended(ImageEnd::Exported("/data/media/0/skyward/boot_a.img".into()));
        let job = app.state.image_job.as_ref().expect("the job is kept to say how it ended");
        assert_eq!(job.outcome, Some(ImageEnd::Exported("/data/media/0/skyward/boot_a.img".into())));
    }

    #[test]
    fn an_image_that_does_not_hold_what_it_should_writes_nothing() {
        // The two digests are both said, so the page can show what was wanted
        // beside what was found.
        let end = ImageEnd::Mismatched { wanted: "ab".repeat(32), found: "cd".repeat(32) };
        let mut app = App::new(Screen::ImageBackup, 1440, 3200, 3.5);
        app.image_at(ImageFlow::Import, "boot_a", 0, 1000);
        app.image_ended(end.clone());
        assert_eq!(app.state.image_job.unwrap().outcome, Some(end), "nothing was written, and it says why");
    }
}
