//! The subject of a triple.

use std::fmt;

use iri::{BlankNode, Iri};

/// The subject of a Triple.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Subject {
    /// An IRI
    Iri(Iri),
    /// A blank node
    BlankNode(BlankNode),
}

impl Subject {
    /// Is the subject a blank node
    pub fn is_blank_node(&self) -> bool {
        match *self {
            Subject::BlankNode(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Subject::Iri(ref iri) => iri.fmt(f),
            Subject::BlankNode(ref node) => node.fmt(f),
        }
    }
}
