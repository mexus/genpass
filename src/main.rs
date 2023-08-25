use std::{collections::BTreeSet, num::NonZeroU32, str::FromStr};

use arboard::Clipboard;
#[cfg(target_os = "linux")]
use arboard::SetExtLinux;

use clap::Parser;
use rand::{prelude::Distribution, seq::IteratorRandom, Rng};
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// Turns off latin lowercase symbols.
    #[clap(long)]
    no_latin_lower: bool,

    /// Turns off latin uppercase symbols.
    #[clap(long)]
    no_latin_upper: bool,

    /// Turns off latin symbols.
    #[clap(long)]
    no_latin: bool,

    /// Turns off digits.
    #[clap(long)]
    no_digits: bool,

    /// Turns off special symbols.
    #[clap(long)]
    no_special: bool,

    /// Allow additional symbols.
    ///
    /// You can repeat this multiple times to allow additional sets of symbols.
    #[clap(short = 'a', long = "allow")]
    allowed: Vec<SymbolsSet>,

    /// Deny additional symbols.
    ///
    /// You can repeat this multiple times to forbid additional sets of symbols.
    /// Takes precedence over the whitelist.
    #[clap(short = 'd', long = "deny")]
    disallowed: Vec<SymbolsSet>,

    /// Be verbose.
    #[clap(short, long)]
    verbose: bool,

    /// Copy generated password to the clipboard.
    ///
    /// In this case the generated password won't be printed to the stdout.
    #[clap(short, long)]
    copy: bool,

    /// Length of the generated password, in unicode scalar values.
    #[clap(default_value_t = nonzero_ext::nonzero!(24u32))]
    length: NonZeroU32,
}

/// A non-empty set of symbols.
#[derive(Clone)]
pub struct SymbolsSet {
    inner: BTreeSet<char>,
}

impl SymbolsSet {
    /// Subtracts another set from the current.
    ///
    /// Returns [`None`] if the result is an empty set.
    pub fn subtract(&self, other: &SymbolsSet) -> Option<Self> {
        let inner: BTreeSet<char> = self.inner.difference(&other.inner).copied().collect();
        if inner.is_empty() {
            None
        } else {
            Some(Self { inner })
        }
    }

    /// Adds another set to the current.
    pub fn add(&self, other: &SymbolsSet) -> Self {
        Self {
            inner: self.inner.union(&other.inner).copied().collect(),
        }
    }

    /// Returns the amount of symbols.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Always returns false, since the set is never empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl Distribution<char> for &'_ SymbolsSet {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        *self
            .inner
            .iter()
            .choose(rng)
            .expect("The list is non-empty")
    }
}

impl<const N: usize> From<[char; N]> for SymbolsSet {
    fn from(list: [char; N]) -> Self {
        assert_ne!(N, 0);
        let inner: BTreeSet<char> = list.into_iter().collect();
        Self { inner }
    }
}

impl std::fmt::Debug for SymbolsSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolsSet")
            .field("inner", &self.inner)
            .finish()
    }
}

impl std::fmt::Display for SymbolsSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for value in &self.inner {
            write!(f, "{value}")?;
        }
        Ok(())
    }
}

const LATIN_UPPER_SET: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const LATIN_LOWER_SET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const DIGITS_SET: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const SPECIAL_SET: [char; 32] = [
    '`', '~', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+', '[', ']', '{',
    '}', '\\', '|', ';', ':', '\'', '"', ',', '<', '.', '>', '/', '?',
];

/// Unable to parse a [`SymbolsSet`] from a string.
#[derive(Debug, Snafu)]
#[snafu(display("Symbols set can't be empty"))]
pub struct SymbolsSetParseError {}

impl FromStr for SymbolsSet {
    type Err = SymbolsSetParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: BTreeSet<char> = s.chars().collect();
        snafu::ensure!(!values.is_empty(), SymbolsSetParseSnafu);
        Ok(Self { inner: values })
    }
}

#[test]
fn check_args() {
    <Args as clap::CommandFactory>::command().debug_assert();
}

/// Run error.
#[derive(Debug, Snafu)]
pub enum Error {
    /// No symbols are allowed to generate password with.
    #[snafu(display("No symbols are allowed to generate password with"))]
    EmptySet,

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

    /// Unable to fork the process.
    #[snafu(display("Unable to fork the process"))]
    ForkFailed {
        /// Source error.
        source: nix::errno::Errno,
    },

    /// Unable to create a session (see `man 2 setsid`).
    #[snafu(display("Unable to create a session (see `man 2 setsid`)"))]
    SessionCreate {
        /// Source error.
        source: nix::errno::Errno,
    },
}

fn main() -> snafu::Report<Error> {
    snafu::Report::capture(run)
}

fn run() -> Result<(), Error> {
    let Args {
        no_latin_lower,
        no_latin_upper,
        no_latin,
        no_digits,
        no_special,
        allowed,
        disallowed,
        length,
        verbose,
        copy,
    } = Args::parse();

    let level = if verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::WARN
    };
    tracing_subscriber::fmt()
        .without_time()
        .compact()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .init();

    let empty = (no_latin || (no_latin_upper && no_latin_lower))
        && no_digits
        && no_special
        && allowed.is_empty();
    snafu::ensure!(!empty, EmptySetSnafu);

    let mut maybe_set = None::<SymbolsSet>;
    if !no_latin && !no_latin_upper {
        merge(&mut maybe_set, SymbolsSet::from(LATIN_UPPER_SET));
    }
    if !no_latin && !no_latin_lower {
        merge(&mut maybe_set, SymbolsSet::from(LATIN_LOWER_SET))
    }
    if !no_digits {
        merge(&mut maybe_set, SymbolsSet::from(DIGITS_SET))
    }
    if !no_special {
        merge(&mut maybe_set, SymbolsSet::from(SPECIAL_SET))
    }
    for allowed in allowed {
        merge(&mut maybe_set, allowed);
    }
    let mut result_symbols = maybe_set.context(EmptySetSnafu)?;

    for disallowed in disallowed {
        tracing::debug!("Remove symbols {disallowed}");
        result_symbols = result_symbols
            .subtract(&disallowed)
            .context(EmptySetSnafu)?;
    }

    let symbols = result_symbols;
    tracing::debug!("Symbols to use: {symbols}");

    if symbols.len() == 1 {
        tracing::warn!("There is only one symbol available for password generation")
    }

    let password: String = rand::rngs::OsRng
        .sample_iter(&symbols)
        .take(length.get() as usize)
        .collect();
    if copy {
        #[cfg(target_os = "linux")]
        {
            match unsafe { nix::unistd::fork() }.context(ForkFailedSnafu)? {
                nix::unistd::ForkResult::Parent { child } => {
                    tracing::debug!("Process daemonized and now running with pid {child}");
                }
                nix::unistd::ForkResult::Child => {
                    nix::unistd::setsid().context(SessionCreateSnafu)?;
                    tracing::debug!("Session created");
                    let mut c = Clipboard::new().context(InitClipboardSnafu)?;
                    c.set().wait().text(password).context(ClipboardStoreSnafu)?;
                    tracing::debug!("Lost clipboard ownership; terminating");
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            Clipboard::new()
                .context(InitClipboardSnafu)?
                .set_text(password)
                .context(ClipboardStoreSnafu)?;
        }
    } else {
        println!("{password}");
    }

    Ok(())
}

fn merge(maybe_set: &mut Option<SymbolsSet>, another: SymbolsSet) {
    tracing::debug!("Add symbols {another}");
    if let Some(set) = maybe_set {
        *set = set.add(&another);
    } else {
        *maybe_set = Some(another)
    }
}
