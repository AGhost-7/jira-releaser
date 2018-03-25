
// Just a wrapper around futures to make things more ergonomic...

use futures;
use jira::error::Error;

use futures::Future;

pub struct JiraFuture<A>(Box<Future<Item=A, Error=Error>>);

impl <A> Future for JiraFuture<A> {
    type Item = A;
    type Error = self::Error;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

impl <A> JiraFuture<A> {
    pub fn new(future: Box<Future<Item=A, Error=Error>>) -> JiraFuture<A> {
        JiraFuture(future)
    }
}
