#![allow(bad_style)]

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TurtleParser;

impl Rule {
    pub fn is_string_value(self) -> bool {
        match self {
            Rule::STRING_VALUE |
            Rule::SINGLE_STRING_VALUE |
            Rule::SINGLE_LONG_STRING_VALUE |
            Rule::LONG_STRING_VALUE => true,
            _ => false,
        }
    }
}
