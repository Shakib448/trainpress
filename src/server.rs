use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use tokio::net::TcpListener;
use tokio::task::JoinSet;
use tokio::time::timeout;

use crate::app::App;
use crate::error::AppError;
use crate::extract::PathParams;
use crate::handler::into_handler;
use crate::middleware::Next;
use crate::{Handler, Request};

pub async fn serve<S>(
    app: App<S>,
    addr: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: Clone + Send + Sync + 'static,
{
    let addr: SocketAddr = addr.parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "server listening");

    let state_ext = app.state_ext();
    let router = Arc::new(app.router);
    let middlewares = Arc::new(app.middlewares);

    let not_found: Handler = into_handler(|_req: Request| async { AppError::NotFound });

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    let mut connections = JoinSet::new();

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, peer) = match accepted {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(error = %e, "accept failed");
                        continue;
                    }
                };

                let io = TokioIo::new(stream);
                let router = router.clone();
                let middlewares = middlewares.clone();
                let state_ext = state_ext.clone();
                let not_found = not_found.clone();

                connections.spawn(async move {
                    handle_connection(io, router, middlewares, state_ext, not_found, peer).await;
                });
            }

            _ = &mut shutdown => {
                tracing::info!("⏹  shutdown signal received");
                break;
            }

            Some(_) = connections.join_next() => {
            }
        }
    }
    tracing::info!(
        active_connection = connections.len(),
        "draining connections"
    );
    drop(listener);

    // Graceful shutdown timeout (30 seconds)
    let graceful_timeout = Duration::from_secs(30);

    match timeout(graceful_timeout, async {
        while let Some(result) = connections.join_next().await {
            if let Err(e) = result {
                tracing::warn!(error = %e, "connection task panicked");
            }
        }
    })
    .await
    {
        Ok(_) => {
            tracing::info!("all connections closed gracefully");
        }
        Err(_) => {
            tracing::warn!(
                remaining_connections = connections.len(),
                "graceful shutdown timeout exceeded, forcefully closing remaining connections"
            );
            connections.abort_all();
        }
    }

    tracing::info!("server stopped cleanly");

    Ok(())
}

async fn handle_connection<S>(
    io: TokioIo<tokio::net::TcpStream>,
    router: Arc<crate::router::Router>,
    middlewares: Arc<Vec<Arc<dyn crate::middleware::Middleware>>>,
    state_ext: crate::extract::StateExt<S>,
    not_found: Handler,
    peer: SocketAddr,
) where
    S: Clone + Send + Sync + 'static,
{
    let service = service_fn(move |mut req: Request| {
        let router = router.clone();
        let middlewares = middlewares.clone();
        let state_ext = state_ext.clone();
        let not_found = not_found.clone();

        async move {
            req.extensions_mut().insert(state_ext);

            let handler = match router.find(req.method(), req.uri().path()) {
                Some(matched) => {
                    req.extensions_mut().insert(PathParams(matched.params));
                    matched.handler
                }
                None => not_found,
            };

            let next = Next {
                middlewares,
                handler,
                index: 0,
            };

            let response = next.run(req).await;

            Ok::<_, Infallible>(response)
        }
    });

    let builder = AutoBuilder::new(TokioExecutor::new());
    if let Err(e) = builder.serve_connection(io, service).await {
        tracing::debug!(peer = %peer, error = ?e, "connection ended");
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
