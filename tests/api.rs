extern crate mockito;

use mockito::mock;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn slurp(file_path: &str) -> String {
    let p = Path::new(file_path);
    let mut string = String::new();
    let mut file = File::open(&p).unwrap();

    file.read_to_string(&mut string).unwrap();
    string
}

#[test]
fn pull_data() {
    let issue_1 = slurp("tests/fixtures/issue1_response.json");
    let versions = slurp("tests/fixtures/versions_response.json");
    mock("GET", "rest/api/2/issue/EX-2")
        .match_header("content-type", "application/json")
        .with_body(&issue_1);
    mock("GET", "rest/api/2/project/EX/versions")
        .match_header("content-type", "application/json")
        .with_body(&versions);
    mock("POST", "rest/api/2/issue/EX-2")
        .match_header("content-type", "application/json")
        .with_status(204);

}
