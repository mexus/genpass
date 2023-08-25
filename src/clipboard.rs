//! Clipboard manipulations.

use std::borrow::Cow;

use snafu::{ResultExt, Snafu};

/// Clipboard manipulation error.
#[derive(Debug, Snafu)]
pub enum Error {
    /// Unable to initialize clipboard.
    #[snafu(display("Unable to initialize clipboard"))]
    InitClipboard {
        /// Source error.
        source: arboard::Error,
    },

    /// Unable to store the password to the clipboard.
    #[snafu(display("Unable to store the password to the clipboard"))]
    ClipboardStore {
        /// Source error.
        source: arboard::Error,
    },
}

/// Stores a value to the clipboard.
///
/// On linux, will wait for the clipboard’s contents to be replaced after
/// setting it.
pub fn store(value: &str) -> Result<(), Error> {
    store_impl(value)
}

#[cfg(target_os = "linux")]
fn store_impl<'a, T>(value: T) -> Result<(), Error>
where
    T: Into<Cow<'a, str>>,
{
    use arboard::SetExtLinux;
    arboard::Clipboard::new()
        .context(InitClipboardSnafu)?
        .set()
        .wait()
        .text(value)
        .context(ClipboardStoreSnafu)
}

#[cfg(not(target_os = "linux"))]
fn store_impl<'a, T>(value: T) -> Result<(), Error>
where
    T: Into<Cow<'a, str>>,
{
    arboard::Clipboard::new()
        .context(InitClipboardSnafu)?
        .set()
        .text(value)
        .context(ClipboardStoreSnafu)
}
