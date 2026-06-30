use api::{
    endpoints::exclusion_zone::{AddZoneRequest, ExclusionZoneResponse},
    types::{
        Point,
        area::{Area, Circle, Line, Polygon},
    },
};
use uuid::Uuid;

#[tokio::test]
async fn list_exclusion_zones_empty_game_returns_empty() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;

    let resp = reqwest::get(format!(
        "{}/api/games/{}/exclusion_zones",
        server.base_url, game.id
    ))
    .await
    .expect("GET /api/games/{id}/exclusion_zones failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: Vec<ExclusionZoneResponse> = resp
        .json()
        .await
        .expect("Response is not valid Vec<ExclusionZoneResponse> JSON");

    assert!(body.is_empty());
}

#[tokio::test]
async fn list_exclusion_zones_returns_seeded_zones() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;

    let z1 = api_tests::seed::seed_exclusion_zone_circle(&server.pool, game.id).await;
    let z2 = api_tests::seed::seed_exclusion_zone_polygon(&server.pool, game.id).await;

    let resp = reqwest::get(format!(
        "{}/api/games/{}/exclusion_zones",
        server.base_url, game.id
    ))
    .await
    .expect("GET /api/games/{id}/exclusion_zones failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: Vec<ExclusionZoneResponse> = resp
        .json()
        .await
        .expect("Response is not valid Vec<ExclusionZoneResponse> JSON");

    assert_eq!(body.len(), 2);

    let found1 = body.iter().find(|z| z.id == z1.id).expect("Zone 1 missing");
    assert_eq!(found1.label, z1.label);
    assert_eq!(found1.exclude_outside, z1.exclude_outside);

    let found2 = body.iter().find(|z| z.id == z2.id).expect("Zone 2 missing");
    assert_eq!(found2.label, z2.label);
    assert_eq!(found2.exclude_outside, z2.exclude_outside);
}

#[tokio::test]
async fn create_exclusion_zone_circle_returns_zone() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;

    let request = AddZoneRequest {
        label: "Downtown circle".to_string(),
        exclude_outside: false,
        area: Area::Circle(Circle {
            center: Point { lat: 59.330, lng: 18.065 },
            radius: 500.0,
        }),
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!(
            "{}/api/games/{}/exclusion_zones",
            server.base_url, game.id
        ))
        .json(&api_tests::exclusion_zone_body(request))
        .send()
        .await
        .expect("POST /api/games/{id}/exclusion_zones failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: ExclusionZoneResponse = resp
        .json()
        .await
        .expect("Response is not valid ExclusionZoneResponse JSON");

    assert!(!body.id.is_nil());
    assert_eq!(body.label, "Downtown circle");
    assert!(!body.exclude_outside);
    assert!(matches!(body.area, Area::Circle(_)));

    if let Area::Circle(c) = body.area {
        assert!((c.center.lat - 59.330).abs() < 1e-9);
        assert!((c.center.lng - 18.065).abs() < 1e-9);
        assert!((c.radius - 500.0).abs() < 1.0);
    }
}

#[tokio::test]
async fn create_exclusion_zone_line_returns_zone() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;

    let request = AddZoneRequest {
        label: "Border line".to_string(),
        exclude_outside: true,
        area: Area::Line(Line {
            start: Point { lat: 59.330, lng: 18.060 },
            end: Point { lat: 59.335, lng: 18.070 },
        }),
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!(
            "{}/api/games/{}/exclusion_zones",
            server.base_url, game.id
        ))
        .json(&api_tests::exclusion_zone_body(request))
        .send()
        .await
        .expect("POST /api/games/{id}/exclusion_zones failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: ExclusionZoneResponse = resp
        .json()
        .await
        .expect("Response is not valid ExclusionZoneResponse JSON");

    assert!(!body.id.is_nil());
    assert_eq!(body.label, "Border line");
    assert!(body.exclude_outside);
    assert!(matches!(body.area, Area::Line(_)));

    if let Area::Line(l) = body.area {
        assert!((l.start.lat - 59.330).abs() < 1e-9);
        assert!((l.end.lng - 18.070).abs() < 1e-9);
    }
}

#[tokio::test]
async fn create_exclusion_zone_polygon_returns_zone() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;

    let vertices = vec![
        Point { lat: 59.330, lng: 18.065 },
        Point { lat: 59.335, lng: 18.065 },
        Point { lat: 59.330, lng: 18.075 },
    ];

    let request = AddZoneRequest {
        label: "Polygon zone".to_string(),
        exclude_outside: false,
        area: Area::Polygon(Polygon { vertices: vertices.clone() }),
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!(
            "{}/api/games/{}/exclusion_zones",
            server.base_url, game.id
        ))
        .json(&api_tests::exclusion_zone_body(request))
        .send()
        .await
        .expect("POST /api/games/{id}/exclusion_zones failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: ExclusionZoneResponse = resp
        .json()
        .await
        .expect("Response is not valid ExclusionZoneResponse JSON");

    assert!(!body.id.is_nil());
    assert_eq!(body.label, "Polygon zone");
    assert!(matches!(body.area, Area::Polygon(_)));

    if let Area::Polygon(p) = body.area {
        assert_eq!(p.vertices.len(), vertices.len());
        for (got, want) in p.vertices.iter().zip(vertices.iter()) {
            assert!((got.lat - want.lat).abs() < 1e-9);
            assert!((got.lng - want.lng).abs() < 1e-9);
        }
    }
}

#[tokio::test]
async fn created_zone_appears_in_list() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;

    let request = AddZoneRequest {
        label: "Test zone".to_string(),
        exclude_outside: false,
        area: Area::Circle(Circle {
            center: Point { lat: 59.330, lng: 18.065 },
            radius: 100.0,
        }),
    };

    let client = reqwest::Client::new();
    let create_resp = client
        .post(format!(
            "{}/api/games/{}/exclusion_zones",
            server.base_url, game.id
        ))
        .json(&api_tests::exclusion_zone_body(request))
        .send()
        .await
        .expect("POST failed");

    assert_eq!(create_resp.status().as_u16(), 200);
    let created: ExclusionZoneResponse = create_resp.json().await.expect("Not valid JSON");

    let list_resp = reqwest::get(format!(
        "{}/api/games/{}/exclusion_zones",
        server.base_url, game.id
    ))
    .await
    .expect("GET failed");

    assert_eq!(list_resp.status().as_u16(), 200);
    let list: Vec<ExclusionZoneResponse> = list_resp.json().await.expect("Not valid JSON");

    let found = list.iter().find(|z| z.id == created.id).expect("Created zone missing from list");
    assert_eq!(found.label, "Test zone");
}

#[tokio::test]
async fn remove_exclusion_zone_deletes_it() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;
    let zone = api_tests::seed::seed_exclusion_zone_circle(&server.pool, game.id).await;

    let client = reqwest::Client::new();
    let del_resp = client
        .delete(format!(
            "{}/api/games/{}/exclusion_zones/{}",
            server.base_url, game.id, zone.id
        ))
        .send()
        .await
        .expect("DELETE failed");

    assert_eq!(del_resp.status().as_u16(), 200);

    let list_resp = reqwest::get(format!(
        "{}/api/games/{}/exclusion_zones",
        server.base_url, game.id
    ))
    .await
    .expect("GET failed");

    let list: Vec<ExclusionZoneResponse> = list_resp.json().await.expect("Not valid JSON");
    assert!(list.iter().all(|z| z.id != zone.id));
}

#[tokio::test]
async fn remove_exclusion_zone_for_wrong_game_returns_error() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;
    let other_game = api_tests::seed::seed_game(&server.pool, map_id).await;
    let zone = api_tests::seed::seed_exclusion_zone_circle(&server.pool, game.id).await;

    let client = reqwest::Client::new();
    let resp = client
        .delete(format!(
            "{}/api/games/{}/exclusion_zones/{}",
            server.base_url, other_game.id, zone.id
        ))
        .send()
        .await
        .expect("DELETE failed");

    assert!(
        resp.status().is_client_error() || resp.status().is_server_error(),
        "Expected error for zone belonging to another game, got {}",
        resp.status(),
    );
}

#[tokio::test]
async fn remove_nonexistent_zone_returns_error() {
    let server = api_tests::spawn_test_server().await;
    let map_id = api_tests::seed::seed_map(&server.pool).await;
    let game = api_tests::seed::seed_game(&server.pool, map_id).await;

    let client = reqwest::Client::new();
    let resp = client
        .delete(format!(
            "{}/api/games/{}/exclusion_zones/{}",
            server.base_url,
            game.id,
            Uuid::new_v4()
        ))
        .send()
        .await
        .expect("DELETE failed");

    assert!(
        resp.status().is_client_error() || resp.status().is_server_error(),
        "Expected error for nonexistent zone, got {}",
        resp.status(),
    );
}
