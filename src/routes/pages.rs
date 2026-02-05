use actix_web::{web, HttpResponse};
use super::tables::AppState;

pub async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(r#"<script>
            if (localStorage.getItem('jwt_token')) {
                window.location.href = '/dashboard';
            } else {
                window.location.href = '/login';
            }
        </script>"#)
}

pub async fn login_page(state: web::Data<AppState>) -> HttpResponse {
    let ctx = tera::Context::new();
    match state.templates.render("login.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn dashboard_page(state: web::Data<AppState>) -> HttpResponse {
    let ctx = tera::Context::new();
    match state.templates.render("dashboard.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn table_detail_page(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let table_name = path.into_inner();
    let mut ctx = tera::Context::new();
    ctx.insert("table_name", &table_name);
    match state.templates.render("table_detail.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}
