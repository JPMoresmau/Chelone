extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("grammar.pest"); // relative to this file


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TurtleParser;
