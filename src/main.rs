use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::time::Instant;
use tera::{Context, Tera};

mod models;
mod database;
mod auth;
mod handlers;

use database::Database;
use auth::{logout, OptionalAuthUser};

fn tera_engine() -> Tera {
    let mut tera = Tera::new("templates/**/*").expect("init tera");
    
    // Enable autoescape for security
    tera.autoescape_on(vec!["html", "tera"]);
    
    tera
}

async fn index(tmpl: web::Data<Tera>, user: OptionalAuthUser) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("title", "Rust Web AI");
    ctx.insert("active", "home");
    ctx.insert("brand_name", "Rust Web AI");
    ctx.insert("meta_description", "High-performance web applications built with Rust, Actix-Web, and Smarty theme");
    
    // Navigation items
    let nav_items = vec![
        serde_json::json!({
            "key": "home",
            "label": "Home",
            "url": "/",
            "icon": "bi bi-house-fill"
        }),
        serde_json::json!({
            "key": "features",
            "label": "Features", 
            "url": "/features",
            "icon": "bi bi-stars"
        }),
        serde_json::json!({
            "key": "docs",
            "label": "Documentation",
            "url": "/docs", 
            "icon": "bi bi-book"
        }),
        serde_json::json!({
            "key": "about",
            "label": "About",
            "dropdown": vec![
                serde_json::json!({
                    "label": "Our Story",
                    "url": "/about",
                    "icon": "bi bi-info-circle"
                }),
                serde_json::json!({
                    "label": "Team",
                    "url": "/team",
                    "icon": "bi bi-people"
                }),
                serde_json::json!({"divider": true}),
                serde_json::json!({
                    "label": "Contact",
                    "url": "/contact",
                    "icon": "bi bi-envelope"
                })
            ]
        })
    ];
    ctx.insert("nav_items", &nav_items);
    ctx.insert("show_search", &true);
    
    // Footer configuration
    ctx.insert("footer_style", "full");
    ctx.insert("footer_description", "High-performance web applications built with Rust and the beautiful Smarty theme.");
    
    let footer_columns = vec![
        serde_json::json!({
            "title": "Product",
            "links": [
                {"label": "Features", "url": "/features"},
                {"label": "Pricing", "url": "/pricing"},
                {"label": "Documentation", "url": "/docs"},
                {"label": "API Reference", "url": "/api"}
            ]
        }),
        serde_json::json!({
            "title": "Company",
            "links": [
                {"label": "About Us", "url": "/about"},
                {"label": "Team", "url": "/team"},
                {"label": "Careers", "url": "/careers"},
                {"label": "Contact", "url": "/contact"}
            ]
        }),
        serde_json::json!({
            "title": "Resources",
            "links": [
                {"label": "Blog", "url": "/blog"},
                {"label": "Help Center", "url": "/help"},
                {"label": "Community", "url": "/community"},
                {"label": "Status", "url": "/status"}
            ]
        })
    ];
    ctx.insert("footer_columns", &footer_columns);
    
    let social_links = vec![
        serde_json::json!({"name": "GitHub", "url": "https://github.com", "icon": "bi bi-github"}),
        serde_json::json!({"name": "Twitter", "url": "https://twitter.com", "icon": "bi bi-twitter"}),
        serde_json::json!({"name": "LinkedIn", "url": "https://linkedin.com", "icon": "bi bi-linkedin"})
    ];
    ctx.insert("social_links", &social_links);
    
    // Pass user info if logged in
    if let Some(user_data) = user.0 {
        ctx.insert("user", &user_data);
    }
    
    let body = tmpl
        .render("index.html.tera", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

#[derive(Debug, Deserialize)]
struct BenchQuery {
    /// number of operations to run (default: 5_000_000)
    ops: Option<u64>,
}

#[derive(Debug, Serialize)]
struct BenchOut {
    ops: u64,
    seconds: f64,
    ops_per_sec: f64,
    mops_per_sec: f64,
    acc: u64, // to ensure the loop isn't optimized away
}

async fn bench(q: web::Query<BenchQuery>) -> impl Responder {
    let ops = q.ops.unwrap_or(5_000_000);
    let mut acc: u64 = 0;

    let start = Instant::now();
    for i in 0..ops {
        // mix a few cheap integer ops
        acc = acc
            .wrapping_mul(1664525)
            .wrapping_add(1013904223)
            ^ i;
        black_box(acc);
    }
    let dt = start.elapsed().as_secs_f64();

    let ops_per_sec = (ops as f64) / dt;
    let out = BenchOut {
        ops,
        seconds: dt,
        ops_per_sec,
        mops_per_sec: ops_per_sec / 1_000_000.0,
        acc,
    };

    HttpResponse::Ok().json(out)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let tera = tera_engine();
    
    // Initialize database
    let database = Database::new().await.expect("Failed to initialize database");
    
    // Generate a random key for session encryption
    // In production, use a persistent key from environment variable
    let secret_key = Key::generate();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(database.clone()))
            .wrap(Logger::default())
            .wrap(SessionMiddleware::new(
                actix_session::storage::CookieSessionStore::default(),
                secret_key.clone()
            ))
            .service(Files::new("/assets", "static/assets").prefer_utf8(true))
            // Public routes
            .route("/", web::get().to(index))
            .route("/api/bench", web::get().to(bench))
            
            // Authentication routes
            .route("/login", web::get().to(handlers::login_page))
            .route("/login", web::post().to(handlers::login_submit))
            .route("/register", web::get().to(handlers::register_page))
            .route("/register", web::post().to(handlers::register_submit))
            .route("/logout", web::post().to(logout))
            
            // Protected routes
            .route("/dashboard", web::get().to(handlers::dashboard_page))
            .route("/profile", web::get().to(handlers::profile_page))
    })
    .bind("127.0.0.1:8080")?;
    
    println!("🚀 Server running on http://127.0.0.1:8080");
    server.run().await
}
