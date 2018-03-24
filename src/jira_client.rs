use parameters::Params;

use std::fmt;
use std::error::Error as StdError;

use futures;
use futures::future;
use futures::{Future, Stream};

use tokio_core::reactor::{Handle};

use hyper;
use hyper::Client;
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;

use std::io;
use serde_json;

// TODO: client pooling??
pub struct JiraClient {
    client: Client<HttpsConnector<HttpConnector>>
}

#[derive(Debug)]
pub enum Error {
    StatusCode(Vec<String>),
    JsonParse,
    Hyper(hyper::Error)
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
        match self {
            StatusCode(ref messages) => write!(formatter, "Got bad response from Jira"),
            JsonParse => write!(formatter, "Bam"),
            Hyper(ref err) => err.fmt(formatter)
        }
    }
}

// Just a wrapper around futures to make things more ergonomic...
pub struct JiraFuture(Box<Future<Item=serde_json::Value, Error=Error>>);

impl Future for JiraFuture {
    type Item = serde_json::Value;
    type Error = self::Error;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

impl JiraClient {

    pub fn new(handle: &Handle) -> JiraClient {
        let client = Client::configure()
            .connector(HttpsConnector::new(4, handle).unwrap())
            .build(handle);

        JiraClient {
            client: client
        }
    }

    pub fn get_versions(&self, params: &Params) -> JiraFuture {
        let uri = params.url.to_owned() + "/rest/api/2/project/" + params.project.as_ref().unwrap();
        let result = self
            .client
            .get((&uri).parse().unwrap())
            .and_then(|res| {
                res.body().concat2().and_then(|body| {
                    //let parsed = serde_json::from_slice(&body).map_err(|e| {
                    //    io::Error::new(
                    //        io::ErrorKind::Other,
                    //        e
                    //    )
                    //})?;

                    match serde_json::from_slice(&body) {
                        Ok(parsed) if res.status().is_success() => {
                            println!("parsed: {:?}", parsed);
                            parsed
                        },
                        Ok(parsed) => {
                            Error::StatusCode(Vec::new())
                        },
                        Err(_) => Error::JsonParse
                    }

                    //if let Ok(parsed) = serde_json(&body) {
                    //    if !res.status().is_success() {

                    //} else {
                    //    Error::JsonParse
                    //}
                    //if !res.status().is_success() {
                    //    return Error::StatusCode(Vec::new())
                    //}

                    //    
                    //Ok(parsed)
                })
            });
        JiraFuture(Box::new(result))
    }
}
