use crate::utils::{empty_body, full_body};
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::{Body, Bytes},
    Method, Request, Response, StatusCode,
};
pub struct MakeShortHandler;
use super::Handler;
use crate::utils::URL_REGEX;
use crate::STORAGE;

impl Handler for MakeShortHandler {
    fn is_match(method: &Method, uri: &str) -> bool {
        (method, uri) == (&Method::POST, "/make_short")
    }

    async fn handle(
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let url_len = req.body().size_hint().upper().unwrap_or(u64::MAX);
        if url_len > 1024 * 4 {
            return Ok(Response::builder()
                .status(StatusCode::PAYLOAD_TOO_LARGE)
                .body(full_body("Url too long"))
                .unwrap());
        }

        let url = req.collect().await?.to_bytes();
        let url = String::from_utf8(url.into());
        if url.is_err() {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(full_body("Incorrect utf8 string provided"))
                .unwrap());
        }
        let url = url.unwrap();

        if !URL_REGEX.is_match(&url) {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(full_body(format!(
                    "Provided string is not a valid url: {url}"
                )))
                .unwrap());
        }

        let key = STORAGE.get().unwrap().generate_key().await;
        if key.is_err() {
            eprint!("Unable to generate_key");
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(empty_body())
                .unwrap());
        }
        let key = key.unwrap();

        match STORAGE.get().unwrap().save(&key, &url).await {
            Ok(_) => Ok(Response::builder()
                .status(StatusCode::CREATED)
                .body(full_body(key))
                .unwrap()),
            Err(_) => {
                eprintln!("Unable to set value");
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(empty_body())
                    .unwrap())
            }
        }
    }
}
