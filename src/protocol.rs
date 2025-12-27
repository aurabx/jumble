//! MCP Protocol types for JSON-RPC communication.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_request_with_params() {
        let json_str = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": "list_projects", "arguments": {}}
        }"#;

        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.method, "tools/call");
        assert_eq!(request.id, Some(json!(1)));
        assert_eq!(request.params["name"], "list_projects");
    }

    #[test]
    fn test_parse_request_without_params() {
        let json_str = r#"{
            "jsonrpc": "2.0",
            "id": "abc-123",
            "method": "initialize"
        }"#;

        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.method, "initialize");
        assert_eq!(request.id, Some(json!("abc-123")));
        assert!(request.params.is_null());
    }

    #[test]
    fn test_parse_notification_no_id() {
        let json_str = r#"{
            "jsonrpc": "2.0",
            "method": "initialized"
        }"#;

        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.method, "initialized");
        assert!(request.id.is_none());
    }

    #[test]
    fn test_success_response_serialization() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!({"status": "ok"}));
        let serialized = serde_json::to_string(&response).unwrap();
        
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("\"result\""));
        assert!(!serialized.contains("\"error\""));
    }

    #[test]
    fn test_error_response_serialization() {
        let error = JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };
        let response = JsonRpcResponse::error(Some(json!(1)), error);
        let serialized = serde_json::to_string(&response).unwrap();
        
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"error\""));
        assert!(serialized.contains("-32601"));
        assert!(!serialized.contains("\"result\""));
    }

    #[test]
    fn test_error_with_data() {
        let error = JsonRpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({"field": "project", "reason": "required"})),
        };
        let response = JsonRpcResponse::error(Some(json!("req-1")), error);
        let serialized = serde_json::to_string(&response).unwrap();
        
        assert!(serialized.contains("\"data\""));
        assert!(serialized.contains("\"field\":\"project\""));
    }
}
