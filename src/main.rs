use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::time::Instant;
use tera::{Context, Tera};

fn tera_engine() -> Tera {
    Tera::new("templates/**/*").expect("init tera")
}

async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("title", "Rust Web AI");
    ctx.insert("active", "home");
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
    let tera = tera_engine();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .service(Files::new("/assets", "static/assets").prefer_utf8(true))
            .route("/", web::get().to(index))
            .route("/api/bench", web::get().to(bench))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
