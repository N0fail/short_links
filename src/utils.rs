use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt as _, Empty, Full};
use hyper::body::Bytes;
use lazy_static::lazy_static;
use regex::Regex;
pub const SHORT_LINK_LEN: usize = 6;
pub const SHORT_LINK_ALPHABET: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
lazy_static! {
    pub static ref URL_REGEX: Regex = Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&\/=]*)").unwrap();
    pub static ref KEY_REGEX: Regex = Regex::new(format!("[{SHORT_LINK_ALPHABET}]{{{SHORT_LINK_LEN}}}").as_str()).unwrap();
}
// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
pub fn empty_body() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
pub fn full_body<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
