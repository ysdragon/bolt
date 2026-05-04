// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Authentication: JWT, CSRF, Basic Auth

use super::{HttpServer, PendingResponse, ResponseBody};
use crate::ring_list_to_json;
use ring_lang_rs::*;
use serde::{Deserialize, Serialize};

use crate::HTTP_SERVER_TYPE;

// ========================================
// CSRF
// ========================================

/// bolt_enable_csrf(server, secret) - enable CSRF protection
ring_func!(bolt_enable_csrf, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let secret = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.csrf_secret = Some(secret.to_string());
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_csrf_token(server) -> generate signed CSRF token (session_id.timestamp.hmac)
ring_func!(bolt_csrf_token, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "");
        return;
    }

    let secret = unsafe {
        let server = &*(ptr as *const HttpServer);
        server.csrf_secret.clone().unwrap_or_default()
    };
    if secret.is_empty() {
        ring_ret_string!(p, "");
        return;
    }

    let session_id = unsafe {
        let server = &*(ptr as *mut HttpServer);
        let guard = server.current_request.lock();
        guard
            .as_ref()
            .map(|ctx| ctx.session_id.clone())
            .unwrap_or_default()
    };

    if session_id.is_empty() {
        ring_ret_string!(p, "");
        return;
    }

    // Ensure BOLTSESSION cookie is set in response so client returns it
    unsafe {
        let server = &*(ptr as *const HttpServer);
        let has_session_cookie = {
            let guard = server.current_request.lock();
            guard
                .as_ref()
                .map(|ctx| ctx.cookies.contains_key("BOLTSESSION"))
                .unwrap_or(false)
        };
        if !has_session_cookie {
            let cookie_val = format!("BOLTSESSION={}; Path=/; HttpOnly", session_id);
            let mut response = server.current_response.lock();
            if let Some(ref mut res) = *response {
                if !res.cookies.iter().any(|c| c.starts_with("BOLTSESSION=")) {
                    res.cookies.push(cookie_val);
                }
            } else {
                *response = Some(PendingResponse {
                    status: 200,
                    headers: std::collections::HashMap::new(),
                    cookies: vec![cookie_val],
                    body: ResponseBody::Bytes(Vec::new()),
                    only_headers: true,
                });
            }
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let payload = format!("{}.{}", session_id, timestamp);

    type HmacSha256 = hmac::Hmac<sha2::Sha256>;
    use hmac::Mac;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let hmac_hex = hex::encode(result.into_bytes());
    let token = format!("{}.{}", payload, &hmac_hex[..16]);
    ring_ret_string!(p, &token);
});

/// bolt_verify_csrf(server, token) -> 1 if valid (checks session_id + HMAC + 1h expiry)
ring_func!(bolt_verify_csrf, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let token = ring_get_string!(p, 2);
    if token.is_empty() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let secret = unsafe {
        let server = &*(ptr as *const HttpServer);
        server.csrf_secret.clone().unwrap_or_default()
    };
    if secret.is_empty() {
        ring_ret_number!(p, 0.0);
        return;
    }

    // Parse: session_id.timestamp.hmac(16)
    let last_dot = match token.rfind('.') {
        Some(pos) => pos,
        None => {
            ring_ret_number!(p, 0.0);
            return;
        }
    };
    let provided_sig = &token[last_dot + 1..];
    let payload = &token[..last_dot];

    if provided_sig.len() != 16 {
        ring_ret_number!(p, 0.0);
        return;
    }

    let payload_dot = match payload.rfind('.') {
        Some(pos) => pos,
        None => {
            ring_ret_number!(p, 0.0);
            return;
        }
    };
    let token_session_id = &payload[..payload_dot];
    let timestamp: u64 = match payload[payload_dot + 1..].parse() {
        Ok(t) => t,
        Err(_) => {
            ring_ret_number!(p, 0.0);
            return;
        }
    };

    // Verify session binding
    let current_session_id = unsafe {
        let server = &*(ptr as *mut HttpServer);
        let guard = server.current_request.lock();
        guard
            .as_ref()
            .map(|ctx| ctx.session_id.clone())
            .unwrap_or_default()
    };

    if current_session_id.is_empty() || token_session_id != current_session_id {
        ring_ret_number!(p, 0.0);
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if now.saturating_sub(timestamp) > 3600 {
        ring_ret_number!(p, 0.0);
        return;
    }

    type HmacSha256 = hmac::Hmac<sha2::Sha256>;
    use hmac::Mac;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    let expected_sig = &sig[..16];

    if provided_sig.len() != expected_sig.len() {
        ring_ret_number!(p, 0.0);
        return;
    }
    let mut diff: u8 = 0;
    for (a, b) in provided_sig.bytes().zip(expected_sig.bytes()) {
        diff |= a ^ b;
    }
    ring_ret_number!(p, if diff == 0 { 1.0 } else { 0.0 });
});

// ========================================
// JWT
// ========================================

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    #[serde(flatten)]
    data: serde_json::Value,
    exp: Option<u64>,
    iat: Option<u64>,
}

/// bolt_jwt_encode(data, secret, expires_in_seconds) -> token  (data can be list or json string)
ring_func!(bolt_jwt_encode, |p| {
    ring_check_paracount_range!(p, 2, 3);
    ring_check_string!(p, 2);

    let data_json = if ring_api_islist(p, 1) {
        let list = ring_api_getlist(p, 1);
        let value = ring_list_to_json(list);
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    } else {
        ring_get_string!(p, 1).to_string()
    };
    let secret = ring_get_string!(p, 2);

    let expires_in = if ring_api_paracount(p) >= 3 && ring_api_isnumber(p, 3) {
        Some(ring_get_number!(p, 3) as u64)
    } else {
        None
    };

    let data: serde_json::Value = serde_json::from_str(&data_json)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = JwtClaims {
        data,
        iat: Some(now),
        exp: expires_in.map(|e| now + e),
    };

    match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    ) {
        Ok(token) => ring_ret_string!(p, &token),
        Err(e) => {
            ring_error!(p, &format!("JWT encode error: {}", e));
        }
    }
});

/// bolt_jwt_decode(token, secret) -> json data or empty on error
ring_func!(bolt_jwt_decode, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let token = ring_get_string!(p, 1);
    let secret = ring_get_string!(p, 2);

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.clear();

    match jsonwebtoken::decode::<JwtClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            let json = serde_json::to_string(&token_data.claims.data).unwrap_or_default();
            ring_ret_string!(p, &json);
        }
        Err(_) => {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_jwt_verify(token, secret) -> 1 if valid, 0 if invalid
ring_func!(bolt_jwt_verify, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let token = ring_get_string!(p, 1);
    let secret = ring_get_string!(p, 2);

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.clear();

    match jsonwebtoken::decode::<JwtClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(_) => ring_ret_number!(p, 1.0),
        Err(_) => {
            ring_ret_number!(p, 0.0);
        }
    }
});

// ========================================
// Basic Auth
// ========================================

/// bolt_basic_auth_decode(auth_header) -> json with username and password
ring_func!(bolt_basic_auth_decode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let auth_header = ring_get_string!(p, 1);

    let encoded = auth_header.strip_prefix("Basic ").unwrap_or(auth_header);

    use base64::Engine;
    let decoded = match base64::engine::general_purpose::STANDARD.decode(encoded) {
        Ok(d) => d,
        Err(_) => {
            ring_ret_string!(p, "");
            return;
        }
    };

    let credentials = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => {
            ring_ret_string!(p, "");
            return;
        }
    };

    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if parts.len() != 2 {
        ring_ret_string!(p, "");
        return;
    }

    let result = serde_json::json!({
        "username": parts[0],
        "password": parts[1]
    });

    ring_ret_string!(p, &result.to_string());
});

/// bolt_basic_auth_encode(username, password) -> "Basic xxx" header value
ring_func!(bolt_basic_auth_encode, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let username = ring_get_string!(p, 1);
    let password = ring_get_string!(p, 2);

    use base64::Engine;
    let credentials = format!("{}:{}", username, password);
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

    ring_ret_string!(p, &format!("Basic {}", encoded));
});

#[cfg(test)]
mod tests {
    use base64::Engine;
    use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestJwtClaims {
        #[serde(flatten)]
        data: serde_json::Value,
        exp: Option<u64>,
        iat: Option<u64>,
    }

    #[test]
    fn test_jwt_encode_decode_roundtrip() {
        let data = serde_json::json!({"user_id": 42, "role": "admin"});
        let secret = "test-secret-key-12345678901234567890123456789012";

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = TestJwtClaims {
            data: data.clone(),
            iat: Some(now),
            exp: Some(now + 3600),
        };

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims.clear();

        let decoded = jsonwebtoken::decode::<TestJwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .unwrap();

        assert_eq!(decoded.claims.data, data);
    }

    #[test]
    fn test_jwt_decode_wrong_secret_fails() {
        let data = serde_json::json!({"user_id": 42});
        let secret1 = "secret-one-12345678901234567890123456789012";
        let secret2 = "secret-two-12345678901234567890123456789012";

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = TestJwtClaims {
            data,
            iat: Some(now),
            exp: Some(now + 3600),
        };

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret1.as_bytes()),
        )
        .unwrap();

        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims.clear();

        let result = jsonwebtoken::decode::<TestJwtClaims>(
            &token,
            &DecodingKey::from_secret(secret2.as_bytes()),
            &validation,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_decode_expired_token_fails() {
        let data = serde_json::json!({"user_id": 42});
        let secret = "test-secret-key-12345678901234567890123456789012";

        let past = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 7200;
        let claims = TestJwtClaims {
            data,
            iat: Some(past),
            exp: Some(past + 1),
        };

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims.clear();

        let result = jsonwebtoken::decode::<TestJwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_auth_encode_decode() {
        let username = "admin";
        let password = "secret123";

        let credentials = format!("{}:{}", username, password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        let header_value = format!("Basic {}", encoded);

        let decoded_b64 = header_value.strip_prefix("Basic ").unwrap_or(&header_value);
        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(decoded_b64)
            .unwrap();
        let decoded_str = String::from_utf8(decoded_bytes).unwrap();

        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], username);
        assert_eq!(parts[1], password);
    }

    #[test]
    fn test_basic_auth_decode_malformed() {
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"nocolonhere");
        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        let decoded_str = String::from_utf8(decoded_bytes).unwrap();

        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_basic_auth_decode_invalid_base64() {
        let result = base64::engine::general_purpose::STANDARD.decode("!!!invalid!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_auth_decode_invalid_utf8() {
        let result = String::from_utf8(vec![0xff, 0xfe, 0x80]);
        assert!(result.is_err());
    }

    // Security tests

    #[test]
    fn test_jwt_tampered_signature_fails() {
        let data = serde_json::json!({"user_id": 42});
        let secret = "test-secret-key-12345678901234567890123456789012";

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = TestJwtClaims {
            data,
            iat: Some(now),
            exp: Some(now + 3600),
        };

        let mut token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        // Tamper with the payload (second segment)
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        let tampered_payload =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"user_id":999}"#);
        token = format!("{}.{}.{}", parts[0], tampered_payload, parts[2]);

        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims.clear();

        let result = jsonwebtoken::decode::<TestJwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_none_algorithm_rejected() {
        let secret = "test-secret-key-12345678901234567890123456789012";

        // Craft a token with "alg": "none" header
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"alg":"none","typ":"JWT"}"#);
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"user_id":42}"#);
        let forged = format!("{header}.{payload}.");

        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims.clear();

        let result = jsonwebtoken::decode::<TestJwtClaims>(
            &forged,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_empty_token_fails() {
        let secret = "test-secret-key-12345678901234567890123456789012";
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims.clear();

        let result = jsonwebtoken::decode::<TestJwtClaims>(
            "",
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_malformed_token_fails() {
        let secret = "test-secret-key-12345678901234567890123456789012";
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims.clear();

        let result = jsonwebtoken::decode::<TestJwtClaims>(
            "not.a.jwt",
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_csrf_token_format_parsing() {
        // Test that token parsing handles dots correctly using rfind
        let token = "uuid-123.1700000000.abcdef1234567890";
        let last_dot = token.rfind('.').unwrap();
        let provided_sig = &token[last_dot + 1..];
        let payload = &token[..last_dot];

        assert_eq!(provided_sig, "abcdef1234567890");
        assert_eq!(payload, "uuid-123.1700000000");

        let payload_dot = payload.rfind('.').unwrap();
        let timestamp: u64 = payload[payload_dot + 1..].parse().unwrap();
        assert_eq!(timestamp, 1700000000);
    }

    #[test]
    fn test_csrf_token_rfind_parsing() {
        let token = "uuid-123.1700000000.abcdef1234567890";
        let last_dot = token.rfind('.').unwrap();
        let sig = &token[last_dot + 1..];
        let payload = &token[..last_dot];

        let payload_dot = payload.rfind('.').unwrap();
        let timestamp_str = &payload[payload_dot + 1..];
        let timestamp: u64 = timestamp_str.parse().unwrap();

        assert_eq!(sig.len(), 16);
        assert_eq!(timestamp, 1700000000);
    }

    #[test]
    fn test_basic_auth_special_chars_password() {
        let username = "admin";
        let password = "p@ssw0rd!#$%^&*()_+-=[]{}|;':\",./<>?";

        let credentials = format!("{}:{}", username, password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        let decoded_str = String::from_utf8(decoded_bytes).unwrap();

        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], username);
        assert_eq!(parts[1], password);
    }

    #[test]
    fn test_basic_auth_unicode_credentials() {
        let username = "admin";
        let password = "密码123";

        let credentials = format!("{}:{}", username, password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        let decoded_str = String::from_utf8(decoded_bytes).unwrap();

        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], username);
        assert_eq!(parts[1], password);
    }

    #[test]
    fn test_basic_auth_empty_password() {
        let credentials = "admin:";
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        let decoded_str = String::from_utf8(decoded_bytes).unwrap();

        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "admin");
        assert_eq!(parts[1], "");
    }
}
