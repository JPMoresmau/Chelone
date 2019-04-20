#[macro_use] extern crate unwrap_to;
extern crate chelone;
extern crate url;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::error::Error;

use chelone::{Graph, Triples, TripleSearcher};
use chelone::literal::Literal;
use chelone::object::Object;
use chelone::subject::Subject;
use chelone::iri::Iri;
use url::Url;

const BASE_URL: &str = "http://www.w3.org/2013/TurtleTests/";

macro_rules! urls {
    ($($name:ident: $url:expr);+) => {
        $(
            macro_rules! $name {
                ($suffix:expr) => { concat!($name!(), "#", $suffix) };
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

urls! {
    rdf:  "http://www.w3.org/1999/02/22-rdf-syntax-ns";
    rdft: "http://www.w3.org/ns/rdftest";
    mf:   "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest"
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("can't get OUT_DIR");
    let url = {
        use std::fs;
        let mut path = fs::canonicalize("tests/data/manifest.ttl")
            .expect("Couldn't find manifest file.")
            .into_os_string()
            .into_string()
            .expect("Couldn't covert path to string.");
        path.insert_str(0, "file://");

        path
    };

    let rdf_nil_object = Object::Iri(Iri::parse(rdf!("nil")).unwrap());
    let rdf_first = Iri::parse(rdf!("first")).unwrap();
    let rdf_rest = Iri::parse(rdf!("rest")).unwrap();
    let mf_name = Iri::parse(mf!("name")).unwrap();
    let mf_action = Iri::parse(mf!("action")).unwrap();
    let mf_result = Iri::parse(mf!("result")).unwrap();
    let rdf_type = Iri::parse(rdf!("type")).unwrap();
    let triples = read_to_graph("tests/data/manifest.ttl", &url)
        .expect("Couldn't read manifest into graph.");
    let mut entries = Vec::new();
    let mut output = String::new();

    let mf_entries = Iri::parse(mf!("entries")).unwrap();
    eprintln!("{:?}", mf_entries);

    let mut last_node =  TripleSearcher::new()
            .predicate(&mf_entries)
            .execute(&triples)
            .expect("No mf:entries field")
            .object;

    while last_node != rdf_nil_object {
        let subject = last_node.to_subject();
        let entry = TripleSearcher::new()
            .subject(&subject)
            .predicate(&rdf_first)
            .execute(&triples)
            .expect("Couldn't find rdf:nil entry.")
            .object
            .to_subject();

        entries.push(entry);

        last_node = TripleSearcher::new()
            .subject(&subject)
            .predicate(&rdf_rest)
            .execute(&triples)
            .expect("Couldn't get rdf:rest entry")
            .object;
    }

    for entry in entries {
        let rdf_type = TripleSearcher::new()
            .subject(&entry)
            .predicate(&rdf_type)
            .execute(&triples)
            .expect("Couldn't find rdf:type.")
            .object;

        let name = {
            let object = TripleSearcher::new()
                .subject(&entry)
                .predicate(&mf_name)
                .execute(&triples)
                .expect("Couldn't find mf:name entry.")
                .object;

            unwrap_to!(object => Object::Literal).value.replace("-", "_")
        };

        let file = TripleSearcher::new()
            .subject(&entry)
            .predicate(&mf_action)
            .execute(&triples)
            .expect("Couldn't find mf:action")
            .object
            .to_string();

        output += &match &*rdf_type.to_string() {
            wrapped!(rdft, "TestTurtlePositiveSyntax") => format!(r#"
                #[test]
                fn {name}() {{
                    read_to_triples("{file}", "{base}");
                }}
            "#, name = name,
                file = &file[8..file.len() - 1],
                base = BASE_URL),
            wrapped!(rdft, "TestTurtleNegativeEval") |
            wrapped!(rdft, "TestTurtleNegativeSyntax") => format!(r#"
                #[test]
                #[should_panic]
                #[allow(non_snake_case)]
                fn {name}() {{
                    read_to_triples("{file}", "{base}");
                }}
            "#, name = name,
                file = &file[8..file.len() - 1],
                base = BASE_URL),
            wrapped!(rdft, "TestTurtleEval") => {
                let expected = TripleSearcher::new()
                    .subject(&entry)
                    .predicate(&mf_result)
                    .execute(&triples)
                    .expect("Couldn't find mf:result.")
                    .object
                    .to_string();

                format!(r##"
                    #[test]
                    #[allow(non_snake_case)]
                    fn {name}() {{
                        let mut result = read_to_triples("{result}", "{base}");
                        let mut expected = read_to_triples("{expected}", "{base}");

                        if !result.is_isomorphic(&mut expected) {{
                            compare(result, expected);
                        }}
                    }}"##,
                    name = name,
                    result = &file[8..file.len() - 1],
                    expected = &expected[8..expected.len() - 1],
                    base = BASE_URL)
            },

            _ => String::new(),
        };
    }

    File::create(Path::new(&out_dir).join("tests.rs"))
        .expect("Couldn't create tests.rs.")
        .write_all(output.as_bytes())
        .expect("Couldn't write to tests.rs");
}

fn read_to_graph(path: &str, base: &str) -> Result<Triples, Box<Error>> {
    use std::fs;

    let input = fs::read_to_string(path)?;
    let mut graph = Graph::new(&input).unwrap_or_else(|e| panic!("{}", e));

    graph.set_base(Iri::parse(base)?);
    Ok(graph.parse())
}
