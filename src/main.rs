extern crate regex;
extern crate hyper;
extern crate rustc_serialize;

use std::process::Command;
use hyper::{Client, Url};
use hyper::client::response::Response;
use hyper::status::StatusCode;

use hyper::mime;

use std::io::Read;
use rustc_serialize::json::{self, Json};
use std::collections::BTreeMap;

mod parameters;
mod token_parser;

use parameters::Params;
use token_parser::TokenParser;

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

fn parse_body(res: &mut Response) -> Result<Json, String> {
    let mut body: String = String::new();
    match res.read_to_string(&mut body) {
        Err(_) => {
            Err("Error reading body".to_owned())
        },
        Ok(_) => {
            Json::from_str(&body).map_err(|e| "Error parsing Json response.".to_owned())
        }
    }
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
                let body: String = match res.status {
                    StatusCode::Ok | StatusCode::Created | StatusCode::Accepted => {
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

fn create_jira_version(client: &Client, params: &Params) -> Result<Json, String> {
    let mut map = BTreeMap::new();
    // what is the version??
    map.insert("name".to_owned(), Json::String(params.version_name.clone()));
    map.insert("project".to_owned(), Json::String(params.project_id.clone()));
    let payload_obj = Json::Object(map);
    let content_type = hyper::header::ContentType(
        mime::Mime(mime::TopLevel::Application, mime::SubLevel::Json, Vec::new())
    );
    let url = params.url.to_owned() + "/rest/api/2/version";
    let res = client
        .post(&url)
        .body(&payload_obj.to_string())
        .header(content_type)
        .send();
    match res {
        Ok(mut res) => {
            match res.status {
                StatusCode::NoContent | StatusCode::Ok => {
                    parse_body(&mut res)
                },
                rest => {
                    let str_status = res.status.canonical_reason().unwrap();
                    Err("Server error creating Jira version: ".to_owned() + str_status)
                }
            }
        },
        Err(_) => {
            Err("Could not request creation of Jira version".to_owned())
        }
    }
}

fn get_jira_versions(client: &Client, params: &Params) -> Result<Json, String> {
    let url = params.url.to_owned() + "/rest/api/2/project/" + &params.project_id + "versions";
    unimplemented!();
}

// Makes a GET request and then creates the jira version if it doesnt exists
fn ensure_jira_version(client: &Client, params: &Params) -> Result<Json, String> {
    unimplemented!();
}

// I need to handle cases where issues don't exist

fn main() {
    let parser = parameters::ParamsParser::new();
    let params: Params = parser.parse_params();
    println!("{:?}", params);
    let token_parser = TokenParser::new(&params.project_id);
    match git_logs(&params) {
        Ok(logs) => {
            let parsed = token_parser.parse(&logs);//parse_jira_identifiers(&params, logs);
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
