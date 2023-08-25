//! Symbols manipulations.

use std::{collections::BTreeSet, str::FromStr};

use rand::{prelude::Distribution, seq::IteratorRandom, Rng};
use snafu::Snafu;

/// A non-empty set of symbols.
#[derive(Clone)]
pub struct SymbolsSet {
    inner: BTreeSet<char>,
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

impl SymbolsSet {
    /// Returns a set of latin uppercase symbols.
    pub fn latin_upper() -> Self {
        Self::from(LATIN_UPPER_SET)
    }

    /// Returns a set of latin lowercase symbols.
    pub fn latin_lower() -> Self {
        Self::from(LATIN_LOWER_SET)
    }

    /// Returns a set of digits symbols.
    pub fn digits() -> Self {
        Self::from(DIGITS_SET)
    }

    /// Returns a set of special symbols.
    pub fn special() -> Self {
        Self::from(SPECIAL_SET)
    }

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
        assert_ne!(N, 0, "Set can not be empty");
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

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn add() {
        let result = SymbolsSet::from(['a'])
            .add(&SymbolsSet::from(['b']))
            .inner
            .into_iter()
            .collect::<Vec<char>>();
        assert_eq!(result, vec!['a', 'b']);
    }

    #[test]
    fn remove() {
        let result = SymbolsSet::from(['a', 'b'])
            .subtract(&SymbolsSet::from(['b']))
            .expect("Must be non-empty")
            .inner
            .into_iter()
            .collect::<Vec<char>>();
        assert_eq!(result, vec!['a']);

        assert!(SymbolsSet::from(['a', 'b'])
            .subtract(&SymbolsSet::from(['b', 'a']))
            .is_none());
    }

    #[quickcheck]
    fn parse(input: String) -> TestResult {
        if input.is_empty() {
            return TestResult::discard();
        }

        let symbols: BTreeSet<char> = input.chars().collect();
        let symbols_parsed: SymbolsSet = input.parse().unwrap();

        assert_eq!(symbols, symbols_parsed.inner);

        TestResult::passed()
    }

    #[test]
    fn parse_empty() {
        let result: Result<SymbolsSet, _> = "".parse();
        result.unwrap_err();
    }
}
