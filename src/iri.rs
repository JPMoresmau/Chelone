//! An Internationalized Resource Identifier.
use std::fmt;
use std::ops::{Deref, DerefMut};

use url::Url;

/// Iri with containing url, inner string does not contain wrapping `<>`, they
/// can be added by calling `iri.to_string`.
#[derive(Clone, Eq, Hash, PartialOrd, Ord)]
pub struct Iri {
    url: Url,
    had_fragment: bool,
}

impl Iri {
    /// Parses `&str` into `Iri`. Uses `Url::parse` internally.
    pub fn parse(raw: &str) -> Result<Self, url::ParseError> {
        let had_fragment = raw.contains("#");
        let url = Url::parse(raw)?;


        Ok(Iri { url, had_fragment })
    }

    /// Parses `&str` into `Iri`, adding it the base `Iri` optionally provided.
    /// Uses `Url::parse` internally.
    pub fn parse_with_base_iri(raw: &str, base: Option<&Iri>)
        -> Result<Self, url::ParseError>
    {
        let had_fragment = raw.contains("#");
        let url = Url::options()
            .base_url(base.map(|b| &**b))
            .parse(raw)
            .unwrap();


        Ok(Iri { url, had_fragment })
    }

    /// Joins a relative path to a `Iri` if the `Iri` did not end with a `#` in
    /// which case sets the path fragment, calls the the `Url::join` and
    /// `Url::set_fragment` methods respectively internally.
    pub fn join(mut self, relative: &str) -> Result<Self, url::ParseError> {
        let new = if self.had_fragment {
            self.url.set_fragment(Some(relative));
            self
        } else {
            Iri { url: self.url.join(relative)?, had_fragment: false, }
        };


        Ok(new)
    }
}

impl PartialEq for Iri {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl fmt::Debug for Iri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.url.fmt(f)
    }
}

/// A blank node generated at parse time.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlankNode(pub String);

impl Deref for Iri {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl DerefMut for Iri {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.url
    }
}

impl fmt::Display for Iri {
    fn fmt(&self, f: &mut fmt::Formatter)-> fmt::Result {
        write!(f,"<{}>", self.url)
    }
}

impl fmt::Display for BlankNode {
    fn fmt(&self, f: &mut fmt::Formatter)-> fmt::Result {
        write!(f,"_:{}", self.0)
    }
}
