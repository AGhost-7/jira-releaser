
// This file contains code which parses the parameters. It handles parsing the
// arguments as well as falling back to environment variables. If I ever decide
// to add a .*rc file, the code handling this will be added here.

extern crate clap;
use self::clap::{App, Arg};
use std::env;
use std::ffi::OsString;

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
    pub project_id: String,
    pub version_name: String
}
impl Params {
    pub fn new () -> Params {
        Params {
            release_branch: String::from(""),
            latest_branch: String::from(""),
            username: String::from(""),
            password: String::from(""),
            url: String::from(""),
            project_id: String::from(""),
            version_name: String::from("")
        }
    }
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
            .version("0.2.0")
            .author("Jonathan Boudreau")
            .arg(Arg::with_name("Release branch")
                 .short("r")
                 .long("release-branch")
                 .takes_value(true)
                 .required(true)
                 .default_value("master")
                 .help("The branch which once the release is created, \
                       will be merged into"))
            .arg(Arg::with_name("Latest branch")
                .short("l")
                .long("latest-branch")
                .takes_value(true)
                .required(true)
                .default_value("develop")
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
            .arg(Arg::with_name("Version name")
                 .short("v")
                 .long("version-name")
                 .takes_value(true)
                 .required(true)
                 .help("The version name to use for the release."))
            .arg(self.username_arg())
            .arg(self.password_arg())
    }

    fn username_arg(&self) -> Arg {
        let arg = Arg::with_name("Username")
             .short("u")
             .long("username")
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

    pub fn parse_str<I, T>(&self, itr: I) -> Params 
            where I: IntoIterator<Item=T>, T: Into<OsString> {
        let app = self.get_app();
        let matches = app.get_matches_from(itr);
        let from_key = |s: &str| matches.value_of(s).unwrap().to_owned();
        Params {
            username: from_key("Username"),
            password: from_key("Password"),
            url: from_key("Jira URL"),
            release_branch: from_key("Release branch"),
            latest_branch: from_key("Latest branch"),
            project_id: from_key("Project Id"),
            version_name: from_key("Version name")
        }
    }

    pub fn parse_params(&self) -> Params {
        self.parse_str(env::args_os())
    }
}

#[test]
fn simple_parser() {
    let parser = ParamsParser {
        username_env: None,
        password_env: None
    };
    let args = vec![
        "program",
        "--username", "Foobar",
        "--password", "123",
        "--release-branch", "master",
        "--latest-branch", "devel",
        "--url", "http://noodle.com",
        "--project-id", "NOOB-9000",
        "--version-name", "1.1.1"
    ];
    let params = parser.parse_str(&args);
    assert_eq!(&params.username, "Foobar");
    assert_eq!(&params.release_branch, "master");
}

#[test]
fn with_env() {
    let parser = ParamsParser {
        username_env: Some(String::from("Hai")),
        password_env: Some(String::from("123"))
    };
    let args = vec![
        "program",
        "--release-branch", "foobar",
        "--url", "http://doodle.com",
        "--project-id", "WTF-2",
        "--version-name", "1.1.1"
    ];
    let params = parser.parse_str(&args);
    assert_eq!(&params.username, "Hai");
    assert_eq!(&params.latest_branch, "develop");
    assert_eq!(&params.release_branch, "foobar");
}
