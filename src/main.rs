use std::num::NonZeroU32;

use clap::Parser;
use genpass::symbols::SymbolsSet;
use rand::Rng;
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

    /// Clipboard error.
    #[snafu(display("Clipboard error"))]
    Clipboard {
        /// Source error.
        source: genpass::clipboard::Error,
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
        merge(&mut maybe_set, SymbolsSet::latin_upper());
    }
    if !no_latin && !no_latin_lower {
        merge(&mut maybe_set, SymbolsSet::latin_lower())
    }
    if !no_digits {
        merge(&mut maybe_set, SymbolsSet::digits())
    }
    if !no_special {
        merge(&mut maybe_set, SymbolsSet::special())
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
                    // We create a session to detach from the parent.
                    nix::unistd::setsid().context(SessionCreateSnafu)?;
                    tracing::debug!("Session created");
                    genpass::clipboard::store(&password).context(ClipboardSnafu)?;
                    tracing::debug!("Lost clipboard ownership; terminating");
                }
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            genpass::clipboard::store(&password).context(ClipboardSnafu)?;
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
