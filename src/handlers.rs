use crate::{
    errors::{not_found, Error},
    FreezersStore, ImageStore, ProductsStore, Result,
};
use actix_web::{get, post, web, HttpResponse, Responder};
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

#[post("/api/load_images/{key}/{value}")]
pub async fn load_image(
    redis: web::Data<Mutex<ImageStore>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (user_id, friend) = path.into_inner();
    redis
        .lock()
        .await
        .load_image(user_id.as_bytes(), friend.as_bytes())
        .await
        .unwrap();
    HttpResponse::Ok()
}

#[get("/api/images/{key}")]
pub async fn get_image(
    redis: web::Data<Mutex<ImageStore>>,
    path: web::Path<(String,)>,
) -> impl Responder {
    let (user_id,) = path.into_inner();
    let x = redis.lock().await.image(user_id.as_bytes()).await.unwrap();
    println!("{x:?}");
    HttpResponse::Ok()
}

#[post("/api/post")]
pub async fn post(body: web::Bytes) -> impl Responder {
    std::fs::write("file.png", body).unwrap();
    HttpResponse::Ok()
}
