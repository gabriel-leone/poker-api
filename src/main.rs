mod models;
mod game;
mod handlers;
mod websocket;

use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tokio::net::TcpListener;

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
        .route("/room", post(handlers::create_room))
        .route("/room/:room_id/join", post(handlers::join_room))
        .route("/room/:room_id/ws", get(websocket::websocket_handler))
        .route("/room/:room_id/start", post(handlers::start_game))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Servidor rodando em http://0.0.0.0:3000");
    
    axum::serve(listener, app).await.unwrap();
}
