extern crate chelone;
extern crate url;

use std::env;

use chelone::{Graph, Triples};
use url::Url;

fn main() {
    let url = Url::parse("https://www.w3.org/2013/TurtleTests/").unwrap();
    let mut args = env::args();
    args.next();

    let path = args.next().expect("Expected a file path.");
    println!("{}", read_to_graph(&path, url.clone()));
}

fn read_to_graph(path: &str, base: Url) -> Triples {
    use std::fs::File;
    use std::io::Read;

    let mut input = String::new();

    File::open(path)
        .unwrap()
        .read_to_string(&mut input)
        .unwrap();

    let mut graph = Graph::new(&input).unwrap_or_else(|e| panic!("{}", e));
    graph.set_base(base);
    graph.parse()
}

