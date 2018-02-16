//! A literal (String, Integer, Decimal, Double, Bool)
use std::fmt;

use iri::Iri;

macro_rules! rdf {
    ($typ:expr) => {
        concat!("http://www.w3.org/1999/02/22-rdf-syntax-ns#", $typ)
    };
}

macro_rules! xsd {
    ($typ:expr) => {
        concat!("http://www.w3.org/2001/XMLSchema#", $typ)
    };
}

/// A Literal
#[derive(Clone, Debug)]
pub enum Literal {
    /// A RDF literal
    RdfLiteral {
        /// The raw value of string.
        value: String,
        /// The language tag of the string.
        language_tag: Option<String>,
        /// The iri type.
        iri: Option<Iri>,
    },

    /// Integer
    Integer(String),
    /// Decimal
    Decimal(String),
    /// Double
    Double(String),
    /// Bool
    Bool(String),
}

impl Literal {

    /// The datatype of the literal.
    pub fn datatype(&self) -> &str {
        match *self {
            Literal::RdfLiteral { ref language_tag, ref iri, .. } => {
                if let Some(iri) = iri.as_ref() {
                    iri
                } else if let Some(_) = language_tag.as_ref() {
                    rdf!("langString")
                } else {
                    xsd!("string")
                }
            }

            Literal::Integer(_) => xsd!("integer"),
            Literal::Double(_) => xsd!("double"),
            Literal::Decimal(_) => xsd!("decimal"),
            Literal::Bool(_) => xsd!("boolean"),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut value = format!("\"{}\"", match *self {
            Literal::RdfLiteral { ref value, .. } => value,
            Literal::Integer(ref value) => value,
            Literal::Double(ref value) => value,
            Literal::Decimal(ref value) => value,
            Literal::Bool(ref value) => value
        });

        if let Literal::RdfLiteral {ref language_tag, ..} = *self {
            if let &Some(ref tag) = language_tag {
                value.push_str(&format!("@{}", tag));
            }
        }

        write!(f, "{}^^{}", value, self.datatype())
    }
}
