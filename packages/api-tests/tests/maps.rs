use api::{endpoints::maps::ListMapsResponse, types::map_size::MapSize};

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
