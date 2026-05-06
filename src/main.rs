use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::pin;

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("in hello before sleep");
    tokio::time::sleep(Duration::from_secs(6)).await;
    println!("in hello after sleep");
    Ok(Response::new(Full::new(Bytes::from("Hello World!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let connection_timeouts = vec![Duration::from_secs(5), Duration::from_secs(2)];
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _addr) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let connection_timeouts_clone = connection_timeouts.clone();

        tokio::task::spawn(async move {
            let builder = AutoBuilder::new(TokioExecutor::new());
            let conn = builder.serve_connection(io, service_fn(hello));
            pin!(conn);

            for (iter, sleep_duration) in connection_timeouts_clone.iter().enumerate() {
                println!("iter = {} sleep_duration = {:?}", iter, sleep_duration);
                tokio::select! {
                    res = conn.as_mut() => {
                        match res {
                            Ok(()) => println!("after polling conn, no error"),
                            Err(e) =>  println!("error serving connection: {:?}", e),
                        };
                        break;
                    }
                    _ = tokio::time::sleep(*sleep_duration) => {
                        println!("iter = {} got timeout_interval, calling conn.graceful_shutdown", iter);
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }
        });
    }
}
