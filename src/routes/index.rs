use crate::utils::full_body;
use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, Method, Request, Response};
pub struct IndexHandler;
use super::Handler;

impl Handler for IndexHandler {
    fn is_match(method: &Method, uri: &str) -> bool {
        (method, uri) == (&Method::GET, "/")
    }

    async fn handle(
        _req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        Ok(Response::new(full_body("Try POSTing link to /make_short")))
    }
}
