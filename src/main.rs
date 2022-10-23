#![feature(specialization)]
#![feature(decl_macro)]
#![deny(clippy::all, clippy::perf)]

mod errors;
mod handlers;
mod model;
mod service;
mod utils;

pub(crate) use errors::Result;

use crate::service::{FreezersStore, ImageStore, ProductsStore};
use actix_web::{web, App, HttpServer};
use async_std::sync::Mutex;
use futures::{StreamExt, TryStreamExt};

use mongodb::bson::oid::ObjectId;
use std::env;
use tap::Pipe;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

use crate::handlers::post;

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .pretty()
        .init();
    dotenv::from_filename(".env.local").ok();

    let mongo = env::var("MONGO_URI").expect("`MONGO_URI` env var should be specified");
    let redis = env::var("REDIS_URI").expect("`REDIS_URI` env var should be specified");

    info!(mongo, redis, "db envs:");

    // let mongodb_client = MongoDbClient::new(mongodb_uri).await;

    let redis = redis::Client::open(redis)?.get_async_connection().await?;
    let images = ImageStore::new(redis).pipe(Mutex::new).pipe(web::Data::new);

    let mongo = mongodb::Client::with_uri_str(mongo).await?;

    let freezers =
        FreezersStore::new(mongo.database("admin").collection("freezers")).pipe(web::Data::new);

    let products =
        ProductsStore::new(mongo.database("admin").collection("products")).pipe(web::Data::new);

    info!("bind to: http://localhost:8080");

    let x = json::to_value(ObjectId::new()).unwrap();
    println!("{x}");
    let y: ObjectId = json::from_value(x).unwrap();
    println!("{y}");

    let x = json::json!(y);

    HttpServer::new(move || {
        App::new()
            .service(post)
            .service(handlers::freezers)
            .service(handlers::products)
            .service(handlers::one_product)
            .service(handlers::put_in)
            .service(handlers::put_out)
            .service(handlers::remove_product)
            .app_data(web::PayloadConfig::new(16_777_216)) // 16 MB
            .app_data(images.clone())
            .app_data(freezers.clone())
            .app_data(products.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .map_err(Into::into)
}
