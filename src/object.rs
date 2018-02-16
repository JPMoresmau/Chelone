//! An object from RDF graph.
use std::fmt;

use iri::{BlankNode, Iri};
use literal::Literal;
use subject::Subject;

/// The object at end of a Triple.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Object {
    /// An Iri
    Iri(Iri),
    /// A blank node
    BlankNode(BlankNode),
    /// A literal
    Literal(Literal),
}

impl Object {
    pub(crate) fn to_subject(self) -> Subject {
        match self {
            Object::Iri(iri) => Subject::Iri(iri),
            Object::BlankNode(node) => Subject::BlankNode(node),
            _ => panic!("Tried to convert literal to subject"),
        }
    }
}


impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Iri(ref iri) => iri.fmt(f),
            Object::BlankNode(ref node) => node.fmt(f),
            Object::Literal(ref literal) => literal.fmt(f),
        }
    }
}
