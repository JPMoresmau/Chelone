//! Output of `Graph::parse`

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::iter::IntoIterator;
use std::collections::HashMap;

use itertools::Itertools;
use petgraph::Graph;
use petgraph::algo;

use iri::{BlankNode, Iri};
use object::Object;
use subject::Subject;

/// A set of Triples
#[derive(Clone, Debug, Default)]
pub struct Triples(pub Vec<Triple>);

/// A struct to provide search functionality over `Triples`.
#[derive(Clone, Debug, Default)]
pub struct TripleSearcher<'a> {
    subject: Option<&'a Subject>,
    predicate: Option<&'a Iri>,
    object: Option<&'a Object>
}

impl<'a> TripleSearcher<'a> {
    /// Creates an empty searcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the searcher to find a subject matching the provided `Subject`.
    pub fn subject<S: Into<&'a Subject>>(mut self, subject: S) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Sets the searcher to find a predicate matching the provided `Iri`.
    pub fn predicate<I: Into<&'a Iri>>(mut self, predicate: I) -> Self {
        self.predicate = Some(predicate.into());
        self
    }

    /// Sets the searcher to find a object matching the provided `Object`.
    pub fn object<O: Into<&'a Object>>(mut self, object: O) -> Self {
        self.object = Some(object.into());
        self
    }

    /// Searches through triples to find triple that matches any of the results.
    /// If a parameter is none then any triple that matches the rest of the
    /// conditions. An empty searcher will always return `None`.
    pub fn execute(self, triples: &Triples) -> Option<Triple> {
        triples.iter().find(move |triple| {
            macro_rules! matched {
                ($prop:ident) => {{
                    if let Some(item) = self.$prop {
                        &triple.$prop == item
                    } else {
                        true
                    }
                }}
            }

            matched!(subject) && matched!(predicate) && matched!(object)
        }).cloned()
    }

    /// Searches through triples to find triples that matches any of the
    /// results.  If a parameter is none then any triple that matches the
    /// rest of the conditions. An empty searcher will always return `None`.
    pub fn execute_multiple(self, triples: &Triples) -> Vec<Triple> {
        triples.iter().filter(move |triple| {
            macro_rules! matched {
                ($prop:ident) => {{
                    if let Some(item) = self.$prop {
                        &triple.$prop == item
                    } else {
                        true
                    }
                }}
            }

            matched!(subject) && matched!(predicate) && matched!(object)
        }).cloned().collect()
    }
}

impl Triples {

    /// Determines if two graphs are isomorphic.
    pub fn is_isomorphic(&mut self, other: &mut Self) -> bool {
        if self.len() != other.len() {
            return false
        }

        self.0.sort();
        other.0.sort();
        self.hashed();
        other.hashed();

        let a = self.as_graph();
        let b = other.as_graph();

        algo::is_isomorphic_matching(&a, &b, |x, y| x == y, |x, y| x == y)
    }

    fn as_graph(&self) -> Graph<Object, Iri> {
        let mut graph = Graph::new();

        for triple in &self.0 {
            let subject = graph.add_node(triple.subject.as_object());
            let object = graph.add_node(triple.object.clone());

            graph.add_edge(subject, object, triple.predicate.clone());
        }

        graph
    }

    fn hashed(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hashed = HashMap::new();

        {
            let subject_blanks = self.0.iter().filter(|t| {
                t.subject.is_blank_node()
            }).map(|t| {
                unwrap_to!(t.subject => Subject::BlankNode).clone()
            }).collect::<Vec<_>>();

            let terminal_blanks = self.0.iter().filter(|t| {
                t.object.is_blank_node() &&
                !subject_blanks.contains(unwrap_to!(t.object => Object::BlankNode))
            });

            for triple in terminal_blanks {
                let key = if triple.object.is_blank_node() {
                    unwrap_to!(triple.object => Object::BlankNode)
                } else {
                    unwrap_to!(triple.subject => Subject::BlankNode)
                };

                hashed.insert(key.clone(), String::from("terminal"));
            }

            let root_blanks = self.0.iter().filter(|t| {
                t.subject.is_blank_node() && !t.object.is_blank_node()
            });

            for (key, group) in &root_blanks.group_by(|t| t.subject.clone()) {
                let mut hash = DefaultHasher::new();

                for triple in group {
                    triple.predicate.to_string().hash(&mut hash);
                    triple.object.to_string().hash(&mut hash);
                }

                hashed.insert(key.into_blank_node().unwrap(), format!("{:x}", hash.finish()));
            }

            while self.0.iter().any(|t| t.subject.is_blank_node() &&
                                    !hashed.contains_key(&t.subject.clone().into_blank_node().unwrap()))
            {
                let mut items = Vec::new();
                {
                    let iter = self.0.iter()
                        .filter(|t| t.subject.is_blank_node() &&
                                t.object.is_blank_node() &&
                                hashed.contains_key(&t.object.clone().into_blank_node().unwrap()))
                        .group_by(|t| t.subject.clone());
                    for (key, group) in iter.into_iter() {
                        let mut hash = DefaultHasher::new();

                        for triple in group {
                            triple.predicate.to_string().hash(&mut hash);
                            hashed[&triple.object.clone().into_blank_node().unwrap()].to_string().hash(&mut hash);
                        }

                        items.push((key.into_blank_node().unwrap(), format!("{:x}", hash.finish())));
                    }
                }

                for (k, v) in items {
                    hashed.insert(k, v);
                }
            }
        }

        for triple in &mut self.0 {
            if triple.subject.is_blank_node() {
                let node = triple.subject.clone().into_blank_node().unwrap();
                triple.subject = Subject::BlankNode(BlankNode(hashed[&node].clone()));
            }

            if triple.object.is_blank_node() {
                let node = triple.object.clone().into_blank_node().unwrap();
                triple.object = Object::BlankNode(BlankNode(hashed[&node].clone()));
            }
        }
    }
}


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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Triple {
    /// The subject of the triple.
    pub subject: Subject,
    /// The predicate of the triple.
    pub predicate: Iri,
    /// The object of the triple.
    pub object: Object
}

impl Triple {
    /// Instantiates a new Triple.
    pub fn new(subject: Subject, predicate: Iri, object: Object) -> Self {
        Triple {
            subject,
            predicate,
            object
        }
    }
}

impl fmt::Display for Triple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} .", self.subject, self.predicate, self.object)
    }
}
