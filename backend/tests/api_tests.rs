//! Integration Test Suite for the CMU Bus Sign Backend
//!
//! This module contains end-to-end integration tests for the Axum web server and its
//! underlying data processing logic.
//!
//! ## Testing Strategy
//! To ensure tests are fast, reliable, and run entirely offline, this suite uses the
//! `wiremock` crate to spin up a local mock HTTP server. By injecting the mock server's
//! URI into the `AppState`, we intercept all outbound HTTP requests meant for the Pittsburgh
//! Regional Transit (PRT) API. This prevents the test suite from consuming the 10,000
//! calls/day PRT rate limit and allows us to simulate extreme edge cases.
//!
//! ## Test Coverage
//! The tests in this suite verify the following behaviors:
//!
//! * **Happy Paths:**
//!   * Successfully fetching, parsing, and formatting valid PRT transit data.
//!   * Correctly calculating the seconds remaining until arrival.
//!   * Grouping buses by Stop ID and Route, and sorting them chronologically.
//!
//! * **Error Handling & Resiliency:**
//!   * Returning `500 Internal Server Error` if the upstream API returns malformed JSON/HTML.
//!   * Returning `502 Bad Gateway` if the PRT API returns a native error payload or rejects the API key.
//!   * Stale Cache Fallback: Gracefully ignoring upstream API crashes if cached data is available,
//!     returning `200 OK` to keep the physical bus signs operational.
//!
//! * **Edge Cases & Concurrency:**
//!   * Handling "Middle of the night" scenarios where the API returns zero active buses.
//!   * Skipping individual bus predictions that contain corrupted timestamp strings without crashing.
//!   * Timeout Protection: Ensuring the application aborts hanging upstream connections before Tokio freezes.
//!

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use scottylabs_bus_backend::{AppState, create_router};
use std::time::Duration;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_predictions_with_mocked_prt_api() {
    // 1. Start a background HTTP mock server
    let mock_server = MockServer::start().await;

    // 2. Define exactly what the "fake" PRT API should return
    // Note: We deliberately format tmstmp and prdtm exactly 5 minutes apart
    let fake_prt_json = r#"{
        "bustime-response": {
            "prd": [
                {
                    "rt": "61A",
                    "des": "Swissvale",
                    "stpid": "4407",
                    "vid": "1234",
                    "tmstmp": "20240101 12:00:00",
                    "prdtm": "20240101 12:05:00",
                    "psgld": "HALF_EMPTY"
                }
            ]
        }
    }"#;

    // 3. Tell the mock server to return this JSON whenever it gets a GET request to /getpredictions
    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fake_prt_json))
        .mount(&mock_server)
        .await;

    // 4. Arrange: Setup your app, injecting the Mock Server's dynamic URI!
    let state = AppState::new("FAKE_KEY".to_string(), Some(mock_server.uri()));
    let app = create_router(state);

    let request = Request::builder()
        .uri("/predictions")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // 5. Act: Send the request to your Axum app
    let response = app.oneshot(request).await.unwrap();

    // 6. Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify that our Axum app successfully ingested the fake data, parsed it,
    // did the math (12:05:00 - 12:00:00 = 300 seconds), and output the right format.
    let seconds_away = body_json["4407"][0]["arrivals"][0]["seconds"]
        .as_i64()
        .unwrap();
    assert_eq!(seconds_away, 300, "Should be 5 minutes (300 seconds) away");
}

#[tokio::test]
async fn test_predictions_handles_bad_json() {
    let mock_server = MockServer::start().await;

    // 1. Arrange: Return malformed JSON / HTML instead of the expected structure
    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>Gateway Timeout</html>"))
        .mount(&mock_server)
        .await;

    let state = AppState::new("TEST_KEY".to_string(), Some(mock_server.uri()));
    let app = create_router(state);

    let request = Request::builder()
        .uri("/predictions")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // 2. Act
    let response = app.oneshot(request).await.unwrap();

    // 3. Assert: App should catch the serde_json::Error and return a 500 status
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify the error message format
    assert!(
        body_json["error"]
            .as_str()
            .unwrap()
            .contains("API Parse Error")
    );
}

#[tokio::test]
async fn test_predictions_handles_prt_api_error_no_cache() {
    let mock_server = MockServer::start().await;

    // 1. Arrange: The PRT API returns a 200 OK, but includes an embedded error object
    let error_json = r#"{
        "bustime-response": {
            "error": [
                { "msg": "No data found for parameter stpid" }
            ]
        }
    }"#;

    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(error_json))
        .mount(&mock_server)
        .await;

    let state = AppState::new("TEST_KEY".to_string(), Some(mock_server.uri()));
    let app = create_router(state);

    let request = Request::builder()
        .uri("/predictions")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // 2. Act
    let response = app.oneshot(request).await.unwrap();

    // 3. Assert: Because the cache is empty, this should fail entirely with a 502
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify the API's specific error message was passed down
    assert!(
        body_json["error"]
            .as_str()
            .unwrap()
            .contains("No data found for parameter stpid")
    );
}

#[tokio::test]
async fn test_predictions_stale_cache_fallback() {
    let mock_server = MockServer::start().await;

    // 1. Setup the successful response (to populate the cache)
    let success_json = r#"{
        "bustime-response": {
            "prd": [{
                "rt": "61A", "des": "Swissvale", "stpid": "4407", "vid": "1234",
                "tmstmp": "20240101 12:00:00", "prdtm": "20240101 12:05:00", "psgld": "HALF_EMPTY"
            }]
        }
    }"#;

    // 2. Setup the error response (to simulate the API crashing later)
    let error_json = r#"{
        "bustime-response": {
            "error": [{ "msg": "Transaction limit exceeded" }]
        }
    }"#;

    // Create a mock that returns success on the FIRST call, and an error on the SECOND call
    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_json))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(error_json))
        .mount(&mock_server)
        .await;

    let state = AppState::new("TEST_KEY".to_string(), Some(mock_server.uri()));
    let app = create_router(state.clone());

    // --- First Request: Should succeed and populate cache ---
    let request_1 = Request::builder()
        .uri("/predictions")
        .body(Body::empty())
        .unwrap();
    let response_1 = app.clone().oneshot(request_1).await.unwrap();
    assert_eq!(response_1.status(), StatusCode::OK);

    // --- Wait for cache to "age" slightly to force an API call ---
    // (In your logic, CACHE_DURATION is 20s. To strictly test the fallback without sleeping for 20s,
    // we can manually reach into the state and alter the last_update timestamp to simulate time passing).
    {
        let mut cache = state.cache.lock().await;
        // Move the timestamp back 30 seconds so the app thinks the cache is stale
        cache.last_update = Some(chrono::Utc::now() - chrono::Duration::seconds(30));
    }

    // --- Second Request: Should hit the API, get an error, but return a 200 OK via cache ---
    let request_2 = Request::builder()
        .uri("/predictions")
        .body(Body::empty())
        .unwrap();
    let response_2 = app.clone().oneshot(request_2).await.unwrap();

    // Assert: Even though the API failed, we should still get a 200 OK
    assert_eq!(
        response_2.status(),
        StatusCode::OK,
        "Fallback failed: Expected 200 OK despite API error"
    );

    // Extract the body and ensure it contains the stale "61A" route data
    let body_bytes = axum::body::to_bytes(response_2.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(
        body_json.get("4407").is_some(),
        "Fallback response should contain the cached data"
    );
}

#[tokio::test]
async fn test_predictions_empty_buses_at_night() {
    let mock_server = MockServer::start().await;

    // 1. Arrange: API succeeds but returns NO predictions (empty array or missing field)
    let empty_json = r#"{
        "bustime-response": {
            "prd": []
        }
    }"#;

    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(empty_json))
        .mount(&mock_server)
        .await;

    let state = AppState::new("TEST_KEY".to_string(), Some(mock_server.uri()));
    let app = create_router(state);

    let request = Request::builder()
        .uri("/predictions")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // 2. Act
    let response = app.oneshot(request).await.unwrap();

    // 3. Assert: Should return 200 OK with an empty dictionary
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body_json.is_object());
    assert_eq!(
        body_json.as_object().unwrap().len(),
        0,
        "Expected empty JSON object"
    );
}

#[tokio::test]
async fn test_predictions_skips_malformed_timestamps() {
    let mock_server = MockServer::start().await;

    // 1. Arrange: One good bus, and one bus with a completely busted timestamp
    let bad_time_json = r#"{
        "bustime-response": {
            "prd": [
                {
                    "rt": "61A", "des": "Swissvale", "stpid": "4407", "vid": "GOOD_BUS",
                    "tmstmp": "20240101 12:00:00", "prdtm": "20240101 12:05:00", "psgld": "HALF_EMPTY"
                },
                {
                    "rt": "61B", "des": "Braddock", "stpid": "4407", "vid": "BAD_BUS",
                    "tmstmp": "20240101 12:00:00", "prdtm": "GARBAGE_STRING", "psgld": "EMPTY"
                }
            ]
        }
    }"#;

    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(bad_time_json))
        .mount(&mock_server)
        .await;

    let state = AppState::new("TEST_KEY".to_string(), Some(mock_server.uri()));
    let app = create_router(state);

    let request = Request::builder()
        .uri("/predictions")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    // 2. Assert: It should return 200 OK, skipping the bad bus and keeping the good one
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // We should only have 1 route group (61A) because 61B was skipped
    let stop_4407 = body_json.get("4407").unwrap().as_array().unwrap();
    assert_eq!(stop_4407.len(), 1);
    assert_eq!(stop_4407[0]["route"], "61A");
}

#[tokio::test]
async fn test_predictions_complex_grouping_and_sorting() {
    let mock_server = MockServer::start().await;

    // 1. Arrange: Multiple stops, multiple routes, unsorted arrival times
    let complex_json = r#"{
        "bustime-response": {
            "prd": [
                {
                    "rt": "61A", "des": "Swissvale", "stpid": "4407", "vid": "BUS_1",
                    "tmstmp": "20240101 12:00:00", "prdtm": "20240101 12:10:00"
                },
                {
                    "rt": "61A", "des": "Swissvale", "stpid": "4407", "vid": "BUS_2_SOONER",
                    "tmstmp": "20240101 12:00:00", "prdtm": "20240101 12:02:00"
                },
                {
                    "rt": "61B", "des": "Braddock", "stpid": "4407", "vid": "BUS_3",
                    "tmstmp": "20240101 12:00:00", "prdtm": "20240101 12:05:00"
                },
                {
                    "rt": "71A", "des": "Negley", "stpid": "7117", "vid": "BUS_4",
                    "tmstmp": "20240101 12:00:00", "prdtm": "20240101 12:08:00"
                }
            ]
        }
    }"#;

    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(complex_json))
        .mount(&mock_server)
        .await;

    let state = AppState::new("TEST_KEY".to_string(), Some(mock_server.uri()));
    let app = create_router(state);

    let request = Request::builder()
        .uri("/predictions")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    // 2. Assert
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Check Stop 7117 (Should have 1 route)
    let stop_7117 = body_json
        .get("7117")
        .expect("Stop 7117 missing")
        .as_array()
        .unwrap();
    assert_eq!(stop_7117.len(), 1);
    assert_eq!(stop_7117[0]["route"], "71A");

    // Check Stop 4407 (Should have 2 routes: 61A and 61B)
    let stop_4407 = body_json
        .get("4407")
        .expect("Stop 4407 missing")
        .as_array()
        .unwrap();
    assert_eq!(stop_4407.len(), 2);

    // Find the 61A group to test sorting
    let route_61a = stop_4407
        .iter()
        .find(|g| g["route"] == "61A")
        .expect("61A missing");
    let arrivals = route_61a["arrivals"].as_array().unwrap();

    // There were two 61A buses. BUS_2_SOONER was 2 mins away (120s). BUS_1 was 10 mins away (600s).
    // They appeared out of order in the JSON, but your code should have sorted them!
    assert_eq!(arrivals.len(), 2);
    assert_eq!(arrivals[0]["bus_id"], "BUS_2_SOONER");
    assert_eq!(arrivals[1]["bus_id"], "BUS_1");
}

#[tokio::test]
async fn test_predictions_upstream_timeout() {
    let mock_server = MockServer::start().await;

    // 1. Arrange: Tell the mock server to take 10 seconds to respond
    Mock::given(method("GET"))
        .and(path("/getpredictions"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
        .mount(&mock_server)
        .await;

    let state = AppState::new("TEST_KEY".to_string(), Some(mock_server.uri()));
    let app = create_router(state);

    let request = Request::builder()
        .uri("/predictions")
        .body(Body::empty())
        .unwrap();

    // 2. Act: Wrap the request in a Tokio timeout of 6 seconds
    // (longer than our client timeout of 5 seconds)
    let result = tokio::time::timeout(Duration::from_secs(6), app.oneshot(request)).await;

    // 3. Assert: Unwrap the timeout Result, AND unwrap the Axum Result
    let response = result
        .expect("App hung indefinitely! You need an HTTP client timeout.")
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}
