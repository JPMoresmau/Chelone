macro_rules! rdf {
    ($typ:expr) => {{
        use iri::Iri;

        let raw = concat!("http://www.w3.org/1999/02/22-rdf-syntax-ns#", $typ).into();
        Iri::parse(raw).unwrap()
    }};
}

macro_rules! xsd {
    ($typ:expr) => {{
        use iri::Iri;

        let raw = concat!("http://www.w3.org/2001/XMLSchema#", $typ).into();
        Iri::parse(raw).unwrap()
    }};
}

