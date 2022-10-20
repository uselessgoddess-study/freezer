mod model;
mod service;
mod utils;

use crate::service::ImageStore;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use json::json;
use redis::{AsyncCommands, Commands};
use std::sync::Arc;
use std::{env, io};
use tap::Pipe;
use tracing::{info, trace};
use tracing_subscriber::filter::LevelFilter;

#[get("/api/products")]
async fn products() -> impl Responder {
    json!({
        "freezers": "/api/products/freezers",
        // ...
    })
    .pipe(web::Json)
}

#[post("/api/load_images/{key}/{value}")]
async fn foo(
    mut redis: web::Data<ImageStore>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (user_id, friend) = path.into_inner();
    redis
        .load_image(user_id.as_bytes(), friend.as_bytes())
        .await
        .unwrap();
    HttpResponse::Ok()
}

#[get("/api/images/{key}")]
async fn bar(
    mut redis: web::Data<ImageStore>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (user_id, friend) = path.into_inner();
    redis
        .load_image(user_id.as_bytes(), friend.as_bytes())
        .await
        .unwrap();
    HttpResponse::Ok()
}

#[post("/api/post")]
async fn post(body: web::Bytes) -> impl Responder {
    std::fs::write("file.png", body).unwrap();
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .with_max_level(LevelFilter::TRACE)
        // .pretty()
        .init();
    dotenv::from_filename(".env.local").ok();

    let mongo = env::var("MONGO_URI").expect("`MONGO_URI` env var should be specified");
    let redis = env::var("REDIS_URI").expect("`REDIS_URI` env var should be specified");

    info!(mongo, redis, "db envs:");

    // let mongodb_client = MongoDbClient::new(mongodb_uri).await;

    let redis = redis::Client::open(redis)?.get_async_connection().await?;
    let images = ImageStore::new(redis).pipe(web::Data::new);

    let client = mongodb::Client::with_uri_str(mongo).await?;

    info!("bind to: http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .service(products)
            .service(post)
            .service(foo)
            .service(bar)
            .app_data(web::PayloadConfig::new(16_777_216)) // 16 MB
            .app_data(images.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .map_err(Into::into)
}
