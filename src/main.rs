mod game;
mod handlers;
mod models;
mod websocket;

#[cfg(test)]
mod integration_tests;

use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::models::*;

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<DashMap<String, Room>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        rooms: Arc::new(DashMap::new()),
    };

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/room", post(handlers::create_room))
        .route("/room/:room_id/join", post(handlers::join_room))
        .route("/room/:room_id/ws", get(websocket::websocket_handler))
        .route("/room/:room_id/start", post(handlers::start_game))
        .route("/room/:room_id/result", get(handlers::get_hand_result))
        .route("/room/:room_id/next", post(handlers::next_hand))
        .with_state(state)
        .layer(CorsLayer::permissive());

    // Configura a porta via variável de ambiente (necessário para o Render)
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Servidor rodando em http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
