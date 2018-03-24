#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate hyper;
extern crate hyper_tls;
extern crate futures;
extern crate tokio_core;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;

use tokio_core::reactor::{Core};

mod parameters;
use parameters::Params;

mod jira_client;
use jira_client::JiraClient;

fn main() {
    env_logger::init().unwrap();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = JiraClient::new(&handle);

    let params = Params::new().unwrap();
    info!("params: {:?}", params);

    if params.issues.len() > 0 {
        println!("getting version");
        core.run(client.get_versions(&params));
    } else {
        println!("nerp");
        info!("No issues specified, skipping version creation");
    }
}

