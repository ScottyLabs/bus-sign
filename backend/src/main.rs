//! Entry point for the CMU bus sign backend.
//!
//! This executable loads environment variables, configures the network,
//! and starts the Tokio runtime and Axum web server. When STATIC_DIR is
//! set (or a `static` directory exists next to the binary), the server
//! also serves the frontend as a fallback behind the API routes.

use backend::{AppState, create_router};
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
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

    let app = match find_static_dir() {
        Some(dir) => {
            let index = dir.join("index.html");
            let serve = ServeDir::new(&dir).not_found_service(ServeFile::new(index));
            router.fallback_service(serve)
        }
        None => router,
    };

    let addr = SocketAddr::from((host, port));
    println!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Resolves the static file directory for the frontend.
///
/// Checks `STATIC_DIR` first (for local development), then looks for a
/// `static` directory alongside the binary (the layout produced by the
/// Nix derivation in flake.nix).
fn find_static_dir() -> Option<PathBuf> {
    if let Ok(dir) = env::var("STATIC_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
    }

    let exe = env::current_exe().ok()?;
    let store_dir = exe.parent()?.parent()?;
    let static_dir = store_dir.join("static");
    if static_dir.exists() {
        Some(static_dir)
    } else {
        None
    }
}
