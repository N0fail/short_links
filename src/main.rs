mod routes;
pub mod storage;
pub mod utils;
use std::sync::Arc;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use once_cell::sync::OnceCell;
use routes::handle;
use storage::ShortLinkStorage;
use tokio::net::TcpListener;

pub static STORAGE: OnceCell<Arc<dyn ShortLinkStorage>> = OnceCell::new();

#[tokio::main]
async fn main() {
    // Add socket listener
    let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    let redis = redis::Client::open("redis://redis:6379")
        .unwrap()
        .get_multiplexed_async_connection_with_timeouts(
            std::time::Duration::from_secs(2),
            std::time::Duration::from_secs(2),
        )
        .await
        .unwrap();
    let _ = STORAGE.set(Arc::new(redis));
    // create a new task for each connection
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let io = TokioIo::new(stream);
                tokio::task::spawn(async move {
                    // Finally, we bind the incoming connection to our `hello` service
                    if let Err(err) = http1::Builder::new()
                        // `service_fn` converts our function in a `Service`
                        .serve_connection(io, service_fn(handle))
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
