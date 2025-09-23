use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::api_key_service::ApiKeyService;

/// Authentication context for request processing
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub deployment_id: String,
    pub api_key_id: String,
}

/// Authorization middleware for API key validation
pub async fn auth_middleware(
    State(api_key_service): State<Arc<ApiKeyService>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract API key from Authorization header
    let api_key = match extract_api_key_from_headers(&headers) {
        Some(key) => key,
        None => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Validate the API key
    let deployment_id = match api_key_service.validate_api_key(&api_key).await {
        Ok(Some(deployment_id)) => deployment_id,
        Ok(None) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get the API key record to extract the ID
    let key_hash = api_key_service.hash_api_key(&api_key);
    let api_key_record = match api_key_service.database.get_api_key_by_hash(&key_hash).await {
        Ok(Some(record)) => record,
        Ok(None) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create auth context and insert into request extensions
    let auth_context = AuthContext {
        deployment_id,
        api_key_id: api_key_record.id,
    };

    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

/// Extract API key from various header formats
pub fn extract_api_key_from_headers(headers: &HeaderMap) -> Option<String> {
    // Try Authorization header with Bearer token
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(key) = auth_str.strip_prefix("Bearer ") {
                return Some(key.to_string());
            }
            if let Some(key) = auth_str.strip_prefix("ApiKey ") {
                return Some(key.to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = headers.get("X-API-Key") {
        if let Ok(key) = api_key_header.to_str() {
            return Some(key.to_string());
        }
    }

    // Try X-Auth-Token header
    if let Some(auth_token_header) = headers.get("X-Auth-Token") {
        if let Ok(key) = auth_token_header.to_str() {
            return Some(key.to_string());
        }
    }

    None
}

/// Helper function to extract auth context from request
pub fn get_auth_context(request: &Request) -> Option<&AuthContext> {
    request.extensions().get::<AuthContext>()
}

/// Helper function to check if request is authorized
pub fn is_authorized(request: &Request) -> bool {
    get_auth_context(request).is_some()
}

/// Helper function to get deployment ID from auth context
pub fn get_deployment_id(request: &Request) -> Option<&str> {
    get_auth_context(request).map(|ctx| ctx.deployment_id.as_str())
}

/// Helper function to get API key ID from auth context
pub fn get_api_key_id(request: &Request) -> Option<&str> {
    get_auth_context(request).map(|ctx| ctx.api_key_id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_api_key_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("Bearer sk_test123"));
        
        let api_key = extract_api_key_from_headers(&headers);
        assert_eq!(api_key, Some("sk_test123".to_string()));
    }

    #[test]
    fn test_extract_api_key_apikey() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("ApiKey sk_test123"));
        
        let api_key = extract_api_key_from_headers(&headers);
        assert_eq!(api_key, Some("sk_test123".to_string()));
    }

    #[test]
    fn test_extract_api_key_x_header() {
        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_static("sk_test123"));
        
        let api_key = extract_api_key_from_headers(&headers);
        assert_eq!(api_key, Some("sk_test123".to_string()));
    }

    #[test]
    fn test_extract_api_key_x_auth_token() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Auth-Token", HeaderValue::from_static("sk_test123"));
        
        let api_key = extract_api_key_from_headers(&headers);
        assert_eq!(api_key, Some("sk_test123".to_string()));
    }

    #[test]
    fn test_extract_api_key_no_headers() {
        let headers = HeaderMap::new();
        let api_key = extract_api_key_from_headers(&headers);
        assert_eq!(api_key, None);
    }

    #[test]
    fn test_extract_api_key_invalid_format() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("Invalid sk_test123"));
        
        let api_key = extract_api_key_from_headers(&headers);
        assert_eq!(api_key, None);
    }
}
