//! Output of `Graph::parse`

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::iter::IntoIterator;
use std::collections::HashMap;

use itertools::Itertools;
use petgraph::Graph;

use iri::{BlankNode, Iri};
use object::Object;
use subject::Subject;

/// A set of Triples
#[derive(Clone, Debug, Default)]
pub struct Triples(pub Vec<Triple>);

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

        ::petgraph::algo::is_isomorphic(&a, &b)
    }

    fn as_graph(&self) -> Graph<Object, Iri> {
        let mut graph = Graph::new();

        for triple in &self.0 {
            let subject = graph.add_node(triple.0.as_object());
            let object = graph.add_node(triple.2.clone());

            graph.add_edge(subject, object, triple.1.clone());
        }

        graph
    }

    fn hashed(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hashed = Box::new(HashMap::new());

        {
            let root_blanks = self.0.iter()
                .filter(|t| t.0.is_blank_node() && !t.2.is_blank_node());

            for (key, group) in &root_blanks.group_by(|s| s.0.clone()) {
                let mut hash = DefaultHasher::new();

                for triple in group {
                    triple.1.to_string().hash(&mut hash);
                    triple.2.to_string().hash(&mut hash);
                }

                hashed.insert(key.into_blank_node().unwrap(), format!("{:x}", hash.finish()));
            }

            while self.0.iter().any(|t| t.0.is_blank_node() &&
                                    !hashed.contains_key(&t.0.clone().into_blank_node().unwrap()))
            {
                let mut items = Vec::new();
                {
                    let iter = self.0.iter()
                        .filter(|t| t.0.is_blank_node() &&
                                t.2.is_blank_node() &&
                                hashed.contains_key(&t.2.clone().into_blank_node().unwrap()))
                        .group_by(|t| t.0.clone());
                    for (key, group) in iter.into_iter() {
                        let mut hash = DefaultHasher::new();

                        for triple in group {
                            triple.1.to_string().hash(&mut hash);
                            hashed[&triple.2.clone().into_blank_node().unwrap()].to_string().hash(&mut hash);
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
            if triple.0.is_blank_node() {
                let node = triple.0.clone().into_blank_node().unwrap();
                triple.0 = Subject::BlankNode(BlankNode(hashed[&node].clone()));
            }

            if triple.2.is_blank_node() {
                let node = triple.2.clone().into_blank_node().unwrap();
                triple.2 = Object::BlankNode(BlankNode(hashed[&node].clone()));
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
pub struct Triple(pub Subject, pub Iri, pub Object);

impl Triple {

    fn blank_subject(&self) -> bool {
        self.0.is_blank_node()
    }

    fn contains_blank_nodes(&self) -> bool {
        self.blank_subject() || self.2.is_blank_node()
    }

    fn only_grounded_nodes(&self) -> bool {
        !self.contains_blank_nodes()
    }
}

impl fmt::Display for Triple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} .", self.0, self.1, self.2)
    }
}
