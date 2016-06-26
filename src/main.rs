#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

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
                    .map_err(|_| "Could not parse git command \
                             stdout".to_owned())
            } else {
                match String::from_utf8(output.stderr) {
                    Ok(e) => {
                        Err("Error running git command".to_owned() + (&e))
                    },
                    Err(_) => {
                        Err("Error running git command: cannot parse \
                            stderr".to_owned())
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
            Json::from_str(&body)
                .map_err(|e| "Error parsing Json response.".to_owned())
        }
    }
}

//struct JiraIssue {
//    pub json: Json,
//    base_url: Url
//}
//
//impl JiraIssue {
//
//    // fetch from the issue tag
//    pub fn from_tag(client: &Client, params: &Params, tag: String)
//            -> Result<JiraIssue, String> {
//        let baseUrl = match Url::parse(&params.url) {
//            Ok(u) => u,
//            Err(e) => return Err("Error parsing url".to_owned())
//        };
//        let apiPart = String::from("/rest/api/2/issue") + (&tag);
//        let url = baseUrl.join(&apiPart).unwrap();
//        match client.get(url).send() {
//            Ok(mut res) => {
//                let body: String = match res.status {
//                    StatusCode::Ok | StatusCode::Created | StatusCode::Accepted => {
//                        let mut b = String::new();
//                        res.read_to_string(&mut b).unwrap();
//                        b
//                    },
//                    // TODO: not exist should be a special case?
//                    _ => {
//                        return Err("Error processing request".to_owned())
//                    }
//                };
//                if let Ok(json) = Json::from_str(&body) {
//                    Ok(JiraIssue {
//                        json: json,
//                        base_url: baseUrl
//                    })
//                } else {
//                    Err("Error parsing json response".to_owned())
//                }
//            },
//            Err(_) => {
//                Err("Error connecting to Jira site".to_owned())
//            }
//        }
//    }
//    // put request
//    fn update(&self, client: &Client) -> Result<JiraIssue, String> {
//        unimplemented!();
//    }
//}

use hyper::method::Method;
use hyper::client::IntoUrl;
use hyper::header::{Headers, Authorization, Basic, ContentType};

fn send_jira_request<U: IntoUrl>(
        client: &Client,
        method: Method,
        url: U,
        params: &Params,
        payload: Option<&str>
        ) -> Result<Response, hyper::error::Error> {
    let mut req = client.request(method, url);
    if let Some(payload_str) = payload {
        let content_type = ContentType(
            mime::Mime(
                mime::TopLevel::Application, mime::SubLevel::Json, Vec::new()
            )
        );
        req = req
            .header(content_type)
            .body(payload_str);
    }
    req.header(
        Authorization(
            Basic {
                username: params.username.clone(),
                password: Some(params.password.clone())
            }
        )
    ).send()
}

#[derive(RustcDecodable, RustcEncodable, Clone)]
struct JiraVersion {
    pub name: String,
    pub id: String
}

fn create_jira_version(client: &Client, params: &Params)
        -> Result<JiraVersion, String> {
    let mut map = BTreeMap::new();
    // what is the version??
    map.insert("name".to_owned(), Json::String(params.version_name.clone()));
    map.insert("project".to_owned(), Json::String(params.project_id.clone()));
    let payload_obj = Json::Object(map).to_string();

    let url = params.url.to_owned() + "/rest/api/2/version";
    let res = send_jira_request(
        client,
        Method::Post,
        &url,
        params,
        Some(&payload_obj)
    );

    match res {
        Ok(mut res) => {
            match res.status {
                StatusCode::NoContent | StatusCode::Ok => {
                    let mut body_str = String::new();
                    res.read_to_string(&mut body_str).unwrap();
                    let version: JiraVersion =
                        json::decode(&body_str).unwrap();
                    Ok(version)
                },
                rest => {
                    let str_status = res.status.canonical_reason().unwrap();
                    Err("Server error creating Jira version: ".to_owned() +
                        str_status)
                }
            }
        },
        Err(_) => {
            Err("Could not request creation of Jira version".to_owned())
        }
    }
}

// Returns true if the jira version exists already. False if it needs to be
// created.
fn get_jira_version(client: &Client, params: &Params) 
        -> Result<Option<JiraVersion>, String> {
    let url = params.url.to_owned() +
        "/rest/api/2/project/" +
        &params.project_id + "/versions";
    let res = send_jira_request(client, Method::Get, &url, params, None);
    match res {
        Ok(mut res) => {
            match res.status {
                StatusCode::Ok => {
                    let mut body = String::new();
                    res.read_to_string(&mut body).unwrap();
                    let versions: Vec<JiraVersion> =
                        json::decode(&body).unwrap();
                    let mut version: Option<JiraVersion> = None;
                    for v in versions {
                        if v.name == params.version_name {
                            version = Some(v);
                        }
                    }
                    Ok(version)
                },
                rest => {
                    let str_status = res.status.canonical_reason().unwrap();
                    Err("Server error fetching Jira version: ".to_owned() +
                        str_status)
                }
            }
        }
        Err(_) =>
            Err("Could not requests versions available for project".to_owned())
    }
}

// Makes a GET request and then creates the jira version if it doesnt exists.
fn ensure_project_version(client: &Client, params: &Params)
        -> Result<JiraVersion, String> {
    let version = try!(get_jira_version(client, params));
    if version.is_none() {
        create_jira_version(client, params)
    } else {
        Ok(version.unwrap())
    }
}

fn get_issue_versions(client: &Client, params: &Params, issue_token: &str)
        -> Result<Option<Vec<JiraVersion>>, String> {
    unimplemented!();
}

fn set_issue_versions(
        client: &Client,
        params: &Params,
        issue_token: &str,
        versions: Vec<JiraVersion>
        ) -> Result<Vec<JiraVersion>, String> {
    unimplemented!();
}

// Returns None if the issue doesnt exist. Otherwise returns a Some<Vec> with
// the newly set versions.
fn ensure_issue_version(
        client: &Client,
        params: &Params,
        issue_token: &str,
        version: &JiraVersion
        ) -> Result<Option<Vec<JiraVersion>>, String> {

    match try!(get_issue_versions(client, params, issue_token)) {
        None => Ok(None),
        Some(mut versions) => {
            if versions.iter().any(|v| v.name == params.version_name) {
                Ok(Some(versions))
            } else {
                // TODO: How do I not clone this???
                let o_v: JiraVersion = version.clone();
                versions.push(o_v);
                let new_versions = try!(
                    set_issue_versions(client, params, issue_token, versions)
                );
                Ok(Some(new_versions))
            }
        }
    }
}

// Returns issue tokens which dont exist.
fn publish_release<'s>(
        client: &Client,
        params: &Params,
        issue_tokens: &'s [String]
        ) -> Result<Vec<&'s str>, String> {
    let version = try!(ensure_project_version(client, params));
    // TODO: multihread...
    let mut not_exist: Vec<&'s str> = Vec::new();
    for issue_token in issue_tokens {
        // if the issue doesnt exist then I will just notify the user that
        // the issue found did not exist.
        let ensured = try!(
            ensure_issue_version(client, params, &issue_token, &version)
        );
        if ensured.is_none() {
            not_exist.push(issue_token);
        }
    }

    Ok(not_exist)
}

// I need to handle cases where issues don't exist

fn main() {
    let parser = parameters::ParamsParser::new();
    let params: Params = parser.parse_params();
    let token_parser = TokenParser::new(&params.project_id);
    match git_logs(&params) {
        Ok(logs) => {
            let issue_tokens = token_parser.parse(&logs);
            let client = Client::new();
            match publish_release(&client, &params, &issue_tokens[..]) {
                Ok(invalid_tokens) => {
                    if invalid_tokens.len() > 0 {
                        let mut msg = String::from("Found following issues in \
                            commits not present in Jira: ");
                        for (i, tkn) in invalid_tokens.iter().enumerate() {
                            if i != 0 {
                                msg.push_str(", ");
                            }
                            msg.push_str(tkn);
                        }
                        println!("{}.", msg);
                    }
                    std::process::exit(0);
                },
                Err(e) => {
                    println!("{}", e);
                    std::process::exit(1);
                }

            }
        },
        Err(e) => println!("{}", e)
    };
}
