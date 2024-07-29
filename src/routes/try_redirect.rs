use super::Handler;
use crate::utils::empty_body;
use crate::utils::KEY_REGEX;
use crate::STORAGE;
use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, Method, Request, Response, StatusCode};

pub struct TryRedirectHandler;
impl Handler for TryRedirectHandler {
    fn is_match(method: &Method, uri: &str) -> bool {
        method == Method::GET && KEY_REGEX.is_match(uri)
    }

    async fn handle(
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let uri = req.uri().path();
        let key = uri.strip_prefix('/').unwrap();
        match STORAGE.get().unwrap().clone().load(key).await {
            Ok(None) => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty_body())
                .unwrap()),
            Ok(Some(link)) => Ok(Response::builder()
                .status(StatusCode::PERMANENT_REDIRECT)
                .header("Location", link)
                .body(empty_body())
                .unwrap()),
            Err(_) => {
                eprint!("Error accessing redis storage");
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(empty_body())
                    .unwrap())
            }
        }
    }
}
