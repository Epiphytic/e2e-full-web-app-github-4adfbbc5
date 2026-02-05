use actix_web::{web, HttpResponse};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rsa::RsaPublicKey;
use rsa::traits::PublicKeyParts;
use serde::Serialize;

#[derive(Serialize)]
struct Jwk {
    kty: String,
    #[serde(rename = "use")]
    use_: String,
    alg: String,
    n: String,
    e: String,
    kid: String,
}

#[derive(Serialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

pub async fn jwks_endpoint(public_key: web::Data<RsaPublicKey>) -> HttpResponse {
    let n = URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
    let e = URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

    let jwk_set = JwkSet {
        keys: vec![Jwk {
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            n,
            e,
            kid: "sqlite-editor-key-1".to_string(),
        }],
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .json(jwk_set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::auth::keys::generate_keys;
    use tempfile::TempDir;

    #[actix_web::test]
    async fn test_jwks_endpoint_returns_valid_jwk() {
        let tmp = TempDir::new().unwrap();
        let kp = generate_keys(tmp.path()).unwrap();
        let pub_key = web::Data::new(kp.public_key);

        let app = test::init_service(
            App::new()
                .app_data(pub_key)
                .route("/.well-known/jwks.json", web::get().to(jwks_endpoint))
        ).await;

        let req = test::TestRequest::get().uri("/.well-known/jwks.json").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["keys"].is_array());
        assert_eq!(body["keys"][0]["kty"], "RSA");
        assert_eq!(body["keys"][0]["alg"], "RS256");
        assert!(body["keys"][0]["n"].is_string());
        assert!(body["keys"][0]["e"].is_string());
    }
}
