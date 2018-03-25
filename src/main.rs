#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate hyper;
extern crate hyper_tls;
extern crate futures;
extern crate tokio_core;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;
extern crate env_logger;

use tokio_core::reactor::{Core};

use futures::prelude::*;

mod parameters;
use parameters::Params;

mod jira;
use jira::error::Error as JiraError;


fn main() {
    env_logger::init().unwrap();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = jira::Client::new(&handle);

    let params = Params::new().unwrap();
    info!("params: {:?}", params);

    if params.issues.len() > 0 {
        println!("getting version");
        let future = client
            .get_versions(&params)
            .map_err(|err| {
                println!("{}", err);
                err
            })
            .inspect(|versions| {
                print!("versions: {}", serde_json::to_string_pretty(versions).unwrap());
            });
        core.run(future);
    } else {
        println!("nerp");
        info!("No issues specified, skipping version creation");
    }
}

