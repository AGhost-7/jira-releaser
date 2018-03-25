
use hyper;
use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    StatusCode(Vec<String>),
    JsonParse,
    Hyper(hyper::Error),
    Authentication
}


impl StdError for Error {
    fn description(&self) -> &str {
        "JiraError"
    }
}

// Automatically convert hyper errors into our client errors.
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::Hyper(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            StatusCode(_) => write!(formatter, "Got bad response from Jira"),
            JsonParse => write!(formatter, "Bam"),
            Hyper(ref err) => err.fmt(formatter),
            Authentication => write!(formatter, "Authentication failed")
        }
    }
}
