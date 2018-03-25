use parameters::Params;

use std::str;
use std::fmt;

use futures;
use futures::{Stream, Future};

use tokio_core::reactor::{Handle};

use hyper;
use hyper::Method;
use hyper::header;
use hyper::mime;
use hyper::Client as HttpsClient;
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;

use serde_json;
use serde_json::Value as JsValue;
use serde::de::Deserialize;

use jira::error::Error;
use jira::model;
use jira::future::JiraFuture;

// TODO: client pooling??
pub struct Client {
    client: HttpsClient<HttpsConnector<HttpConnector>>
}

impl Client {

    pub fn new(handle: &Handle) -> Client {
        let client = HttpsClient::configure()
            .connector(HttpsConnector::new(4, handle).unwrap())
            .build(handle);

        Client {
            client: client
        }
    }

    fn handle<'d, 'a, A>(response: hyper::client::FutureResponse) -> JiraFuture<A> 
            where A: fmt::Debug + Deserialize<'d> + 'a {
        let js_future = response
            .map_err(|err| Error::Hyper(err))
            .and_then(|response| {
                let status = response.status();
                response
                    .body()
                    .map_err(|err| Error::Hyper(err))
                    .concat2()
                    .and_then(move |body| {
                        match status {
                            hyper::StatusCode::Unauthorized => {
                                return Err(Error::Authentication)
                            },
                            _ => ()
                        }
                        let result: serde_json::Result<A>= serde_json::from_slice(&body);
                        println!("got body {:?}", str::from_utf8(&body));

                        if let Ok(parsed) = result {
                            println!("response: {:?}", parsed);
                            if status.is_success() {
                                Ok(parsed)
                            } else {
                                // TODO: handle error message from server
                                // correctly
                                Err(Error::StatusCode(Vec::new()))
                            }
                        } else {
                            println!("parse error: {:?}", result);
                            Err(Error::JsonParse)
                        }
                    })
            });
        JiraFuture::new(Box::new(js_future))
    }

    fn request(params: &Params, method: Method, path: &str) -> hyper::Request {
        let uri = params.url.to_owned() + "/rest/api/2" + path;
        let mut request = hyper::Request::new(method, (&uri).parse().unwrap());
        request.headers_mut().set(header::ContentType::json());
        request.headers_mut().set(
            header::Authorization(header::Basic {
                username: params.username.clone(),
                password: Some(params.password.clone())
            })
        );
        request.headers_mut().set(
            header::Accept(vec![
                header::qitem(mime::APPLICATION_JSON)
            ])
        );

        request
    }

    pub fn get_versions(&self, params: &Params) -> JiraFuture<JsValue> {
        let path = String::from("/project/") + params.project.as_ref().unwrap() + "/versions";
        let request = Client::request(params, Method::Get, &path);
        let response = self.client.request(request);
        Client::handle(response)
    }
}
