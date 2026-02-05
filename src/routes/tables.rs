use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;
use crate::db::operations;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub templates: Tera,
    pub public_key_pem: Vec<u8>,
    pub issuer: String,
}

fn auth_check(req: &HttpRequest, state: &web::Data<AppState>) -> Result<(), HttpResponse> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| HttpResponse::Unauthorized().body("Missing Authorization header"))?
        .to_str()
        .map_err(|_| HttpResponse::Unauthorized().body("Invalid header"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(HttpResponse::Unauthorized().body("Invalid Authorization scheme"));
    }
    let token = &auth_header[7..];
    crate::auth::jwt::validate_token(token, &state.public_key_pem, &state.issuer)
        .map_err(|e| HttpResponse::Unauthorized().body(format!("Invalid token: {}", e)))?;
    Ok(())
}

pub async fn list_tables(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    if let Err(resp) = auth_check(&req, &state) { return resp; }
    let conn = state.db.lock().unwrap();
    match operations::list_tables(&conn) {
        Ok(tables) => {
            let mut ctx = tera::Context::new();
            ctx.insert("tables", &tables);
            match state.templates.render("partials/table_list.html", &ctx) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn create_table(
    req: HttpRequest,
    state: web::Data<AppState>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    if let Err(resp) = auth_check(&req, &state) { return resp; }
    let table_name = match form.get("table_name") {
        Some(n) => n.clone(),
        None => return HttpResponse::BadRequest().body("Missing table_name"),
    };
    let conn = state.db.lock().unwrap();
    if let Err(e) = operations::create_table(&conn, &table_name) {
        return HttpResponse::BadRequest().body(e);
    }
    match operations::list_tables(&conn) {
        Ok(tables) => {
            let mut ctx = tera::Context::new();
            ctx.insert("tables", &tables);
            match state.templates.render("partials/table_list.html", &ctx) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn delete_table(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if let Err(resp) = auth_check(&req, &state) { return resp; }
    let table_name = path.into_inner();
    let conn = state.db.lock().unwrap();
    if let Err(e) = operations::drop_table(&conn, &table_name) {
        return HttpResponse::BadRequest().body(e);
    }
    match operations::list_tables(&conn) {
        Ok(tables) => {
            let mut ctx = tera::Context::new();
            ctx.insert("tables", &tables);
            match state.templates.render("partials/table_list.html", &ctx) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn list_columns(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if let Err(resp) = auth_check(&req, &state) { return resp; }
    let table_name = path.into_inner();
    let conn = state.db.lock().unwrap();
    match operations::list_columns(&conn, &table_name) {
        Ok(columns) => {
            let mut ctx = tera::Context::new();
            ctx.insert("columns", &columns);
            ctx.insert("table_name", &table_name);
            match state.templates.render("partials/column_list.html", &ctx) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
            }
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn add_column(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    if let Err(resp) = auth_check(&req, &state) { return resp; }
    let table_name = path.into_inner();
    let column_name = match form.get("column_name") {
        Some(n) => n.clone(),
        None => return HttpResponse::BadRequest().body("Missing column_name"),
    };
    let column_type = match form.get("column_type") {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing column_type"),
    };
    let conn = state.db.lock().unwrap();
    if let Err(e) = operations::add_column(&conn, &table_name, &column_name, &column_type) {
        return HttpResponse::BadRequest().body(e);
    }
    match operations::list_columns(&conn, &table_name) {
        Ok(columns) => {
            let mut ctx = tera::Context::new();
            ctx.insert("columns", &columns);
            ctx.insert("table_name", &table_name);
            match state.templates.render("partials/column_list.html", &ctx) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
            }
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn delete_column(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    if let Err(resp) = auth_check(&req, &state) { return resp; }
    let (table_name, column_name) = path.into_inner();
    let conn = state.db.lock().unwrap();
    if let Err(e) = operations::drop_column(&conn, &table_name, &column_name) {
        return HttpResponse::BadRequest().body(e);
    }
    match operations::list_columns(&conn, &table_name) {
        Ok(columns) => {
            let mut ctx = tera::Context::new();
            ctx.insert("columns", &columns);
            ctx.insert("table_name", &table_name);
            match state.templates.render("partials/column_list.html", &ctx) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
            }
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}
