use api::{
    endpoints::game::{CreateGameRequest, CreateGameResponse, GameResponse},
    types::game_status::GameStatus,
};
use uuid::Uuid;

#[tokio::test]
async fn create_game_with_valid_map_returns_game_code() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/games", server.base_url))
        .json(&api_tests::game_body(CreateGameRequest {
            map_id,
            host_display_name: "Test Host".to_string(),
        }))
        .send()
        .await
        .expect("POST /api/games failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: CreateGameResponse = resp
        .json()
        .await
        .expect("Response body is not valid CreateGameResponse JSON");

    assert_eq!(
        body.game_code.len(),
        8,
        "game_code should be 8 hex characters"
    );
    assert!(
        body.game_code.chars().all(|c| c.is_ascii_hexdigit()),
        "game_code should be all hex digits, got: {}",
        body.game_code,
    );
    assert_ne!(body.game_id, Uuid::nil());
}

#[tokio::test]
async fn create_game_with_nonexistent_map_returns_error() {
    let server = api_tests::spawn_test_server().await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/games", server.base_url))
        .json(&api_tests::game_body(CreateGameRequest {
            map_id: Uuid::new_v4(), // definitely not in the DB
            host_display_name: "Test Host".to_string(),
        }))
        .send()
        .await
        .expect("POST /api/games failed");

    assert!(
        resp.status().is_client_error() || resp.status().is_server_error(),
        "Expected 4xx/5xx for a nonexistent map_id, got {}",
        resp.status(),
    );
}

#[tokio::test]
async fn get_game_returns_game_details() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let seeded = api_tests::seed::seed_game(&server.pool, map_id).await;

    let resp = reqwest::get(format!("{}/api/games/{}", server.base_url, seeded.id))
        .await
        .expect("GET /api/games/{id} failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: GameResponse = resp
        .json()
        .await
        .expect("Response is not valid GetGameResponse JSON");

    assert_eq!(body.game_id, seeded.id);
    assert_eq!(body.game_code, seeded.code.to_string());
    assert_eq!(body.map_id, map_id);
    assert_eq!(body.status, GameStatus::Lobby);
}

#[tokio::test]
async fn get_game_with_nonexistent_id_returns_error() {
    let server = api_tests::spawn_test_server().await;

    let resp = reqwest::get(format!("{}/api/games/{}", server.base_url, Uuid::new_v4()))
        .await
        .expect("GET /api/games/{id} failed");

    assert_ne!(resp.status().as_u16(), 200);
}
