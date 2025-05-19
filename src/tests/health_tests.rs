/**
 * Health Endpoint Tests
 * 
 * This module contains tests for the API health endpoints:
 * - /api/health - Basic service health check
 * - /api/health/rpc - RPC connection health check
 * 
 * These tests verify that:
 * 1. The routes are correctly configured
 * 2. The responses follow the expected structure
 * 3. Error handling works as expected
 */
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use warp::test::request;
    use warp::Filter;
    use crate::client::NamadaClient;
    use crate::AppState;
    use crate::{health_check, rpc_health_check, with_state};
    use serde_json::Value;

    /**
     * Creates a test client with a mock URL.
     * This client won't connect to a real Namada node.
     */
    async fn setup_test_client() -> Arc<AppState> {
        // This is a mock URL that won't be used in tests
        let namada_client = Arc::new(
            NamadaClient::new("http://mock.example.com".to_string())
                .await
                .unwrap_or_else(|_| panic!("Failed to create mock client"))
        );
        
        Arc::new(AppState { namada_client })
    }

    /**
     * Tests the basic health check endpoint.
     * This endpoint should always return 200 OK since it doesn't depend on external services.
     */
    #[tokio::test]
    async fn test_health_check() {
        // Create the filter for testing
        let health_route = warp::path("api")
            .and(warp::path("health"))
            .and(warp::get())
            .and_then(health_check);
        
        // Test the endpoint
        let response = request()
            .method("GET")
            .path("/api/health")
            .reply(&health_route)
            .await;
        
        // Check status code
        assert_eq!(response.status(), 200);
        
        // Parse the response body
        let body: Value = serde_json::from_slice(response.body()).expect("Failed to parse JSON");
        
        // Assert expected response structure
        assert_eq!(body["status"], "ok");
        assert!(body["version"].is_string());
    }

    /**
     * Tests the RPC health check endpoint.
     * Since we're using a mock URL, we expect this to return an error (500 or 503).
     */
    #[tokio::test]
    async fn test_rpc_health_check() {
        let state = setup_test_client().await;
        
        // Create the filter for testing
        let rpc_health_route = warp::path("api")
            .and(warp::path("health"))
            .and(warp::path("rpc"))
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(rpc_health_check);
        
        // Test the endpoint
        let response = request()
            .method("GET")
            .path("/api/health/rpc")
            .reply(&rpc_health_route)
            .await;
        
        // We expect this to fail because we're using a mock URL
        // But we can still check that the response has the right structure
        assert!(response.status() == 500 || response.status() == 503);
    }
} 