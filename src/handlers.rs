use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{models::*, AppState};

pub async fn create_room(
    State(state): State<AppState>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, StatusCode> {
    let room_id = Uuid::new_v4().to_string()[..8].to_string();
    let player_id = Uuid::new_v4().to_string();

    let creator = Player {
        id: player_id.clone(),
        name: request.creator_name,
        chips: 1000, // Fichas iniciais
        hand: Vec::new(),
        current_bet: 0,
        is_folded: false,
        is_all_in: false,
    };

    let mut players = HashMap::new();
    players.insert(player_id.clone(), creator);

    let room = Room {
        id: room_id.clone(),
        creator_id: player_id,
        players,
        game: None,
        max_players: request.max_players.unwrap_or(6),
        websocket_senders: HashMap::new(),
    };

    state.rooms.insert(room_id.clone(), room);

    Ok(Json(CreateRoomResponse { room_id }))
}

pub async fn join_room(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, StatusCode> {
    let mut room = state.rooms.get_mut(&room_id).ok_or(StatusCode::NOT_FOUND)?;

    if room.players.len() >= room.max_players {
        return Ok(Json(JoinRoomResponse {
            success: false,
            message: "Sala lotada".to_string(),
        }));
    }

    if room.game.is_some() {
        return Ok(Json(JoinRoomResponse {
            success: false,
            message: "Jogo já iniciado".to_string(),
        }));
    }

    let player_id = Uuid::new_v4().to_string();
    let player = Player {
        id: player_id.clone(),
        name: request.player_name,
        chips: 1000,
        hand: Vec::new(),
        current_bet: 0,
        is_folded: false,
        is_all_in: false,
    };

    room.players.insert(player_id, player);

    // Notificar outros jogadores via WebSocket
    let message = serde_json::json!({
        "type": "player_joined",
        "data": {
            "players": room.players.values().collect::<Vec<_>>()
        }
    });

    for sender in room.websocket_senders.values() {
        let _ = sender.send(message.to_string());
    }

    Ok(Json(JoinRoomResponse {
        success: true,
        message: "Entrou na sala com sucesso".to_string(),
    }))
}

pub async fn start_game(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut room = state.rooms.get_mut(&room_id).ok_or(StatusCode::NOT_FOUND)?;

    if room.players.len() < 2 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Precisa de pelo menos 2 jogadores para iniciar"
        })));
    }

    if room.game.is_some() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Jogo já iniciado"
        })));
    }    let players: Vec<Player> = room.players.values().cloned().collect();
    let mut game = crate::models::Game::new(players);
    game.start_round();

    let game_state = game.get_game_state();

    // Notificar todos os jogadores via WebSocket
    let message = serde_json::json!({
        "type": "game_started",
        "data": game_state
    });

    for sender in room.websocket_senders.values() {
        let _ = sender.send(message.to_string());
    }

    room.game = Some(game);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Jogo iniciado",
        "game_state": game_state
    })))
}
