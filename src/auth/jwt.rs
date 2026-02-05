use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

pub fn create_token(
    subject: &str,
    issuer: &str,
    ttl_seconds: i64,
    private_key_pem: &[u8],
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: subject.to_string(),
        exp: (now as i64 + ttl_seconds) as usize,
        iat: now,
        iss: issuer.to_string(),
    };
    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(private_key_pem)?;
    encode(&header, &claims, &key)
}

pub fn validate_token(
    token: &str,
    public_key_pem: &[u8],
    issuer: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_rsa_pem(public_key_pem)?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer]);
    validation.leeway = 0;
    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::keys::generate_keys;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::pkcs1::EncodeRsaPublicKey;
    use tempfile::TempDir;

    fn test_keys() -> (Vec<u8>, Vec<u8>) {
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
        (priv_pem.as_bytes().to_vec(), pub_pem.as_bytes().to_vec())
    }

    #[test]
    fn test_create_and_validate_token() {
        let (priv_pem, pub_pem) = test_keys();
        let token = create_token("testuser", "sqlite-editor", 3600, &priv_pem).unwrap();
        let claims = validate_token(&token, &pub_pem, "sqlite-editor").unwrap();
        assert_eq!(claims.sub, "testuser");
        assert_eq!(claims.iss, "sqlite-editor");
    }

    #[test]
    fn test_expired_token_rejected() {
        let (priv_pem, pub_pem) = test_keys();
        let token = create_token("testuser", "sqlite-editor", -10, &priv_pem).unwrap();
        let result = validate_token(&token, &pub_pem, "sqlite-editor");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_issuer_rejected() {
        let (priv_pem, pub_pem) = test_keys();
        let token = create_token("testuser", "wrong-issuer", 3600, &priv_pem).unwrap();
        let result = validate_token(&token, &pub_pem, "sqlite-editor");
        assert!(result.is_err());
    }
}
