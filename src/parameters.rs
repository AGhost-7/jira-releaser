
// This file contains code which parses the parameters. It handles parsing the arguments
// as well as falling back to environment variables. If I ever decide to add a .*rc file,
// the code handling this will be added here.

extern crate clap;
use self::clap::{App, Arg};
use std::env;
use std::process;

pub struct Params<'p> {
    release_branch: &'p str,
    latest_branch: &'p str,
    username: &'p str,
    password: &'p str,
    url: &'p str
}

impl <'p>Params<'p> {

    pub fn get_params() -> Params<'p> {
        let app = Params::get_app();
        let args = app.get_matches();
        println!("args: {:?}", args);
        Params {
            release_branch: "",
            latest_branch: "",
            username: "",
            password: "",
            url: ""
        }
    }

    fn get_password_arg<'r>() -> Arg<'r, 'r> {
        let mut arg = Arg::with_name("Password")
            .short("p")
            .long("password")
            .takes_value(true)
            .help("Jira password. Falls back to JIRA_PASSWORD environment \
                variable");
        match env::var_os("JIRA_PASSWORD") {
            Some(password) => {
                arg.default_value(password)
            },
            None => {
                arg.required(true)
            }
        }
    }

    fn get_username_arg<'r>() -> Arg<'r, 'r> {
        let mut arg = Arg::with_name("Username")
             .short("u")
             .long("usename")
             .takes_value(true)
             .help("Your Jira username. Falls back to the JIRA_USERNAME \
                environment variable");
        match env::var_os("JIRA_USERNAME") {
            Some(username) => {
                arg.default_value(username)
            },
            None => {
                arg.required(true)
            }
        }
    }

    fn get_app<'r>() -> App<'r, 'r> {
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
            .arg(Arg::with_name("Lastest branch")
                .short("l")
                .long("latest-branch")
                .takes_value(true)
                .required(true)
                .help("The branch which is going to be merged to trigger \
                    the release"))
            .arg(Params::get_username_arg())
            .arg(Params::get_password_arg())
            .arg(Arg::with_name("Jira Url")
                  .short("U")
                  .long("url")
                  .takes_value(true)
                  .required(true)
                  .help("This is the api root url for your Jira project."))
    }
}


