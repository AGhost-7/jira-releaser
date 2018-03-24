
// This file contains code which parses the parameters. It handles parsing the
// arguments as well as falling back to environment variables. If I ever decide
// to add a .*rc file, the code handling this will be added here.

extern crate clap;
use self::clap::{App, Arg};
use std::env;
use std::io::{self, Read};
use std::error::Error;
use std::fmt;
use std::fmt::{Display};

#[derive(Debug)]
pub enum  ParamsErr {
    StdinErr,
    InvalidIssueErr(String),
    ProjectMismatchErr(String)
}

impl <'a>Error for ParamsErr {
    fn description(&self) -> &str {
        "ParamsErr"
    }
}

impl <'a>Display for ParamsErr {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &StdinErr =>
                write!(formatter, "Failed to read from stdin"),
            &InvalidIssueErr(ref issue) =>
                write!(formatter, "Invalid Jira issue \"{}\"", issue),
            &ProjectMismatchErr(ref project) =>
                write!(formatter, "Project \"{}\" is not consistent with other issues.", project)
        }
    }
}

#[derive(Debug)]
pub struct Params {
    pub issues: Vec<String>,
    pub username: String,
    pub password: String,
    pub url: String,
    pub version_name: String,
    pub project: Option<String>
}

use self::ParamsErr::*;

impl Params {

    fn get_app<'a>() -> App<'a, 'a> {
        App::new("Jira Release Tool")
            .version("0.3.0")
            .author("Jonathan Boudreau")
            .arg(Arg::with_name("Jira URL")
                 .short("U")
                 .long("url")
                 .takes_value(true)
                 .required(true)
                 .help("This is the api root url for your Jira project."))
            .arg(Arg::with_name("Version name")
                 .short("v")
                 .long("version-name")
                 .takes_value(true)
                 .required(true)
                 .help("The version name to use for the release."))
            .arg(Arg::with_name("Username")
                 .short("u")
                 .long("username")
                 .takes_value(true)
                 .help("Your Jira username. Falls back to the JIRA_USERNAME \
                    environment variable")
                 .required(true)
                 .env("JIRA_USERNAME"))
            .arg(Arg::with_name("Password")
                 .short("p")
                 .long("password")
                 .env("JIRA_PASSWORD")
                 .required(true)
                 .takes_value(true)
                 .help("Jira password. Falls back to JIRA_PASSWORD environment \
                    variable"))
    }

    fn issues<'a>() -> Result<Vec<String>, ParamsErr> {
        let mut issues = String::new();
        io::stdin()
            .read_to_string(&mut issues)
            .map_err(|_| ParamsErr::StdinErr)?;
        Ok(issues.lines().map(|issue| issue.to_owned()).collect())
    }

    fn project<'a>(issues: &'a Vec<String>) -> Result<Option<String>, ParamsErr> {
        let mut project: Option<String> = None;
        for issue in issues.iter() {
            let issue_project = issue
                .find("-")
                .map(|index| &issue[0..index])
                .ok_or_else(|| InvalidIssueErr(issue.to_owned()))?;
            match project {
                Some(ref project) if project != issue_project => {
                    return Err(ProjectMismatchErr(project.to_owned()))
                },
                Some(_) => (),
                None => {
                    project = Some(issue_project.to_owned());
                }
            }
        }
        Ok(project)
    }
    
    pub fn new() -> Result<Params, ParamsErr> {
        let app = Params::get_app();
        let matches = app.get_matches_from(env::args_os());
        let from_key = |s: &str| matches.value_of(s).unwrap().to_owned();
        let issues = Params::issues()?;
        let project = Params::project(&issues)?;
        let params = Params {
            project: project,
            issues: issues,
            username: from_key("Username"),
            password: from_key("Password"),
            url: from_key("Jira URL"),
            version_name: from_key("Version name")
        };

        Ok(params)
    }
}

