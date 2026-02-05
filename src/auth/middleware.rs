use crate::auth::jwt::{validate_token, Claims};
use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::ServiceRequest, Error};

pub fn extract_token(req: &ServiceRequest) -> Result<String, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| ErrorUnauthorized("Missing Authorization header"))?
        .to_str()
        .map_err(|_| ErrorUnauthorized("Invalid Authorization header"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ErrorUnauthorized(
            "Authorization header must start with Bearer",
        ));
    }

    Ok(auth_header[7..].to_string())
}

pub fn authenticate(token: &str, public_key_pem: &[u8], issuer: &str) -> Result<Claims, Error> {
    validate_token(token, public_key_pem, issuer)
        .map_err(|e| ErrorUnauthorized(format!("Invalid token: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::create_token;
    use crate::auth::keys::generate_keys;
    use actix_web::test::TestRequest;
    use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
    use tempfile::TempDir;

    #[test]
    fn test_extract_token_valid_header() {
        let req = TestRequest::default()
            .insert_header(("Authorization", "Bearer abc123"))
            .to_srv_request();
        let token = extract_token(&req).unwrap();
        assert_eq!(token, "abc123");
    }

    #[test]
    fn test_extract_token_missing_header() {
        let req = TestRequest::default().to_srv_request();
        let result = extract_token(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_authenticate_valid_token() {
        let tmp = TempDir::new().unwrap();
        let kp = generate_keys(tmp.path()).unwrap();
        let priv_pem = kp
            .private_key
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .unwrap();
        let pub_pem = kp
            .public_key
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .unwrap();
        let token = create_token("user1", "sqlite-editor", 3600, priv_pem.as_bytes()).unwrap();
        let claims = authenticate(&token, pub_pem.as_bytes(), "sqlite-editor").unwrap();
        assert_eq!(claims.sub, "user1");
    }
}
