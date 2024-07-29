mod index;
mod make_short;
mod not_found;
mod try_redirect;
use std::sync::Arc;

use crate::{ShortLinkStorage, STORAGE};
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Method, Request, Response};
use index::IndexHandler;
use make_short::MakeShortHandler;
use not_found::NotFoundHandler;
use try_redirect::TryRedirectHandler;

pub trait Handler {
    fn is_match(method: &Method, uri: &str) -> bool;
    async fn handle(
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>;
    fn get_storage() -> Arc<dyn ShortLinkStorage> {
        return STORAGE.get().unwrap().clone();
    }
}

macro_rules! reg_handlers {
    ( $req:expr, $( $handler:ident ),* ) => {{
            let method = $req.method();
            let uri = $req.uri().path();
            $(
                if $handler::is_match(method, uri) {
                    return $handler::handle($req).await;
                }
            )*
        }
    };
}

pub async fn handle(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    reg_handlers!(req, IndexHandler, MakeShortHandler, TryRedirectHandler);
    NotFoundHandler::handle(req).await
}
