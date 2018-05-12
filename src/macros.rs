macro_rules! rdf {
    ($typ:expr) => {{
        use iri::Iri;
        Iri(concat!("http://www.w3.org/1999/02/22-rdf-syntax-ns#", $typ).into())
    }};
}

macro_rules! xsd {
    ($typ:expr) => {{
        use iri::Iri;
        Iri(concat!("http://www.w3.org/2001/XMLSchema#", $typ).into())
    }};
}

