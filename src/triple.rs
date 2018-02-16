//! Output of `Graph::parse`

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::iter::IntoIterator;

use iri::Iri;
use object::Object;
use subject::Subject;

/// A set of Triples
#[derive(Clone, Debug, Default)]
pub struct Triples(pub Vec<Triple>);

impl IntoIterator for Triples {
    type Item = Triple;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for Triples {
    type Target = Vec<Triple>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Triples {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Triples {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        for triple in &self.0 {
            write!(f, "{}\n", triple)?;
        }

        Ok(())
    }
}

/// A single triple containing a subject, predicate, and object.
#[derive(Clone, Debug)]
pub struct Triple(pub Subject, pub Iri, pub Object);

impl fmt::Display for Triple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} .", self.0, self.1, self.2)
    }
}
