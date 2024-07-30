use crate::utils::{empty_body, full_body};
use crate::utils::{SHORT_LINK_ALPHABET, SHORT_LINK_LEN};
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

        let key = match STORAGE.get().unwrap().get_seed().await {
            Ok(seed) => MakeShortHandler::generate_key(seed),
            Err(e) => {
                //todo logging
                eprintln!("Unable to create id {e}");
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(empty_body())
                    .unwrap());
            }
        };

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

impl MakeShortHandler {
    fn generate_key(seed: u64) -> String {
        let salt = 123456789u64;
        let mut key = seed ^ salt;
        let alphabet_len = SHORT_LINK_ALPHABET.len() as u64;
        let mut res = vec![0u8; SHORT_LINK_LEN];
        for c in res.iter_mut() {
            *c = SHORT_LINK_ALPHABET.as_bytes()[(key % alphabet_len) as usize];
            key /= alphabet_len;
        }
        String::from_utf8(res).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_generate_key_unique() {
        let mut generated_keys: HashSet<String> = HashSet::new();
        for seed in 0..1000000u64 {
            let new_key = MakeShortHandler::generate_key(seed);
            assert!(!generated_keys.contains(&new_key));
            generated_keys.insert(new_key);
        }
    }

    #[test]
    fn test_generate_key_stable() {
        // this test is needed for backwards compatibility in case of changing generated_key
        let expected = vec![
            (0u64, "huAWIA".to_string()),
            (1u64, "guAWIA".to_string()),
            (1000u64, "Z2AWIA".to_string()),
            (123u64, "8vAWIA".to_string()),
            (123456789u64, "AAAAAA".to_string()),
            (3228u64, "189VIA".to_string()),
        ];
        for (seed, expected_key) in expected {
            assert_eq!(MakeShortHandler::generate_key(seed), expected_key)
        }
    }
}
