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

#![deny(missing_docs)]

#[macro_use] extern crate pest_derive;
#[macro_use] extern crate unwrap_to;
extern crate pest;
extern crate url;
extern crate itertools;
extern crate petgraph;

#[macro_use] mod macros;
mod parser;
pub mod iri;
pub mod literal;
pub mod object;
pub mod subject;
pub mod triple;

use std::collections::HashMap;
use std::iter::Peekable;
use std::fmt;

use pest::Parser;
use pest::iterators::FlatPairs;

use literal::Literal;
use object::Object;
use parser::{Rule, TurtleParser};
use subject::Subject;

pub use iri::{BlankNode, Iri};
pub use triple::{Triple, Triples, TripleSearcher};

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("grammar.pest");
const TYPE_PREDICATE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

macro_rules! get {
    ($this:ident: $rule:expr) => {{
        use Rule::*;
        let next = $this.input.next()?;
        // println!("{:?}", next.as_rule());
        assert_eq!(next.as_rule(), $rule);
        next
    }}
}

/// Graph parser.
pub struct Graph<'a> {
    input: Peekable<FlatPairs<'a, Rule>>,
    base: Option<Iri>,
    blank_node_counter: usize,
    prefixs: HashMap<String, Iri>,
    subject: Option<Subject>,
    predicate: Option<Iri>,
    subject_stack: Vec<Subject>,
    predicate_stack: Vec<Iri>,
    triples: Triples,
    _source: &'a str
}

impl<'a> Graph<'a> {
    /// Creates a new `Graph` from the turtle source.
    pub fn new(_source: &'a str) -> Result<Self, pest::error::Error<Rule>> {
        let parsed = TurtleParser::parse(Rule::turtleDoc, _source)?;
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
            _source
        })
    }

    fn _debug_input(input: Peekable<FlatPairs<'a, Rule>>) {
        for pair in input {
            println!("RULE: {:?} STR: {:?}", pair.as_rule(), pair.as_str());
        }
    }

    /// Sets the initial base url to resolve relative urls against.
    pub fn set_base(&mut self, iri: Iri) {
        self.base = Some(iri)
    }

    /// Parse graph into a set of Triples.
    pub fn parse(mut self) -> Triples {
        self.take();

        while let Some(_) = self.input.peek() {
            if self.input.peek().map(|x| x.as_rule() == Rule::EOI).unwrap() ||
               self.parse_statement().is_none()
            {
                break
            }
        }

        self.triples
    }

    fn parse_statement(&mut self) -> Option<()> {
        get!(self: statement);
        let rule = self.input.peek()?.as_rule();
        let text = self.input.peek()?.as_str();

        match rule {
            Rule::directive => self.parse_directive(),
            Rule::triples => self.parse_triples(),
            _ => self.unreachable(rule, text),
        }
    }

    fn parse_directive(&mut self) -> Option<()> {
        get!(self: directive);

        let pair = self.input.next()?;
        let rule = pair.as_rule();

        match rule {
            Rule::prefixID | Rule::sparqlPrefix => {
                let key = self.input.next()?.as_str().replace(':', "");
                let value = self.parse_iriref()?;
                self.prefixs.insert(key, value);
            }

            Rule::base | Rule::sparqlBase => {
                self.base = Some(self.parse_iriref()?);
            },

            _ => unreachable!(),
        }

        Some(())
    }

    fn parse_triples(&mut self) -> Option<()> {
        get!(self: triples);

        match self.input.peek()?.as_rule() {
            Rule::subject => {
                self.parse_subject()?;
                self.parse_predicate_object_list()?;
            },

            Rule::blankNodePropertyList => {
                let node = self.parse_blank_node_property_list()?;

                self.subject = Some(Subject::BlankNode(node));
                if self.input.peek()?.as_rule() == Rule::predicateObjectList {
                    self.parse_predicate_object_list()?;
                }
            }

            _ => unreachable!(),
        }
        Some(())
    }

    fn parse_predicate_object_list(&mut self) -> Option<()> {
        let next = get!(self: predicateObjectList);
        let end = next.into_span().end();

        while self.belongs_to_list(Rule::verb, end) {
            self.predicate = Some(self.parse_verb()?);
            self.parse_object_list()?;
        }

        Some(())
    }

    fn parse_verb(&mut self) -> Option<Iri> {
        let next = get!(self: verb);
        if next.as_str() == "a" {
            Iri::parse(TYPE_PREDICATE).ok()
        } else {
            self.parse_iri()
        }
    }

    fn parse_subject(&mut self) -> Option<()> {
        get!(self: subject);

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
        get!(self: BlankNode);

        let node = match self.input.peek()?.as_rule() {
            Rule::BLANK_NODE_LABEL => {
                get!(self: BLANK_NODE_LABEL);
                BlankNode(String::from(self.input.next()?.as_str()))
            },
            Rule::ANON => {
                get!(self: ANON);
                self.generate_new_blank_node()
            },
            r => unreachable!("unexpected: {:?}", r),
        };

        Some(node)
    }

    fn parse_object_list(&mut self) -> Option<()> {
        let end = get!(self: objectList).into_span().end();

        while self.belongs_to_list(Rule::object, end) {
            self.parse_object()?;
        }

        Some(())
    }

    fn parse_object(&mut self) -> Option<()> {
        get!(self: object);

        let object = match self.input.peek()?.as_rule() {
            Rule::iri => Object::Iri(self.parse_iri()?),
            Rule::literal => Object::Literal(self.parse_literal()?),
            Rule::BlankNode => Object::BlankNode(self.parse_blank_node()?),
            Rule::collection => self.parse_collection()?,
            Rule::blankNodePropertyList => {
                Object::BlankNode(self.parse_blank_node_property_list()?)
            }
            _ => unreachable!(),
        };

        self.emit_triple(object);
        Some(())
    }

    fn parse_collection(&mut self) -> Option<Object> {
        self.save_subject();
        self.save_predicate();

        let end = get!(self: collection).into_span().end();
        let mut node = self.generate_new_blank_node();

        if !self.belongs_to_list(Rule::object, end) {
            self.pop_subject();
            self.pop_predicate();

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

    fn parse_blank_node_property_list(&mut self) -> Option<BlankNode> {
        get!(self: blankNodePropertyList);

        let new_node = self.generate_new_blank_node();

        self.save_subject();
        self.subject = Some(Subject::BlankNode(new_node.clone()));
        self.save_predicate();

        self.parse_predicate_object_list()?;
        self.pop_subject();
        self.pop_predicate();

        Some(new_node)
    }

    fn parse_literal(&mut self) -> Option<Literal> {
        get!(self: literal);

        match self.input.peek()?.as_rule() {
            Rule::RDFLiteral => self.parse_rdf_literal(),
            Rule::NumericLiteral => self.parse_numeric_literal(),
            Rule::BooleanLiteral => self.parse_bool_literal(),
            _ => unreachable!(),
        }
    }

    fn parse_rdf_literal(&mut self) -> Option<Literal> {
        get!(self: RDFLiteral);

        let value = self.parse_string()?;
        Some(Literal::new(value, self.parse_langtag(), self.parse_datatype()))
    }

    fn parse_numeric_literal(&mut self) -> Option<Literal> {
        get!(self: NumericLiteral);

        let pair = self.input.next()?;
        let mut value = String::from(pair.as_str());

        Some(match pair.as_rule() {
            Rule::INTEGER => Literal::new_integer(value),
            Rule::DECIMAL => Literal::new_decimal(value),
            Rule::DOUBLE => {
                if self.input.peek().map(|p| p.as_rule()) == Some(Rule::EXPONENT)
                {
                    value.push_str(pair.as_str());
                    self.take();
                }

                Literal::new_double(value)
            }
            _ => unreachable!(),
        })
    }

    fn parse_bool_literal(&mut self) -> Option<Literal> {
        Some(Literal::new_bool(String::from(self.input.next()?.as_str())))
    }

    fn parse_string(&mut self) -> Option<String> {
        get!(self: STRING);
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
            "\\n" => '\n',
            "\\r" => '\r',
            "\\f" => '\u{0C}',
            "\\'" => '\'',
            "\\\"" => '\"',
            "\\\\" => '\\',
            c => unreachable!("Unexpected {:?}", c),
        })
    }

    fn parse_uchar(&mut self) -> Option<char> {
        get!(self: UCHAR);
        let mut hex = 0;

        while self.input.peek()?.as_rule() == Rule::HEX {
            let value = self.input.next()?;

            hex <<= 4;
            hex |= value.as_str().chars().next()?.to_digit(16)?;
        }

        std::char::from_u32(hex)
    }

    fn parse_iri(&mut self) -> Option<Iri> {
        get!(self: iri);
        let iri = match self.input.peek()?.as_rule() {
            Rule::PrefixedName => self.parse_prefixed_name(),
            Rule::IRIREF => self.parse_iriref(),
            _ => unreachable!(),
        };

        iri
    }

    fn parse_iriref(&mut self) -> Option<Iri> {

        let end = self.input.next().unwrap().into_span().end();
        let mut next_start = self.input.peek()?.clone().into_span().start();
        let mut iriref = String::new();

        if next_start > end {
            return self.base.clone()
        }

        while next_start < end {
            match self.input.peek()?.as_rule() {
                Rule::IRI_VALUE => iriref.push_str(self.input.next()?.as_str()),
                Rule::UCHAR => iriref.push(self.parse_uchar().unwrap()),
                _ => unreachable!(),
            }

            next_start = if let Some(peek) = self.input.peek(){
                peek.clone().into_span().start()
            } else {
                break
            };
        };

        Iri::parse_with_base_iri(&iriref, self.base.as_ref()).ok()
    }

    fn parse_prefixed_name(&mut self) -> Option<Iri> {
        get!(self: PrefixedName);

        let rule = self.input.peek()?.as_rule();

        match rule {
            Rule::PNAME_LN => self.parse_pname_ln(),
            Rule::PNAME_NS => self.parse_pname_ns(),
            _ => unreachable!(),
        }
    }

    fn parse_pname_ln(&mut self) -> Option<Iri> {
        get!(self: PNAME_LN);

        let base = self.parse_pname_ns()?;
        let mut base = base.as_str().to_owned();
        // Replace should work here as the escapes are validated in pest.
        let pn_local = get!(self: PN_LOCAL).as_str().replace('\\', "");

        base.push_str(&pn_local);

        Iri::parse(&base).ok()
    }

    fn parse_pname_ns(&mut self) -> Option<Iri> {
        let (_next_rule, next) = {
            let next = self.input.next()?;

            (next.as_rule(), next.as_str())
        };

        let prefix = next.replace(":", "");
        Some(self.prefixs[&prefix].clone())
    }

    fn emit_triple(&mut self, object: Object) -> Option<()> {
        let subject = self.subject.clone().expect("No Subject found");
        let predicate = self.predicate.clone().expect("No Predicate found");

        self.triples.push(Triple::new(subject, predicate, object));

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
        let _x = self.input.next();
        //println!("{:?}", _x.map(|x| x.as_rule()));
    }

    fn unreachable(&mut self, rule: Rule, text: &str) -> ! {
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
