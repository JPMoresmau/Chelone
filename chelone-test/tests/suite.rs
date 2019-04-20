extern crate chelone;

use std::fs;

use chelone::{Graph, Triples, Iri};

include!(concat!(env!("OUT_DIR"), "/tests.rs"));

fn read_to_triples(path: &'static str, base: &'static str) -> Triples {
    let input = fs::read_to_string(path).unwrap();
    let mut graph = Graph::new(&input).unwrap_or_else(|e| panic!("{}", e));
    graph.set_base(Iri::parse(base).unwrap());
    graph.parse()
}

fn compare(a: Triples, b: Triples) -> ! {
    if a.len() != b.len() {
        panic!("DIFFERENT LENGTH TRIPLES\nEXPECTED:\n{:#?}\nACTUAL:\n{:#?}", a, b)
    } else {
        let mut output = String::new();
        for (i, (a, b)) in a.iter().zip(b.iter()).enumerate() {
            if a != b {
                output += &format!("\nDIFFERENCE IN {} INDEX:\nEXPECTED:\n{:#?}\nACTUAL:{:#?}\n", i, a, b);
            }
        }

        panic!("{}", output);
    }
}
