use http_body_util::{combinators::BoxBody, BodyExt};
use http_body_util::{Empty, Full};
use hyper::body::{Body, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use regex::Regex;
use tokio::net::TcpListener;

pub const SHORT_LINK_LEN: usize = 6;
pub const SHORT_LINK_ALPHABET: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

lazy_static! {
    pub static ref URL_REGEX: Regex = Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();
}

async fn short_link(
    mut con: MultiplexedConnection,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full("Try POSTing link to /make_short"))),
        (&Method::POST, "/make_short") => {
            let url_len = req.body().size_hint().upper().unwrap_or(u64::MAX);
            if url_len > 1024 * 4 {
                return Ok(Response::builder()
                    .status(StatusCode::PAYLOAD_TOO_LARGE)
                    .body(full("Url too long"))
                    .unwrap());
            }

            let url = req.collect().await?.to_bytes();
            let url = String::from_utf8(url.into());
            if url.is_err() {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Incorrect utf8 string provided"))
                    .unwrap());
            }
            let url = url.unwrap();

            if !URL_REGEX.is_match(&url) {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Provided string is not a valid url"))
                    .unwrap());
            }

            match con.incr::<_, u64, u64>("url_id", 1u64).await {
                Ok(id) => {
                    let mut rng = ChaCha8Rng::seed_from_u64(id);
                    let sample = SHORT_LINK_ALPHABET.as_bytes();
                    let key = sample
                        .choose_multiple(&mut rng, SHORT_LINK_LEN)
                        .copied()
                        .collect();
                    let key = String::from_utf8(key).unwrap();
                    match con.set::<&String, String, String>(&key, url).await {
                        Ok(_) => Ok(Response::builder()
                            .status(StatusCode::CREATED)
                            .body(full(key))
                            .unwrap()),
                        Err(_) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(full("Unable to set value"))
                            .unwrap()),
                    }
                }
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(full("Unable to create id"))
                    .unwrap()),
            }
        }
        (&Method::GET, uri) => {
            let key = uri.strip_prefix('/').unwrap();
            match con.get::<&str, Option<String>>(key).await {
                Ok(None) => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(full(format!("Key {} not found", key)))
                    .unwrap()),
                Ok(Some(link)) => Ok(Response::builder()
                    .status(StatusCode::PERMANENT_REDIRECT)
                    .header("Location", link)
                    .body(empty())
                    .unwrap()),
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(full("Error accessing redis storage"))
                    .unwrap()),
            }
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full("Unknown URL"))
            .unwrap()),
    }
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() {
    // Setup connection to Redis
    let client = redis::Client::open("redis://redis:6379").unwrap();
    let con = client
        .get_multiplexed_async_connection_with_timeouts(
            std::time::Duration::from_secs(2),
            std::time::Duration::from_secs(2),
        )
        .await
        .unwrap();

    // Add socket listener
    let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();

    // create a new task for each connection
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let io = TokioIo::new(stream);
                let con = con.clone();
                tokio::task::spawn(async move {
                    let service = |req| short_link(con.clone(), req);
                    // Finally, we bind the incoming connection to our `hello` service
                    if let Err(err) = http1::Builder::new()
                        // `service_fn` converts our function in a `Service`
                        .serve_connection(io, service_fn(service))
                        .await
                    {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error when establishing connection: {}", e)
            }
        }
    }
}
