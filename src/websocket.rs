use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;

use crate::{models::*, AppState};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Enviar mensagens do canal para o WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Processar mensagens recebidas do WebSocket
    let room_id_clone = room_id.clone();
    let state_clone = state.clone();
    let tx_clone = tx.clone();
    
    let recv_task = tokio::spawn(async move {
        let mut player_id: Option<String> = None;

        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                if let Message::Text(text) = msg {
                    if let Ok(ws_message) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match ws_message.message_type.as_str() {
                            "join" => {
                                if let Ok(join_data) = serde_json::from_value::<serde_json::Value>(ws_message.data) {
                                    if let Some(pid) = join_data.get("player_id").and_then(|v| v.as_str()) {
                                        player_id = Some(pid.to_string());
                                        
                                        // Adicionar sender do WebSocket à sala
                                        if let Some(mut room) = state_clone.rooms.get_mut(&room_id_clone) {
                                            room.websocket_senders.insert(pid.to_string(), tx_clone.clone());
                                            
                                            // Enviar estado atual da sala
                                            let room_state = serde_json::json!({
                                                "type": "room_state",
                                                "data": {
                                                    "room_id": room.id,
                                                    "players": room.players.values().collect::<Vec<_>>(),
                                                    "game": room.game.as_ref().map(|g| g.get_game_state())
                                                }
                                            });
                                            
                                            let _ = tx_clone.send(room_state.to_string());
                                        }
                                    }
                                }
                            }
                            "game_action" => {
                                if let Some(ref pid) = player_id {
                                    if let Ok(action_data) = serde_json::from_value::<GameActionMessage>(ws_message.data) {
                                        handle_game_action(&state_clone, &room_id_clone, pid, action_data.action).await;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                break;
            }
        }

        // Remove o jogador quando desconectar
        if let Some(pid) = player_id {
            if let Some(mut room) = state_clone.rooms.get_mut(&room_id_clone) {
                room.websocket_senders.remove(&pid);
            }
        }
    });

    // Aguardar qualquer uma das tasks terminar
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

async fn handle_game_action(
    state: &AppState,
    room_id: &str,
    player_id: &str,
    action: PlayerAction,
) {
    if let Some(mut room) = state.rooms.get_mut(room_id) {
        // Primeiro, coletar todos os senders
        let senders: Vec<_> = room.websocket_senders.values().cloned().collect();
        let player_sender = room.websocket_senders.get(player_id).cloned();
        
        if let Some(ref mut game) = room.game {
            match game.process_action(player_id, action) {
                Ok(()) => {
                    // Enviar estado atualizado do jogo para todos os jogadores
                    let game_state = game.get_game_state();
                    let message = serde_json::json!({
                        "type": "game_update",
                        "data": game_state
                    });

                    for sender in &senders {
                        let _ = sender.send(message.to_string());
                    }

                    // Se o jogo terminou, iniciar uma nova rodada após um delay
                    let is_game_finished = matches!(game.state, GameState::Finished);
                    let pot = game.pot;
                    
                    if is_game_finished {
                        let message = serde_json::json!({
                            "type": "round_finished",
                            "data": {
                                "winner": "TBD", // Implementar lógica de vencedor
                                "pot": pot
                            }
                        });

                        for sender in &senders {
                            let _ = sender.send(message.to_string());
                        }

                        // Aguardar 5 segundos e iniciar nova rodada
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        
                        // Verificar se ainda há jogadores suficientes
                        let active_players_count = game.players.iter()
                            .filter(|p| p.chips > 0)
                            .count();

                        if active_players_count >= 2 {
                            game.dealer_index = (game.dealer_index + 1) % game.players.len();
                            game.start_round();

                            let game_state = game.get_game_state();
                            let message = serde_json::json!({
                                "type": "new_round",
                                "data": game_state
                            });

                            for sender in &senders {
                                let _ = sender.send(message.to_string());
                            }
                        }
                    }
                }
                Err(error) => {
                    // Enviar erro para o jogador específico
                    if let Some(sender) = player_sender {
                        let error_message = serde_json::json!({
                            "type": "error",
                            "data": {
                                "message": error
                            }
                        });
                        let _ = sender.send(error_message.to_string());
                    }
                }
            }
        }
    }
}
