extern crate chelone;
extern crate url;

use chelone::{Graph, Triples};
use url::Url;

include!(concat!(env!("OUT_DIR"), "/tests.rs"));

fn read_to_triples(path: &'static str, base: &'static str) -> Triples {
    let input = read_to_string(path);
    let mut graph = Graph::new(&input).unwrap_or_else(|e| panic!("{}", e));
    graph.set_base(Url::parse(base).unwrap());
    graph.parse()
}

fn read_to_string(path: &str) -> String {
    use std::fs::File;
    use std::io::Read;

    let mut input = String::new();

    File::open(path)
        .unwrap()
        .read_to_string(&mut input)
        .unwrap();

    input
}
