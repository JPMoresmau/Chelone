//! The subject of a triple.

use std::fmt;

use iri::{BlankNode, Iri};

/// The subject of a Triple.
#[derive(Clone, Debug)]
pub enum Subject {
    /// An IRI
    Iri(Iri),
    /// A blank node
    BlankNode(BlankNode),
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match *self {
            Subject::Iri(ref iri) => iri.to_string(),
            Subject::BlankNode(ref node) => node.to_string(),
        };

        write!(f, "{}", val)
    }
}
