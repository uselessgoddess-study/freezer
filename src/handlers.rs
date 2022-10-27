use crate::{
    errors::{not_found, Error},
    FreezersStore, ImageStore, ProductsStore, Result,
};
use actix_web::{delete, get, http::header::ContentType, post, web, HttpResponse, Responder};
use async_std::sync::Mutex;
use futures::StreamExt;
use mongodb::bson::oid::ObjectId;
use std::{borrow::Borrow, collections::HashMap, str::FromStr};

use crate::model::Product;
use tap::{Pipe, Tap};

#[get("/api/products")]
pub async fn products(store: web::Data<ProductsStore>) -> Result<impl Responder> {
    Ok(store
        .products()
        .await?
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .pipe(web::Json))
}

#[get("/api/products/{product_id}")]
pub async fn one_product(
    product: web::Path<String>,
    store: web::Data<ProductsStore>,
) -> Result<impl Responder> {
    let product = product.into_inner();
    store.product(&product).await.map(web::Json)
}

#[post("/api/freezers/{freezer}/put-in")]
pub async fn put_in(
    freezer: web::Path<String>,
    web::Json(prods): web::Json<HashMap<String, usize>>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();
    let mut freezer = store.freezer(&id).await?.tap_mut(|freezer| {
        for (product, count) in prods {
            freezer
                .products
                .entry(product)
                .and_modify(|entry| {
                    *entry += count;
                })
                .or_insert(count);
        }
    });

    Ok(store.update(&id, freezer).await?.pipe(web::Json))
}

#[post("/api/freezers/{freezer}/put-out")]
pub async fn put_out(
    freezer: web::Path<String>,
    web::Json(prods): web::Json<HashMap<String, usize>>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();
    let mut freezer = store.freezer(&id).await?.tap_mut(|freezer| {
        for (product, count) in prods {
            freezer.products.entry(product).and_modify(|entry| {
                *entry -= count;
            });
        }
    });

    Ok(store.update(&id, freezer).await?.pipe(web::Json))
}

struct RemoveReq {
    pattern: Option<String>,
}

#[post("/api/freezers/{freezer}/remove")]
pub async fn remove_product(
    freezer: web::Path<String>,
    product: String,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();
    let mut freezer = store.freezer(&id).await?;

    if freezer.products.remove(&product).is_none() {
        return not_found!("{product}");
    }

    Ok(store.update(&id, freezer).await?.pipe(web::Json))
}

#[get("/api/freezers")]
pub async fn freezers(store: web::Data<FreezersStore>) -> Result<impl Responder> {
    Ok(store
        .freezers()
        .await?
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .pipe(web::Json))
}

#[get("/api/freezers/{freezer}")]
pub async fn one_freezer(
    freezer: web::Path<String>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();
    store.freezer(&id).await.map(web::Json)
}

#[post("/api/load_images/{key}/{value}")]
pub async fn load_image(
    path: web::Path<(String, String)>,
    redis: web::Data<Mutex<ImageStore>>,
) -> impl Responder {
    let (user_id, friend) = path.into_inner();
    redis
        .lock()
        .await
        .load(user_id.as_bytes(), friend.as_bytes())
        .await
        .unwrap();
    HttpResponse::Ok()
}

#[get("/api/freezers/{freezer}/image")]
pub async fn image(
    freezer: web::Path<String>,
    store: web::Data<Mutex<ImageStore>>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();

    const DEV_LOGO: &[u8] = include_bytes!("../embedded/logo.jpg");

    let raw = store
        .lock()
        .await
        .image(id.as_bytes())
        .await?
        .unwrap_or(DEV_LOGO.into());

    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(raw))
}

#[post("/api/freezers/{freezer}/image")]
pub async fn post_image(
    img: web::Bytes,
    freezer: web::Path<String>,
    store: web::Data<Mutex<ImageStore>>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();

    let _ = store.lock().await.load(id.as_bytes(), &img).await?;

    Ok(HttpResponse::Ok())
}

#[delete("/api/freezers/{freezer}/image")]
pub async fn remove_image(
    freezer: web::Path<String>,
    store: web::Data<Mutex<ImageStore>>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();

    let _ = store.lock().await.remove(id.as_bytes()).await?;

    Ok(HttpResponse::Ok())
}

#[post("/api/post")]
pub async fn post(body: web::Bytes) -> impl Responder {
    std::fs::write("file.png", body).unwrap();
    HttpResponse::Ok()
}
