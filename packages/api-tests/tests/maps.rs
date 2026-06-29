use api::{
    endpoints::maps::{CreateMapRequest, GetMapResponse, ListMapsResponse, MapSummary},
    types::{area::Polygon, map_size::MapSize, Point},
};
use uuid::Uuid;

#[tokio::test]
async fn list_maps_empty_db_returns_ok() {
    let server = api_tests::spawn_test_server().await;

    let resp = reqwest::get(format!("{}/api/maps", server.base_url))
        .await
        .expect("GET /api/maps failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: ListMapsResponse = resp
        .json()
        .await
        .expect("Response is not valid ListMapsResponse JSON");

    assert!(body.maps.is_empty());
}

#[tokio::test]
async fn list_maps_returns_seeded_maps() {
    let server = api_tests::spawn_test_server().await;

    let m1 = api_tests::seed::seed_named_map(&server.pool, "Gamla Stan", MapSize::Small).await;
    let m2 = api_tests::seed::seed_named_map(&server.pool, "Djurgården", MapSize::Medium).await;

    let resp = reqwest::get(format!("{}/api/maps", server.base_url))
        .await
        .expect("GET /api/maps failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: ListMapsResponse = resp
        .json()
        .await
        .expect("Response is not valid ListMapsResponse JSON");

    assert_eq!(body.maps.len(), 2);

    let found1 = body.maps.iter().find(|m| m.id == m1.id).expect("Map 1 missing from response");
    assert_eq!(found1.name, m1.name);
    assert_eq!(found1.size, m1.size);

    let found2 = body.maps.iter().find(|m| m.id == m2.id).expect("Map 2 missing from response");
    assert_eq!(found2.name, m2.name);
    assert_eq!(found2.size, m2.size);
}

fn triangle_bounds() -> Polygon {
    Polygon {
        vertices: vec![
            Point { lat: 59.330, lng: 18.065 },
            Point { lat: 59.335, lng: 18.065 },
            Point { lat: 59.330, lng: 18.075 },
        ],
    }
}

#[tokio::test]
async fn create_map_returns_map_summary() {
    let server = api_tests::spawn_test_server().await;

    let request = CreateMapRequest {
        name: "Gamla Stan".to_string(),
        size: MapSize::Small,
        bounds: triangle_bounds(),
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/maps", server.base_url))
        .json(&api_tests::map_body(request))
        .send()
        .await
        .expect("POST /api/maps failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: MapSummary = resp
        .json()
        .await
        .expect("Response is not valid MapSummary JSON");

    assert_eq!(body.name, "Gamla Stan");
    assert_eq!(body.size, MapSize::Small);
    assert!(!body.id.is_nil());
}

#[tokio::test]
async fn created_map_appears_in_list() {
    let server = api_tests::spawn_test_server().await;

    let request = CreateMapRequest {
        name: "Södermalm".to_string(),
        size: MapSize::Large,
        bounds: triangle_bounds(),
    };

    let client = reqwest::Client::new();
    let create_resp = client
        .post(format!("{}/api/maps", server.base_url))
        .json(&api_tests::map_body(request))
        .send()
        .await
        .expect("POST /api/maps failed");

    assert_eq!(create_resp.status().as_u16(), 200);
    let created: MapSummary = create_resp.json().await.expect("Not valid MapSummary JSON");

    let list_resp = reqwest::get(format!("{}/api/maps", server.base_url))
        .await
        .expect("GET /api/maps failed");

    assert_eq!(list_resp.status().as_u16(), 200);
    let list: ListMapsResponse = list_resp.json().await.expect("Not valid ListMapsResponse JSON");

    let found = list.maps.iter().find(|m| m.id == created.id).expect("Created map missing from list");
    assert_eq!(found.name, "Södermalm");
    assert_eq!(found.size, MapSize::Large);
}

#[tokio::test]
async fn get_map_returns_map_with_boundary() {
    let server = api_tests::spawn_test_server().await;

    let vertices = vec![
        Point { lat: 59.330, lng: 18.065 },
        Point { lat: 59.335, lng: 18.065 },
        Point { lat: 59.330, lng: 18.075 },
    ];
    let seeded = api_tests::seed::seed_map_with_boundary(
        &server.pool,
        "Kungsholmen",
        MapSize::Medium,
        vertices.clone(),
    )
    .await;

    let resp = reqwest::get(format!("{}/api/maps/{}", server.base_url, seeded.id))
        .await
        .expect("GET /api/maps/{id} failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: GetMapResponse = resp.json().await.expect("Response is not valid GetMapResponse JSON");

    assert_eq!(body.id, seeded.id);
    assert_eq!(body.name, "Kungsholmen");
    assert_eq!(body.size, MapSize::Medium);
    assert_eq!(body.boundary.vertices.len(), vertices.len());
    for (got, want) in body.boundary.vertices.iter().zip(vertices.iter()) {
        assert!((got.lat - want.lat).abs() < 1e-9);
        assert!((got.lng - want.lng).abs() < 1e-9);
    }
}

#[tokio::test]
async fn get_map_with_nonexistent_id_returns_error() {
    let server = api_tests::spawn_test_server().await;

    let resp = reqwest::get(format!("{}/api/maps/{}", server.base_url, Uuid::new_v4()))
        .await
        .expect("GET /api/maps/{id} failed");

    assert_ne!(resp.status().as_u16(), 200);
}
