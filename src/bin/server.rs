use anyhow::Result;

use tcp_tunnel::Server;
use tokio::{io::AsyncWriteExt, task};

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Extension, Path},
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::{borrow::Cow, net::SocketAddr, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let addr = "0.0.0.0:6666";
    let addr = addr.parse::<SocketAddr>()?;
    let server = Arc::new(tcp_tunnel::Server::new(MyAuth {}, addr));

    let server_c = server.clone();
    task::spawn(async move {
        info!("--run--");
        server_c.serve().await.unwrap();
    });

    let app = Router::new()
        .route("/:proto/:addr/:token", get(root))
        .layer(
            ServiceBuilder::new()
                // Handle errors from middleware
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .layer(Extension(server))
                .into_inner(),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

type SharedState = Arc<Server<MyAuth>>;

async fn root(
    Path((proto, addr, token)): Path<(String, String, String)>,
    Extension(server): Extension<SharedState>,
) -> std::result::Result<String, StatusCode> {
    match server.get_stream(&token, &proto, &addr).await {
        Ok(stream) => match stream {
            Some(mut steam) => {
                steam.write_all("hello".as_bytes()).await;
                Ok("ok".to_string())
            }
            None => Err(StatusCode::NOT_FOUND),
        },

        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}

struct MyAuth {}

impl tcp_tunnel::Authorizer for MyAuth {
    fn auth(&self, token: &str) -> bool {
        println!("token: {}", token);

        true
    }
}
