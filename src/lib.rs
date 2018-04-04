//! # Chelone: A Turtle-RDF parsing library.
//! ```
//! extern crate chelone;
//!
//! use chelone::Graph;
//!
//! const TURTLE: &str = r##"
//! @base <http://example.org/> .
//! @prefix foaf: <http://xmlns.com/foaf/0.1/> .
//! @prefix rel: <http://www.perceive.net/schemas/relationship/> .
//!
//! <#Mozilla>
//!     a foaf:Organization ;
//!     foaf:name "Mozilla" .
//!
//! <#Rust>
//!     rel:childOf <#Mozilla> ;
//!     a foaf:Project ;
//!     foaf:name "Rust" .
//! "##;
//!
//! fn main() {
//!     let graph = Graph::new(TURTLE).unwrap_or_else(|e| panic!("{}", e));
//!     let triples = graph.parse();
//!
//!     println!("{}", triples);
//! }
//! ```

#![cfg_attr(test, deny(missing_docs))]
#![deny(missing_docs)]

#[macro_use] extern crate pest_derive;
extern crate pest;
extern crate url;
extern crate itertools;
extern crate petgraph;

#[macro_use] pub mod literal;
pub mod iri;
pub mod object;
mod parser;
pub mod subject;
pub mod triple;

use std::collections::HashMap;
use std::iter::Peekable;
use std::fmt;

use pest::Parser;
use pest::iterators::FlatPairs;
use url::Url;

use iri::{BlankNode, Iri};
use literal::Literal;
use object::Object;
use parser::{Rule, TurtleParser};
use subject::Subject;

pub use triple::{Triple, Triples};

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("grammar.pest");
const TYPE_PREDICATE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// Graph parser.
pub struct Graph<'a> {
    input: Peekable<FlatPairs<'a, Rule>>,
    base: Option<Url>,
    blank_node_counter: usize,
    prefixs: HashMap<String, Iri>,
    subject: Option<Subject>,
    predicate: Option<Iri>,
    subject_stack: Vec<Subject>,
    predicate_stack: Vec<Iri>,
    triples: Triples,
    source: &'a str
}

impl<'a> Graph<'a> {
    /// Creates a new `Graph` from the turtle source.
    pub fn new(source: &'a str) -> Result<Self, pest::Error<Rule>> {
        let parsed = TurtleParser::parse(Rule::turtleDoc, source)?;
        let input = parsed.flatten().peekable();

        Ok(Graph {
            input,
            base: Option::default(),
            blank_node_counter: usize::default(),
            prefixs: HashMap::default(),
            subject: Option::default(),
            predicate: Option::default(),
            subject_stack: Vec::default(),
            predicate_stack: Vec::default(),
            triples: Triples::default(),
            source
        })
    }

    /// Sets the initial base url to resolve relative urls against.
    pub fn set_base(&mut self, url: Url) {
        self.base = Some(url)
    }

    /// Parse graph into a set of Triples.
    pub fn parse(mut self) -> Triples {
        if self.input.peek().is_none() {
            return self.triples
        }

        self.take();
        while let Some(_) = self.input.peek() {
            if self.parse_statement().is_none() {
                break
            }
        }

        self.triples
    }

    fn parse_statement(&mut self) -> Option<()> {
        self.take();
        let rule = self.input.peek()?.as_rule();
        let text = self.input.peek()?.as_str();

        match rule {
            Rule::directive => self.parse_directive(),
            Rule::triples => self.parse_triples(),
            _ => self.unreachable(rule, text),
        }
    }

    fn parse_directive(&mut self) -> Option<()> {
        self.take();

        let pair = self.input.next()?;
        let rule = pair.as_rule();

        match rule {
            Rule::prefixID | Rule::sparqlPrefix => {
                let name = self.input.next()?;
                let key = if name.as_str() == ":" {
                    String::new()
                } else {
                    let prefix = self.input.next()?;
                    String::from(prefix.as_str())
                };

                let value = self.parse_iriref()?;
                self.prefixs.insert(key, value);
            }

            Rule::base | Rule::sparqlBase => {
                self.base = Some(Url::parse(&self.parse_iriref()?).unwrap());
            },

            _ => unreachable!(),
        }

        Some(())
    }

    fn parse_triples(&mut self) -> Option<()> {
        self.take();

        match self.input.peek()?.as_rule() {
            Rule::subject => {
                self.parse_subject()?;
                self.parse_predicate_object_list()?;
            },

            Rule::blankNodePropertyList => {
                self.parse_blank_node_property_list()?;

                if self.input.peek()?.as_rule() == Rule::predicateObjectList {
                    self.parse_predicate_object_list()?;
                }
            }

            _ => unreachable!(),
        }
        Some(())
    }

    fn parse_predicate_object_list(&mut self) -> Option<()> {
        let end_of_list = self.input.next()?.into_span().end();

        while self.belongs_to_list(Rule::verb, end_of_list) {
            self.predicate = Some(self.parse_verb()?);
            self.parse_object_list()?;
        }

        Some(())
    }

    fn parse_verb(&mut self) -> Option<Iri> {
        if self.input.next()?.as_str() == "a" {
            Some(Iri(String::from(TYPE_PREDICATE)))
        } else {
            self.parse_iri()
        }
    }

    fn parse_subject(&mut self) -> Option<()> {
        self.take();

        let subject = match self.input.peek()?.as_rule() {
            Rule::iri => Subject::Iri(self.parse_iri()?),
            Rule::BlankNode => Subject::BlankNode(self.parse_blank_node()?),
            Rule::collection => self.parse_collection()?.to_subject(),
            _ => unreachable!(),
        };

        self.subject = Some(subject);

        Some(())
    }

    fn parse_blank_node(&mut self) -> Option<BlankNode> {
        self.take();

        let node = match self.input.peek()?.as_rule() {
            Rule::BLANK_NODE_LABEL => {
                self.take();
                BlankNode(String::from(self.input.next()?.as_str()))
            },
            Rule::ANON => {
                self.take();
                self.generate_new_blank_node()
            },
            r => unreachable!("unexpected: {:?}", r),
        };

        Some(node)
    }

    fn parse_object_list(&mut self) -> Option<()> {
        let end_of_list = self.input.next()?.into_span().end();

        while self.belongs_to_list(Rule::object, end_of_list) {
            self.parse_object()?;
        }

        Some(())
    }

    fn parse_object(&mut self) -> Option<()> {
        self.take();

        match self.input.peek()?.as_rule() {
            r @ Rule::iri |
            r @ Rule::literal |
            r @ Rule::BlankNode |
            r @ Rule::collection =>
            {
                let object = match r {
                    Rule::iri => Object::Iri(self.parse_iri()?),
                    Rule::literal => Object::Literal(self.parse_literal()?),
                    Rule::BlankNode => Object::BlankNode(self.parse_blank_node()?),
                    Rule::collection => self.parse_collection()?,
                    _ => unreachable!(),
                };

                self.emit_triple(object);
            },
            Rule::blankNodePropertyList => self.parse_blank_node_property_list()?,

            _ => unreachable!(),
        }

        Some(())
    }

    fn parse_collection(&mut self) -> Option<Object> {
        self.save_subject();
        self.save_predicate();

        let end = self.input.next().unwrap().into_span().end();
        let mut node = self.generate_new_blank_node();

        if !self.belongs_to_list(Rule::object, end) {
            Some(Object::Iri(rdf!("nil")))
        } else {
            self.subject = Some(Subject::BlankNode(node.clone()));
            self.predicate = Some(rdf!("rest"));
            self.emit_triple(Object::Iri(rdf!("nil")));
            self.predicate = Some(rdf!("first"));
            self.parse_object().unwrap();

            while self.belongs_to_list(Rule::object, end) {
                let new_node = self.generate_new_blank_node();
                self.subject = Some(Subject::BlankNode(new_node.clone()));
                self.predicate = Some(rdf!("first"));
                self.parse_object().unwrap();

                self.predicate = Some(rdf!("rest"));
                self.emit_triple(Object::BlankNode(node.clone())).unwrap();
                node = new_node;
            }

            self.pop_subject();
            self.pop_predicate();

            Some(Object::BlankNode(node))
        }
    }

    fn parse_blank_node_property_list(&mut self) -> Option<()> {
        self.take();

        self.save_subject();
        self.subject = Some(Subject::BlankNode(self.generate_new_blank_node()));
        self.save_predicate();

        self.parse_predicate_object_list()?;
        self.pop_subject();
        self.pop_predicate();

        Some(())
    }

    fn parse_literal(&mut self) -> Option<Literal> {
        self.take();

        match self.input.peek()?.as_rule() {
            Rule::RDFLiteral => self.parse_rdf_literal(),
            Rule::NumericLiteral => self.parse_numeric_literal(),
            Rule::BooleanLiteral => self.parse_bool_literal(),
            _ => unreachable!(),
        }
    }

    fn parse_rdf_literal(&mut self) -> Option<Literal> {
        self.take();

        let value = self.parse_string()?;

        Some(Literal::RdfLiteral {
            value,
            language_tag: self.parse_langtag(),
            iri: self.parse_datatype(),
        })
    }

    fn parse_numeric_literal(&mut self) -> Option<Literal> {
        self.take();

        let pair = self.input.next()?;
        let mut value = String::from(pair.as_str());

        Some(match pair.as_rule() {
            Rule::INTEGER => Literal::Integer(value),
            Rule::DECIMAL => Literal::Decimal(value),
            Rule::DOUBLE => {
                if self.input.peek().map(|p| p.as_rule()) == Some(Rule::EXPONENT)
                {
                    value.push_str(pair.as_str());
                    self.take();
                }

                Literal::Double(value)
            }
            _ => unreachable!(),
        })
    }

    fn parse_bool_literal(&mut self) -> Option<Literal> {
        Some(Literal::Bool(String::from(self.input.next()?.as_str())))
    }

    fn parse_string(&mut self) -> Option<String> {
        self.take();
        // because we don't care about which quote syntax was used.
        self.take();

        let mut string = String::new();

        while self.input.peek()?.as_rule().is_string_value() {
            let value = self.input.next()?;
            let (peek_rule, peek_str) = {
                let peek = self.input.peek()?;

                (peek.as_rule(), peek.as_str())
            };

            if value.as_str() == peek_str {
                match peek_rule {
                    Rule::ECHAR => string.push(self.parse_echar()?),
                    Rule::UCHAR => string.push(self.parse_uchar()?),
                    _ => string.push_str(peek_str),
                }
            } else {
                string.push_str(value.as_str());
            }

        }

        Some(string)
    }

    fn parse_langtag(&mut self) -> Option<String> {
        if self.input.peek()?.as_rule() == Rule::LANGTAG {
            Some(self.input.next()?.as_str().replace("@", ""))
        } else {
            None
        }
    }

    fn parse_datatype(&mut self) -> Option<Iri> {
        if self.input.peek()?.as_rule() == Rule::iri {
            self.parse_iri()
        } else {
            None
        }
    }

    fn parse_echar(&mut self) -> Option<char> {
        Some(match self.input.next()?.as_str() {
            "\\t" => '\t',
            "\\b" => '\u{08}',
            "\\n" => '\t',
            "\\r" => '\t',
            "\\f" => '\u{0C}',
            "\\'" => '\'',
            "\\\"" => '\'',
            "\\\\" => '\\',
            c => unreachable!("Unexpected {:?}", c),
        })
    }

    fn parse_uchar(&mut self) -> Option<char> {
        self.take();
        let mut hex = 0;

        while self.input.peek()?.as_rule() == Rule::HEX {
            let value = self.input.next()?;

            hex <<= 4;
            hex |= value.as_str().chars().next()?.to_digit(16)?;
        }

        std::char::from_u32(hex)
    }

    fn parse_iri(&mut self) -> Option<Iri> {
        self.take();
        match self.input.peek()?.as_rule() {
            Rule::PrefixedName => self.parse_prefixed_name(),
            Rule::IRIREF => self.parse_iriref(),
            _ => unreachable!(),
        }
    }

    fn parse_iriref(&mut self) -> Option<Iri> {
        let end = self.input.next().unwrap().into_span().end();
        let rule = self.input.peek()?.as_rule();

        let value = if rule == Rule::IRI_VALUE {
            let mut value = self.input.next()?.as_str().to_owned();
            let mut peek = self.input.peek().cloned();

            while peek.is_some() && end > peek.unwrap().into_span().start() {
                value.push(self.parse_uchar().unwrap());
                peek = self.input.peek().cloned();
            }

            let url = Url::options()
                .base_url(self.base.as_ref())
                .parse(value.as_str())
                .unwrap();

            Iri(String::from(url.as_str()))
        } else {
            self.base.clone().map_or_else(|| Iri(String::new()), |u| {
                Iri(String::from(u.as_str()))
            })
        };

        Some(value)
    }

    fn parse_prefixed_name(&mut self) -> Option<Iri> {
        self.take();

        let rule = self.input.peek()?.as_rule();

        match rule {
            Rule::PNAME_LN => self.parse_pname_ln(),
            Rule::PNAME_NS => self.parse_pname_ns(),
            _ => unreachable!(),
        }
    }

    fn parse_pname_ln(&mut self) -> Option<Iri> {
        self.take();

        let mut iri = self.parse_pname_ns()?;
        let pn_local_end = self.input.next().unwrap().into_span().end();
        let mut local = String::new();
        let mut peek = self.input.peek().cloned();

        while let Some(pair) = peek {
            let (start, rule, value) = {
                let value = pair.as_str();
                let rule = pair.as_rule();
                let start = pair.into_span().start();

                (start, rule, value)
            };

            if start < pn_local_end {
                if rule == Rule::PN_LOCAL_ESC {
                    local.push_str(&value.replace("\\", ""));
                } else {
                    local.push_str(value);
                }

                self.take();
                peek = self.input.peek().cloned();
            } else {
                break
            }
        }

        iri.push_str(&local);

        Some(iri)
    }

    fn parse_pname_ns(&mut self) -> Option<Iri> {
        let (next_rule, next) = {
            let next = self.input.next()?;

            (next.as_rule(), next.as_str())
        };

        println!("RULE: {:?}; STR: {:?}", next_rule, next);
        let prefix = if next == ":" {
            ""
        } else {
            self.input.next()?.as_str()
        };

        println!("{:#?}", self);
        println!("FAILING PREFIX: {:#?}", prefix);
        Some(self.prefixs[prefix].clone())
    }

    fn emit_triple(&mut self, object: Object) -> Option<()> {
        let subject = self.subject.clone()?;
        let predicate = self.predicate.clone()?;

        self.triples.push(Triple(subject, predicate, object));

        Some(())
    }

    fn belongs_to_list(&mut self, rule: Rule, end: usize) -> bool {
        if let Some(peek) = self.input.peek() {
            peek.as_rule() == rule && end > peek.clone().into_span().start()
        } else {
            false
        }
    }

    fn save_subject(&mut self) {
        if let Some(iri) = self.subject.take() {
            self.subject_stack.push(iri);
        }
    }

    fn save_predicate(&mut self) {
        if let Some(iri) = self.predicate.take() {
            self.predicate_stack.push(iri);
        }
    }

    fn pop_subject(&mut self) {
        self.subject = self.subject_stack.pop();
    }

    fn pop_predicate(&mut self) {
        self.predicate = self.predicate_stack.pop();
    }

    fn generate_new_blank_node(&mut self) -> BlankNode {
        self.blank_node_counter += 1;
        let label = format!("b{}", self.blank_node_counter);


        if self.prefixs.get(&label).is_none() {
            BlankNode(label)
        } else {
            self.generate_new_blank_node()
        }
    }

    fn take(&mut self) {
        self.input.next().unwrap();
    }

    fn unreachable(&mut self, rule: Rule, text: &str) -> ! {
        println!("Source:\n\"\"\"\n{}\n\"\"\"", self.source);
        println!("Parser state:\n{:#?}", self);

        unreachable!("Unexpected {:?}: {:?}", rule, text)
    }
}

impl<'a> fmt::Debug for Graph<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Graph")
            .field("input", &"#hidden#")
            .field("base", &self.base)
            .field("prefixs", &self.prefixs)
            .field("subject", &self.subject)
            .field("predicate", &self.predicate)
            .field("subject_stack", &self.subject_stack)
            .field("predicate_stack", &self.predicate_stack)
            .field("triples", &self.triples)
            .finish()
    }
}
