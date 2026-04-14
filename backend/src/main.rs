//! Entry point for the CMU bus sign backend.
//!
//! This executable loads environment variables, configures the network,
//! and starts the Tokio runtime and Axum web servers.

use scottylabs_bus_backend::{AppState, create_router};
use std::env;
use std::net::{IpAddr, SocketAddr};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // 1. load config
    dotenvy::dotenv().ok();

    let api_key = env::var("PRT_API_KEY").expect("PRT_API_KEY must be set in .env");

    let host: String = env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let ip: IpAddr = host.parse().expect("API_HOST must be a valid IP address");

    let port: u16 = env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string()) // default port 8080 if error
        .parse()
        .expect("API_PORT must be a valid port number");

    // 2. init app state (Passing None as base_url uses default live PRT API)
    let state = AppState::new(api_key, None);

    // 3. build router and attach CORS security middleware
    // NOTE: allow(Any) is used for development but not best practice
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    let app = create_router(state).layer(cors);

    // 4. start server
    let addr = SocketAddr::from((ip, port));
    println!("Server started on http://{}/predictions", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
