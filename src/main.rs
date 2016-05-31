extern crate regex;
extern crate hyper;
extern crate rustc_serialize;

use std::process::Command;
use regex::Regex;
use hyper::{Client, Url};
use std::io::Read;
use rustc_serialize::json::Json;
mod parameters;
use parameters::Params;

// Returns the git log diff or the latest and release branches.
fn git_logs(params: &Params) -> Result<String, String> {
    let mut cmd = Command::new("git");
    let release_ptr = String::from("^") + (&params.release_branch);
    cmd
        .arg("log")
        .arg(&params.latest_branch)
        .arg(&release_ptr)
        .arg("--no-merges")
        .arg("--pretty=%s");

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map_err(|_| "Could not parse git command stdout".to_owned())
            } else {
                match String::from_utf8(output.stderr) {
                    Ok(e) => {
                        Err("Error running git command".to_owned() + (&e))
                    },
                    Err(_) => {
                        Err("Error running git command: cannot parse stderr".to_owned())
                    }
                }
            }
        },
        Err(_) => {
            Err("Git command returned an error".to_owned())
        }
    }
}

fn parse_jira_identifiers(params: &Params, logs: String) -> Vec<String> {
    let mut parsed: Vec<String> = Vec::new();
    let start_reg = String::from("(?i)^[ ]*\\[?[ ]*");
    let end_reg = "(-| )(?P<num>[0-9]*)\\]?";
    let end_reg_ref: &str = &end_reg;
    let upper_id = params.project_id.to_uppercase();
    let upper_id_ref: &str = &upper_id;
    let reg_str: String = start_reg + upper_id_ref + end_reg_ref;
    let reg = Regex::new(&reg_str).unwrap();
    for line in logs.lines() {
        if let Some(capture) = reg.captures(line) {
            let num = capture.name("num").unwrap();
            let iden = upper_id.clone() + "-" + &num;
            parsed.push(iden);
        }
    }
    parsed
}

struct JiraIssue {
    pub json: Json,
    base_url: Url
}

impl JiraIssue {

    // fetch from the issue tag
    pub fn from_tag(client: &Client, params: &Params, tag: String) -> Result<JiraIssue, String> {
        let baseUrl = match Url::parse(&params.url) {
            Ok(u) => u,
            Err(e) => return Err("Error parsing url".to_owned())
        };
        let apiPart = String::from("/rest/api/2/issue") + (&tag);
        let url = baseUrl.join(&apiPart).unwrap();
        match client.get(url).send() {
            Ok(mut res) => {
                use hyper::status::StatusCode;
                use hyper::status::StatusCode::{Created, Accepted};
                let body: String = match res.status {
                    StatusCode::Ok | Created | Accepted => {
                        let mut b = String::new();
                        res.read_to_string(&mut b).unwrap();
                        b
                    },
                    // TODO: not exist should be a special case?
                    _ => {
                        return Err("Error processing request".to_owned())
                    }
                };
                if let Ok(json) = Json::from_str(&body) {
                    Ok(JiraIssue {
                        json: json,
                        base_url: baseUrl
                    })
                } else {
                    Err("Error parsing json response".to_owned())
                }
            },
            Err(_) => {
                Err("Error connecting to Jira site".to_owned())
            }
        }
    }
    // put request
    fn update(&self, client: &Client) -> Result<JiraIssue, String> {
        unimplemented!();
    }
}

#[test]
fn test_jira_parser() {
    let mock_logs = "[FOO-123] hello world!\n\
        this wont show up\n\
        foo-12 Another one\n\
        [foo 20] valid\n\
        Saw3 2 heh";
    let mut mock_params = Params::new();
    mock_params.project_id = String::from("FOO");
    let parsed = parse_jira_identifiers(&mock_params, String::from(mock_logs));
    println!("parsed: {:?}", parsed);
    let contains = |s: &'static str| parsed.contains(&String::from(s));
    assert!(contains("FOO-123"));
    assert!(parsed.contains(&String::from("FOO-12")));
    assert!(parsed.contains(&String::from("FOO-20")));
    assert_eq!(parsed.len(), 3);
}

// I need to handle cases where issues don't exist

fn main() {
    let parser = parameters::ParamsParser::new();
    let params: Params = parser.parse_params();
    println!("{:?}", params);
    match git_logs(&params) {
        Ok(logs) => {
            let parsed = parse_jira_identifiers(&params, logs);
            // and then here we go with hyper
            let client = Client::new();
            let url = params.url + "";
            match client.get(&url).send() {
                Ok(mut res) => {
                    let mut buffer = String::new();
                    res.read_to_string(&mut buffer);

                    println!("Ok: {:?}", res);
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        },
        Err(e) => println!("{}", e)
    };
}
