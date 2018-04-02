extern crate chelone;
extern crate url;

use chelone::Graph;
use url::Url;

include!(concat!(env!("OUT_DIR"), "/tests.rs"));

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
