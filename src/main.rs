#![feature(decl_macro)]
#![feature(poll_ready)]
#![feature(try_blocks)]
#![deny(clippy::all, clippy::perf)]

mod auth;
mod errors;
mod handlers;
mod model;
mod service;
mod utils;

pub(crate) use errors::Result;

use crate::service::{FreezersStore, ImageStore, ProductsStore};
use actix_web::{web, App, HttpServer};
use async_std::sync::Mutex;

use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web_grants::GrantsMiddleware;

use std::env;
use tap::Pipe;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

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

    let redis = redis::Client::open(redis)?;
    let images = ImageStore::new(redis.get_async_connection().await?)
        .pipe(Mutex::new)
        .pipe(web::Data::new);

    let mongo = mongodb::Client::with_uri_str(mongo).await?;

    let freezers =
        FreezersStore::new(mongo.database("admin").collection("freezers")).pipe(web::Data::new);

    let products =
        ProductsStore::new(mongo.database("admin").collection("products")).pipe(web::Data::new);

    info!("bind to: http://localhost:1228");

    HttpServer::new(move || {
        App::new()
            .wrap(GrantsMiddleware::with_extractor(auth::extractor))
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(auth::SECRET))
                    .cookie_name("auth".into())
                    // .cookie_domain(Some("localhost".into()))
                    .cookie_secure(false)
                    .build(),
            )
            .service(handlers::get_freezers)
            .service(handlers::one_freezer)
            .service(handlers::update_freezer)
            .service(handlers::remove_freezer)
            //
            .service(handlers::get_products)
            .service(handlers::one_product)
            .service(handlers::remove_product)
            .service(handlers::put_in)
            .service(handlers::put_out)
            //
            .service(handlers::image)
            .service(handlers::post_image)
            .service(handlers::remove_image)
            //
            .service(auth::me)
            .service(auth::login)
            .service(auth::logout)
            //
            .service(handlers::stored_procedure)
            //
            .app_data(web::PayloadConfig::new(16_777_216)) // 16 MB payload
            .app_data(images.clone())
            .app_data(freezers.clone())
            .app_data(products.clone())
    })
    .bind(("127.0.0.1", 1228))?
    .run()
    .await
    .map_err(Into::into)
}
