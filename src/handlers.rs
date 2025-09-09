use actix_web::{web, HttpResponse, Result};
use actix_session::Session;
use tera::{Context, Tera};
use serde::Deserialize;
use crate::models::{LoginForm, RegisterForm, SessionUser};
use crate::database::Database;
use crate::auth::{login_user, OptionalAuthUser, AuthUser};

#[derive(Deserialize)]
pub struct QueryMessage {
    msg: Option<String>,
}

pub async fn login_page(
    tmpl: web::Data<Tera>,
    query: web::Query<QueryMessage>,
    user: OptionalAuthUser,
) -> Result<HttpResponse> {
    // Redirect if already logged in
    if user.0.is_some() {
        return Ok(HttpResponse::Found()
            .insert_header(("location", "/dashboard"))
            .finish());
    }

    let mut ctx = Context::new();
    ctx.insert("brand_name", "Rust Web AI");
    
    if let Some(msg) = &query.msg {
        ctx.insert("msg", msg);
    }

    let body = tmpl
        .render("auth/login-simple.html.tera", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

pub async fn login_submit(
    tmpl: web::Data<Tera>,
    form: web::Form<LoginForm>,
    session: Session,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("brand_name", "Rust Web AI");
    ctx.insert("form_data", &*form);

    // Authenticate user
    match db.authenticate_user(&form.email, &form.password).await {
        Ok(Some(user)) => {
            let session_user = SessionUser::from(user);
            
            if let Err(e) = login_user(&session, session_user) {
                ctx.insert("error", &format!("Login failed: {}", e));
                let body = tmpl
                    .render("auth/login-simple.html.tera", &ctx)
                    .unwrap_or_else(|e| format!("Template error: {e}"));
                return Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(body));
            }

            // Redirect to dashboard
            Ok(HttpResponse::Found()
                .insert_header(("location", "/dashboard"))
                .finish())
        },
        Ok(None) => {
            ctx.insert("error", "Invalid email or password");
            let body = tmpl
                .render("auth/login.html.tera", &ctx)
                .unwrap_or_else(|e| format!("Template error: {e}"));
            Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body))
        },
        Err(e) => {
            ctx.insert("error", &format!("Login error: {}", e));
            let body = tmpl
                .render("auth/login.html.tera", &ctx)
                .unwrap_or_else(|e| format!("Template error: {e}"));
            Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body))
        }
    }
}

pub async fn register_page(
    tmpl: web::Data<Tera>,
    user: OptionalAuthUser,
) -> Result<HttpResponse> {
    // Redirect if already logged in
    if user.0.is_some() {
        return Ok(HttpResponse::Found()
            .insert_header(("location", "/dashboard"))
            .finish());
    }

    let mut ctx = Context::new();
    ctx.insert("brand_name", "Rust Web AI");

    let body = tmpl
        .render("auth/register-simple.html.tera", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

pub async fn register_submit(
    tmpl: web::Data<Tera>,
    form: web::Form<RegisterForm>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("brand_name", "Rust Web AI");
    ctx.insert("form_data", &*form);

    // Validate form
    if let Err(validation_errors) = form.validate() {
        ctx.insert("errors", &validation_errors);
        let body = tmpl
            .render("auth/register-simple.html.tera", &ctx)
            .unwrap_or_else(|e| format!("Template error: {e}"));
        return Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body));
    }

    // Check if email already exists
    match db.email_exists(&form.email).await {
        Ok(true) => {
            ctx.insert("errors", &vec!["Email address is already registered".to_string()]);
            let body = tmpl
                .render("auth/register-simple.html.tera", &ctx)
                .unwrap_or_else(|e| format!("Template error: {e}"));
            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body));
        },
        Err(e) => {
            ctx.insert("errors", &vec![format!("Database error: {}", e)]);
            let body = tmpl
                .render("auth/register-simple.html.tera", &ctx)
                .unwrap_or_else(|e| format!("Template error: {e}"));
            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body));
        },
        _ => {}
    }

    // Check if username already exists
    match db.username_exists(&form.username).await {
        Ok(true) => {
            ctx.insert("errors", &vec!["Username is already taken".to_string()]);
            let body = tmpl
                .render("auth/register-simple.html.tera", &ctx)
                .unwrap_or_else(|e| format!("Template error: {e}"));
            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body));
        },
        Err(e) => {
            ctx.insert("errors", &vec![format!("Database error: {}", e)]);
            let body = tmpl
                .render("auth/register-simple.html.tera", &ctx)
                .unwrap_or_else(|e| format!("Template error: {e}"));
            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body));
        },
        _ => {}
    }

    // Create user
    match db.create_user(form.into_inner()).await {
        Ok(_user) => {
            // Redirect to login with success message
            Ok(HttpResponse::Found()
                .insert_header(("location", "/login?msg=registration_success"))
                .finish())
        },
        Err(e) => {
            ctx.insert("errors", &vec![format!("Registration failed: {}", e)]);
            let body = tmpl
                .render("auth/register-simple.html.tera", &ctx)
                .unwrap_or_else(|e| format!("Template error: {e}"));
            Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body))
        }
    }
}

pub async fn dashboard_page(
    tmpl: web::Data<Tera>,
    user: AuthUser,
) -> Result<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("title", "Dashboard");
    ctx.insert("brand_name", "Rust Web AI");
    ctx.insert("user", &user.0);
    ctx.insert("active", "dashboard");

    // Add navigation items
    let nav_items = vec![
        serde_json::json!({
            "key": "home",
            "label": "Home",
            "url": "/",
            "icon": "bi bi-house-fill"
        }),
        serde_json::json!({
            "key": "dashboard", 
            "label": "Dashboard",
            "url": "/dashboard",
            "icon": "bi bi-speedometer2"
        }),
    ];
    ctx.insert("nav_items", &nav_items);

    let body = tmpl
        .render("dashboard-simple.html.tera", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

pub async fn profile_page(
    tmpl: web::Data<Tera>,
    user: AuthUser,
) -> Result<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("title", "Profile Settings");
    ctx.insert("brand_name", "Rust Web AI");
    ctx.insert("user", &user.0);
    ctx.insert("active", "profile");

    let body = tmpl
        .render("auth/profile.html.tera", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}