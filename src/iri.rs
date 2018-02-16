//! An Internationalized Resource Identifier.
use std::fmt;
use std::ops::{Deref, DerefMut};

/// Iri with containing url, inner string does not contain wrapping `<>`, they
/// can be added by calling `iri.to_string`.
#[derive(Clone, Debug, Default)]
pub struct Iri(pub String);

/// A blank node generated at parse time.
#[derive(Clone, Debug, Default)]
pub struct BlankNode(pub String);

impl Deref for Iri {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Iri {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Iri {
    fn fmt(&self, f: &mut fmt::Formatter)-> fmt::Result {
        write!(f,"<{}>", self.0)
    }
}

impl fmt::Display for BlankNode {
    fn fmt(&self, f: &mut fmt::Formatter)-> fmt::Result {
        write!(f,"_:{}", self.0)
    }
}
