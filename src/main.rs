mod auth;
mod db;
mod routes;

use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use routes::tables::AppState;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

async fn health() -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let keys_dir = Path::new("keys");
    let kp = auth::keys::ensure_keys(keys_dir).expect("Failed to initialize RSA keys");

    let public_key_pem = {
        use rsa::pkcs1::EncodeRsaPublicKey;
        kp.public_key
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .expect("Failed to encode public key")
            .into_bytes()
    };

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/app.db".to_string());
    std::fs::create_dir_all(Path::new(&db_path).parent().unwrap_or(Path::new("."))).ok();
    let conn = Connection::open(&db_path).expect("Failed to open database");
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .expect("Failed to set WAL mode");

    let templates = tera::Tera::new("src/templates/**/*").expect("Failed to load templates");

    let public_key = web::Data::new(kp.public_key.clone());
    let app_state = web::Data::new(AppState {
        db: Mutex::new(conn),
        templates,
        public_key_pem,
        issuer: "sqlite-editor".to_string(),
    });

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    log::info!("Starting server on {}", bind);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(public_key.clone())
            .route("/health", web::get().to(health))
            .route(
                "/.well-known/jwks.json",
                web::get().to(routes::well_known::jwks_endpoint),
            )
            .route("/", web::get().to(routes::pages::index))
            .route("/login", web::get().to(routes::pages::login_page))
            .route("/dashboard", web::get().to(routes::pages::dashboard_page))
            .route(
                "/tables/{table_name}",
                web::get().to(routes::pages::table_detail_page),
            )
            .route("/api/tables", web::get().to(routes::tables::list_tables))
            .route("/api/tables", web::post().to(routes::tables::create_table))
            .route(
                "/api/tables/{table_name}",
                web::delete().to(routes::tables::delete_table),
            )
            .route(
                "/api/tables/{table_name}/columns",
                web::get().to(routes::tables::list_columns),
            )
            .route(
                "/api/tables/{table_name}/columns",
                web::post().to(routes::tables::add_column),
            )
            .route(
                "/api/tables/{table_name}/columns/{column_name}",
                web::delete().to(routes::tables::delete_column),
            )
            .service(Files::new("/static", "static").prefer_utf8(true))
    })
    .bind(&bind)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
