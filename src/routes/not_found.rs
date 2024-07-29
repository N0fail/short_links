use super::Handler;
use crate::utils::empty_body;
use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, Method, Request, Response, StatusCode};
pub struct NotFoundHandler;

impl Handler for NotFoundHandler {
    fn is_match(_method: &Method, _uri: &str) -> bool {
        true
    }

    async fn handle(
        _req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(empty_body())
            .unwrap())
    }
}
