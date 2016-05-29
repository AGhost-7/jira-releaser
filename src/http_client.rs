//extern crate hyper;
//
//// adapted from the hyper test suite.
//
//use hyper::net::HttpStream;
//use hyper::client::{Handler, Request, Response, HttpConnector};
//use hyper::header::Header;
//use hyper::Method;
//use hyper::Client;
//
//struct HttpHandler {
//    method: Method
//}
//
//struct HttpResponse {
//}
//
//struct HttpClient {
//    client: Client
//}
//
//impl Handler<HttpStream> for HttpHandler {
//    fn on_request(&mut self, req: &mut Request) -> Next {
//        req.set_method(self.opts.method.clone());
//        read(&self.opts)
//    }
//
//    fn on_request_writable(&mut self, _encoder: &mut Encoder<HttpStream>) -> Next {
//        read(&self.opts)
//    }
//
//    fn on_response(&mut self, res: Response) -> Next {
//        use hyper::header;
//        // server responses can include a body until eof, if not size is specified
//        let mut has_body = true;
//        if let Some(len) = res.headers().get::<header::ContentLength>() {
//            if **len == 0 {
//                has_body = false;
//            }
//        }
//        self.tx.send(Msg::Head(res)).unwrap();
//        if has_body {
//            read(&self.opts)
//        } else {
//            Next::end()
//        }
//    }
//
//    fn on_response_readable(&mut self, decoder: &mut Decoder<HttpStream>) -> Next {
//        let mut v = vec![0; 512];
//        match decoder.read(&mut v) {
//            Ok(n) => {
//                v.truncate(n);
//                self.tx.send(Msg::Chunk(v)).unwrap();
//                if n == 0 {
//                    Next::end()
//                } else {
//                    read(&self.opts)
//                }
//            },
//            Err(e) => match e.kind() {
//                io::ErrorKind::WouldBlock => read(&self.opts),
//                _ => panic!("io read error: {:?}", e)
//            }
//        }
//    }
//
//    fn on_error(&mut self, err: hyper::Error) -> Next {
//        self.tx.send(Msg::Error(err)).unwrap();
//        Next::remove()
//    }
//}
