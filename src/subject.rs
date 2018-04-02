//! The subject of a triple.

use std::fmt;

use iri::{BlankNode, Iri};
use object::Object;

/// The subject of a Triple.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    pub(crate) fn as_object(&self) -> Object {
        match *self {
            Subject::BlankNode(ref b) => Object::BlankNode(b.clone()),
            Subject::Iri(ref i) => Object::Iri(i.clone()),
        }
    }

    pub(crate) fn into_blank_node(self) -> Option<BlankNode> {
        match self {
            Subject::BlankNode(b) => Some(b),
            _ => None,
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
