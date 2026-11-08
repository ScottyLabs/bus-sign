//! Entry point for the CMU bus sign backend.
//!
//! This executable loads environment variables, configures the network,
//! and starts the Tokio runtime and Axum web server. When `STATIC_DIR` is set, the server also serves the frontend as a fallback behind the API routes.

use backend::{AppState, create_router};
use std::env;
use std::net::{IpAddr, SocketAddr};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let prt_api_key = env::var("PRT_API_KEY").expect("PRT_API_KEY must be set");
    let openweather_api_key =
        env::var("OPENWEATHER_API_KEY").expect("OPENWEATHER_API_KEY must be set");

    let host: IpAddr = env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()
        .expect("HOST must be a valid IP address");

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid port number");

    let state = AppState::new(prt_api_key, openweather_api_key);
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);
    let router = create_router(state).layer(cors);

    let app = match env::var("STATIC_DIR") {
        Ok(dir) => {
            let index = format!("{dir}/index.html");
            let serve = ServeDir::new(&dir).not_found_service(ServeFile::new(index));
            router.fallback_service(serve)
        }
        Err(_) => router,
    };

    let addr = SocketAddr::from((host, port));
    println!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
