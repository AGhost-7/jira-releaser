#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate regex;
extern crate hyper;
extern crate rustc_serialize;
#[cfg(test)]
extern crate mockito;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::process::Command;
use hyper::Client;
use hyper::client::response::Response;
use hyper::status::{StatusCode, StatusClass};

use hyper::mime;

use std::io::Read;
use rustc_serialize::json::{self, Json};
use std::collections::BTreeMap;

use hyper::method::Method;
use hyper::client::IntoUrl;
use hyper::header::{Authorization, Basic, ContentType};

pub mod parameters;
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
#[derive(RustcDecodable, RustcEncodable)]
struct JiraIssueFields {
    pub fixVersions: Vec<JiraVersion>
}

#[derive(RustcDecodable, RustcEncodable)]
struct JiraIssue {
    pub fields: JiraIssueFields
}

fn create_jira_version(client: &Client, params: &Params)
        -> Result<JiraVersion, String> {
    let mut map = BTreeMap::new();
    // what is the version??
    map.insert("name".to_owned(), Json::String(params.version_name.clone()));
    map.insert("project".to_owned(), Json::String(params.project_id.clone()));
    let payload_obj = Json::Object(map).to_string();

    let url = params.url.to_owned() + "/rest/api/2/version";
    debug!("creating Jira version {} through url: {}", params.version_name,
        url);
    debug!("POST payload: {:?}", payload_obj);
    let res = send_jira_request(
        client,
        Method::Post,
        &url,
        params,
        Some(&payload_obj)
    );

    match res {
        Ok(mut res) => {
            match res.status.class() {
                StatusClass::Success => {
                    let mut body_str = String::new();
                    res.read_to_string(&mut body_str).unwrap();
                    let version: JiraVersion =
                        json::decode(&body_str).unwrap();
                    Ok(version)
                },
                _ => {
                    //let str_status = res.status.canonical_reason().unwrap();
                    Err(format!("Server error creating Jira version {}", res.status))
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
    debug!("fetching jira versions for project {} with url: {}",
        params.project_id, url);
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
                _ => {
                    let msg = format!("Server error fetching Jira versions \
                        for project {}: {}", params.project_id, res.status);
                    Err(msg)
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

fn generic_issue_error<E>(code: &StatusCode, issue_token: &str)
        -> Result<E, String> {
    Err(
        format!(
            "Error with issue {issue_token}: {code}",
            code = code,
            issue_token = issue_token
        )
    )
}

fn get_issue_versions(client: &Client, params: &Params, issue_token: &str)
        -> Result<Option<Vec<JiraVersion>>, String> {
    let url = params.url.to_string() + "/rest/api/2/issue/" + issue_token;
    debug!("fetching issue {} through url: {}", issue_token, url);
    match send_jira_request(client, Method::Get, &url, params, None) {
        Ok(mut res) => {
            match res.status {
                StatusCode::NotFound => Ok(None),
                StatusCode::Ok => {
                    let mut data = String::new();
                    res.read_to_string(&mut data).unwrap();
                    let issue: JiraIssue = json::decode(&data).unwrap();
                    Ok(Some(issue.fields.fixVersions))
                },
                rest => generic_issue_error(&rest, issue_token)
            }
        },
        Err(_) => {
            Err("Error connecting to server".to_owned())
        }
    }
}

fn set_issue_versions(
        client: &Client,
        params: &Params,
        issue_token: &str,
        versions: Vec<JiraVersion>
        ) -> Result<Vec<JiraVersion>, String> {
    let url = params.url.to_string() + "/rest/api/2/issue/" + issue_token;
    debug!("modifying issue {} through url: {}", issue_token, url);
    let issue = JiraIssue {
        fields: JiraIssueFields {
            fixVersions: versions.clone()
        }
    };
    let payload = json::encode(&issue).unwrap();
    let response_result =
        send_jira_request(client, Method::Put, &url, params, Some(&payload));
    match response_result {
        Ok(res) => {
            match res.status.class() {
                StatusClass::Success => {
                    Ok(versions)
                },
                _ => generic_issue_error(&res.status, issue_token)
            }
        },
        Err(_) => {
            Err("Error connecting to server".to_owned())
        }
    }
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
pub fn publish_release<'s>(
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

fn main() {
    env_logger::init().unwrap();
    let parser = parameters::ParamsParser::new();
    let params: Params = parser.parse_params();
    info!("params: {:?}", params);
    let token_parser = TokenParser::new(&params.project_id);
    match git_logs(&params) {
        Ok(logs) => {
            let issue_tokens = token_parser.parse(&logs);
            if log_enabled!(log::LogLevel::Debug) {
                let mut msg = String::from("Tokens in logs: ");
                for (i, tkn) in issue_tokens.iter().enumerate() {
                    if i != 0 {
                        msg.push_str(", ");
                    }
                    msg.push_str(tkn);
                }
                debug!("{}", msg);
            }
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


#[cfg(test)]
mod test {
    use hyper::Client;
    use mockito::{mock, SERVER_ADDRESS};
    use parameters::Params;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    use ::std;
    use ::env_logger;
    use std::env;

    fn slurp_fixture(file_path: &str) -> String {
        let fixtures_dir = std::env::var("FIXTURES_DIR").unwrap();
        let base_path = Path::new(&fixtures_dir);
        let p_buf = base_path.join(file_path);
        let p = p_buf.as_path();
        let mut string = String::new();
        let mut file = File::open(&p).unwrap();

        file.read_to_string(&mut string).unwrap();
        string
    }

    #[test]
    fn pull_data() {
        env_logger::init().unwrap();
        let issue_1 = slurp_fixture("issue1_response.json");
        let versions = slurp_fixture("versions_response.json");
        let create_version = slurp_fixture("versions_response.json");
        mock("GET", "/rest/api/2/issue/EX-2")
            .match_header("content-type", "application/json")
            .with_body(&issue_1)
            .create();
        mock("GET", "/rest/api/2/project/EX/versions")
            .match_header("content-type", "application/json")
            .with_body(&versions)
            .create();
        mock("POST", "/rest/api/2/project/EX/versions")
            .match_header("content-type", "applcation/json")
            .with_body(&create_version)
            .create();
        mock("PUT", "/rest/api/2/issue/EX-2")
            .match_header("content-type", "application/json")
            .with_status(204)
            .create();
        mock("GET", "/a").with_body("a").create();

        let mut params = Params::new();
        params.url = "http://".to_owned() + SERVER_ADDRESS;
        params.project_id = "EX".to_owned();
        params.version_name = "1.2.0".to_owned();

        let client = Client::new();

        let issue_tokens = [
            "EX-1".to_owned(),
            "EX-2".to_owned()
        ];
        let res = super::publish_release(&client, &params, &issue_tokens[..]);
        println!("{:?}", res);
        assert!(res.is_ok(), "Did not error out");
    }
}
