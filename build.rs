extern crate chelone;
extern crate url;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use chelone::{Graph, Triples};
use chelone::literal::Literal;
use chelone::object::Object;
use url::Url;

macro_rules! urls {
    ($($name:ident: $url:expr);+) => {
        $(
            macro_rules! $name {
                ($suffix:expr) => { concat!($name!(), $suffix) };
                () => { $url }
            }
        )+
    }
}

macro_rules! wrapped {
    ($name:ident $(, $suffix:expr)*) => {
        concat!("<", $name!($($suffix)*), ">")
    }
}

urls!{
    rdf:   "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    rdfs:  "http://www.w3.org/2000/01/rdf-schema#";
    rdft:   "http://www.w3.org/ns/rdftest#";
    mf:    "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#";
    qt:    "http://www.w3.org/2001/sw/DataAccess/tests/test-query#"
}


fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("can't get OUT_DIR");
    let base_url = {
        use std::fs;
        let mut path = fs::canonicalize("tests/data/manifest.ttl").unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
        path.insert_str(0, "file://");
        path
    };

    let url = Url::parse(&base_url).unwrap();
    let triples = read_to_graph("tests/data/manifest.ttl", url);
    let mut entries = Vec::new();
    let mut output = String::new();

    let mut last_node = triples.iter().find(|i| {
        i.0.to_string().contains(&base_url) && (i.1).0 == mf!("entries")
    }).expect("No entries field").2.to_string();

    while last_node != wrapped!(rdf, "nil") {
        let ref entry = triples.iter().find(|i| {
            i.0.to_string() == last_node &&
            i.1.to_string() == wrapped!(rdf, "first")
        }).unwrap().2;

        entries.push(entry);

        last_node = triples.iter().find(|i| {
            i.0.to_string() == last_node &&
            i.1.to_string() == wrapped!(rdf, "rest")
        }).unwrap().2.to_string();
    }

    for entry in entries {
        let ref rdf_type = triples.iter().find(|i| {
            i.0.to_string() == entry.to_string() &&
            i.1.to_string() == wrapped!(rdf, "type")
        }).unwrap().2;

        let ref name = triples.iter().find(|i| {
            i.0.to_string() == entry.to_string() &&
            i.1.to_string() == wrapped!(mf, "name")
        }).unwrap().2;

        let name = match *name {
            Object::Literal(Literal::RdfLiteral {
                ref value,
                language_tag: _,
                iri: _
            }) => value,
            _ => unreachable!(),
        }.replace("-", "_");

        let file = triples.iter().find(|i| {
            i.0.to_string() == entry.to_string() &&
            i.1.to_string() == wrapped!(mf, "action")
        }).unwrap().2.to_string();

        output += &match &*rdf_type.to_string() {
            wrapped!(rdft, "TestTurtlePositiveSyntax") => format!(r#"
                #[test]
                fn {name}() {{
                    let input = read_to_string("{file}");
                    let mut graph = Graph::new(&input).unwrap_or_else(|e| panic!("{{}}", e));
                    graph.set_base(Url::parse("{base}").unwrap());
                    graph.parse();

                }}
            "#, name = name,
                file = &file[8..file.len() - 1],
                base = &base_url),

            _ => String::new(),
        };
    }

    File::create(Path::new(&out_dir).join("tests.rs"))
        .unwrap()
        .write_all(output.as_bytes())
        .unwrap();
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

