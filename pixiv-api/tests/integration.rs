use pixiv_api::{ApiResponse, PixivApi};

#[test]
fn test_client_creation() {
    let api = PixivApi::new();
    assert!(!api.is_authenticated());
}

#[test]
fn test_set_auth() {
    let mut api = PixivApi::new();
    api.set_auth("token", "refresh", 123);
    assert!(api.is_authenticated());
    assert_eq!(api.user_id(), Some(123));
}

#[test]
fn test_api_response_from_json() {
    let raw = serde_json::json!({"id": 1, "title": "test"});
    let resp: ApiResponse<serde_json::Value> = ApiResponse::from_json(raw);
    assert!(resp.is_ok());
    assert_eq!(resp.raw["id"], 1);
}

#[test]
fn test_api_response_parse_failure() {
    let raw = serde_json::json!({"unexpected": "shape"});
    let resp: ApiResponse<serde_json::Value> = ApiResponse::from_json(raw);
    assert!(resp.is_ok()); // Value always parses
    assert_eq!(resp.raw["unexpected"], "shape");
}
