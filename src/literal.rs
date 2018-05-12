//! A literal (String, Integer, Decimal, Double, Bool)
use std::fmt;

use iri::Iri;


/// A Literal
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Literal {
    /// The raw value of string.
    pub value: String,
    /// The language tag of the string.
    pub language_tag: Option<String>,
    /// The iri type.
    pub iri: Iri,
}

impl Literal {

    /// Creates a new `Literal`. If `iri` is `None` and `language_tag` the
    /// literal's IRI(data type) will be
    /// `http://www.w3.org/1999/02/22-rdf-syntax-ns#langString` else it will
    /// be `http://www.w3.org/2001/XMLSchema#string`
    pub fn new(value: String, language_tag: Option<String>, iri: Option<Iri>)
        -> Self
    {
        let iri = if let Some(iri) = iri {
            iri
        } else if let Some(_) = language_tag.as_ref() {
            rdf!("langString")
        } else {
            xsd!("string")
        };

        Literal {
            value,
            language_tag,
            iri,
        }
    }

    pub(crate) fn new_bool(value: String) -> Self {
        Self::new(value, None, Some(xsd!("boolean")))
    }

    pub(crate) fn new_double(value: String) -> Self {
        Self::new(value, None, Some(xsd!("double")))
    }

    pub(crate) fn new_decimal(value: String) -> Self {
        Self::new(value, None, Some(xsd!("decimal")))
    }

    pub(crate) fn new_integer(value: String) -> Self {
        Self::new(value, None, Some(xsd!("integer")))
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut value = format!("\"{}\"", self.value);

        if let Some(tag) = &self.language_tag {
            value.push_str(&format!("@{}", tag));
        }

        write!(f, "{}^^{}", value, self.iri)
    }
}
