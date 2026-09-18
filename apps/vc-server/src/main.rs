mod routes;
mod state;
mod ws;

use axum::{
    routing::{get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vc_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new();

    // CORS configuration for local web development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(routes::health_check))
        .route("/api/health", get(routes::health_check))
        .route("/api/character", get(routes::get_character))
        .route("/api/personality", put(routes::update_personality).post(routes::update_personality))
        .route("/api/character/personality", put(routes::update_personality).post(routes::update_personality))
        .route("/api/character/memories", get(routes::get_memories))
        .route("/api/character/reset", post(routes::reset_character))
        .route("/ws/interaction", get(ws::ws_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       VirtualCharacter Backend Gateway (vc-server)           ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  HTTP REST API:    http://127.0.0.1:{:<25} ║", format!("{}/api/character", port));
    println!("║  WebSocket Stream: ws://127.0.0.1:{:<27} ║", format!("{}/ws/interaction", port));
    println!("╚══════════════════════════════════════════════════════════════╝");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");
    axum::serve(listener, app)
        .await
        .expect("Server encountered a fatal error");
}
