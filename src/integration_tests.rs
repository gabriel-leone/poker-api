#[cfg(test)]
mod tests {
    use crate::{handlers, AppState};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{get, post},
        Router,
    };
    use dashmap::DashMap;
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;
    use tower_http::cors::CorsLayer;

    async fn create_test_app() -> Router {
        let state = AppState {
            rooms: Arc::new(DashMap::new()),
        };

        Router::new()
            .route("/health", get(handlers::health_check))
            .route("/room", post(handlers::create_room))
            .route("/room/:room_id/join", post(handlers::join_room))
            .with_state(state)
            .layer(CorsLayer::permissive())
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_room() {
        let app = create_test_app().await;

        let request_body = json!({
            "creator_name": "TestPlayer"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/room")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_join_nonexistent_room() {
        let app = create_test_app().await;

        let request_body = json!({
            "player_name": "TestPlayer"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/room/nonexistent/join")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
