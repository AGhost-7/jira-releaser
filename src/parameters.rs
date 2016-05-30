
// This file contains code which parses the parameters. It handles parsing the arguments
// as well as falling back to environment variables. If I ever decide to add a .*rc file,
// the code handling this will be added here.

extern crate clap;
use self::clap::{App, Arg};
use std::env;

pub struct ParamsParser {
    username_env: Option<String>,
    password_env: Option<String>
}

#[derive(Debug)]
pub struct Params {
    pub release_branch: String,
    pub latest_branch: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub project_id: String
}

impl ParamsParser {
    pub fn new<'n>() -> ParamsParser {
        ParamsParser {
            username_env: env::var("JIRA_USERNAME").ok(),
            password_env: env::var("JIRA_PASSWORD").ok()
        }
    }
    fn get_app(&self) -> App {
        App::new("Jira Release Tool")
            .version("0.1.0")
            .author("Jonathan Boudreau")
            .arg(Arg::with_name("Release branch")
                 .short("r")
                 .long("release-branch")
                 .takes_value(true)
                 .required(true)
                 .help("The branch which once the release is created, \
                       will be merged into"))
            .arg(Arg::with_name("Latest branch")
                .short("l")
                .long("latest-branch")
                .takes_value(true)
                .required(true)
                .help("The branch which is going to be merged to trigger \
                    the release"))
            .arg(Arg::with_name("Jira URL")
                  .short("U")
                  .long("url")
                  .takes_value(true)
                  .required(true)
                  .help("This is the api root url for your Jira project."))
            .arg(Arg::with_name("Project Id")
                 .short("P")
                 .long("project-id")
                 .takes_value(true)
                 .required(true)
                 .help("Project id or key on Jira"))
            .arg(self.username_arg())
            .arg(self.password_arg())
    }

    fn username_arg(&self) -> Arg {
        let arg = Arg::with_name("Username")
             .short("u")
             .long("usename")
             .takes_value(true)
             .help("Your Jira username. Falls back to the JIRA_USERNAME \
                environment variable");
        match self.username_env.as_ref() {
            Some(username) => arg.default_value(username),
            None => arg.required(true)
        }
    }

    fn password_arg(&self) -> Arg {
        let arg = Arg::with_name("Password")
            .short("p")
            .long("password")
            .takes_value(true)
            .help("Jira password. Falls back to JIRA_PASSWORD environment \
                variable");
        match self.password_env.as_ref() {
            Some(password) => arg.default_value(password),
            None => arg.required(true)
        }
    }

    pub fn parse_params(&self) -> Params {
        let app = self.get_app();
        let matches = app.get_matches();
        println!("{:?}", matches);
        let from_key = |s: &str| matches.value_of(s).unwrap().to_owned();
        Params {
            username: from_key("Username"),
            password: from_key("Password"),
            url: from_key("Jira URL"),
            release_branch: from_key("Release branch"),
            latest_branch: from_key("Latest branch"),
            project_id: from_key("Project Id")
        }
    }
}


