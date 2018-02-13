extern crate chelone;
extern crate pest;

use chelone::{TurtleParser, Rule};
use pest::Parser;

fn main() {
    let manifest = {
        use std::fs::File;
        use std::io::Read;

        let mut s = String::new();
        let mut f = File::open("./tests/data/manifest.ttl").unwrap();
        f.read_to_string(&mut s).unwrap();
        s
    };

    let pairs = TurtleParser::parse(Rule::turtleDoc, &manifest)
        .unwrap_or_else(|e| panic!("{}", e));
}
