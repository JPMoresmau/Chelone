extern crate chelone;

use chelone::Graph;

const TURTLE: &str = r##"
@base <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix rel: <http://www.perceive.net/schemas/relationship/> .

<#Mozilla>
    a foaf:Organization ;
    foaf:name "Mozilla" .

<#Rust>
    rel:childOf <#Mozilla> ;
    a foaf:Project ;
    foaf:name "Rust" .
"##;

fn main() {
    let graph = Graph::new(TURTLE).unwrap_or_else(|e| panic!("{}", e));
    let triples = graph.parse();

    println!("{}", triples);
}
