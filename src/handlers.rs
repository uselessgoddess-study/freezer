use crate::{auth::Role, errors::not_found, Result};
use actix_web::{delete, get, http::header::ContentType, post, web, HttpResponse, Responder};
use async_std::sync::Mutex;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};


use std::collections::HashMap;

mod grants {
    pub use actix_web_grants::proc_macro::has_any_role as any;
}

use crate::{
    model::Freezer,
    service::{FreezersStore, ImageStore, ProductsStore},
};
use tap::{Pipe, Tap};

#[derive(Serialize, Deserialize)]
pub struct LimitQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

#[get("/api/products")]
pub async fn get_products(
    _query: web::Query<LimitQuery>,
    store: web::Data<ProductsStore>,
) -> Result<impl Responder> {
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
#[grants::any(type = "Role", "Role::Moder", "Role::Admin")]
pub async fn put_in(
    freezer: web::Path<String>,
    web::Json(prods): web::Json<HashMap<String, usize>>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();
    let freezer = store.freezer(&id).await?.tap_mut(|freezer| {
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
#[grants::any(type = "Role", "Role::Moder", "Role::Admin")]
pub async fn put_out(
    freezer: web::Path<String>,
    web::Json(prods): web::Json<HashMap<String, usize>>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();
    let freezer = store.freezer(&id).await?.tap_mut(|freezer| {
        for (product, count) in prods {
            freezer.products.entry(product).and_modify(|entry| {
                *entry -= count;
            });
        }
    });

    Ok(store.update(&id, freezer).await?.pipe(web::Json))
}

#[post("/api/freezers/{freezer}/remove")]
#[grants::any(type = "Role", "Role::Moder", "Role::Admin")]
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
pub async fn get_freezers(
    query: web::Query<LimitQuery>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    store
        .freezers_list_by(query.limit, query.offset)
        .await?
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .pipe(web::Json)
        .pipe(Ok)
}

#[get("/api/freezers/{freezer}")]
pub async fn one_freezer(
    freezer: web::Path<String>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();
    store.freezer(&id).await.map(web::Json)
}

#[post("/api/freezers/update")]
#[grants::any(type = "Role", "Role::Moder", "Role::Admin")]
pub async fn update_freezer(
    freezer: web::Json<Freezer>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let freezer = freezer.into_inner();
    store
        .update(&freezer.name.clone(), freezer)
        .await
        .map(web::Json)
}

#[delete("/api/freezers/{freezer}")]
#[grants::any(type = "Role", "Role::Admin")]
pub async fn remove_freezer(
    freezer: web::Path<String>,
    store: web::Data<FreezersStore>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();
    store.remove(&id).await.map(web::Json)
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
        .unwrap_or_else(|| DEV_LOGO.into());

    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(raw))
}

#[post("/api/freezers/{freezer}/image")]
#[grants::any(type = "Role", "Role::Admin")]
pub async fn post_image(
    img: web::Bytes,
    freezer: web::Path<String>,
    store: web::Data<Mutex<ImageStore>>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();

    store.lock().await.load(id.as_bytes(), &img).await?;

    Ok(HttpResponse::Ok())
}

#[delete("/api/freezers/{freezer}/image")]
#[grants::any(type = "Role", "Role::Moder", "Role::Admin")]
pub async fn remove_image(
    freezer: web::Path<String>,
    store: web::Data<Mutex<ImageStore>>,
) -> Result<impl Responder> {
    let id = freezer.into_inner();

    store.lock().await.remove(id.as_bytes()).await?;

    Ok(HttpResponse::Ok())
}

#[post("/api/stored_procedure")]
#[grants::any(type = "Role", "Role::Admin")]
pub async fn stored_procedure(
    store: web::Data<FreezersStore>,
    products: web::Data<ProductsStore>,
) -> Result<impl Responder> {
    let mut pack = Vec::new();
    let mut freezers = store.freezers().await?;
    while let Some(mut freezer) = TryStreamExt::try_next(&mut freezers).await? {
        for (name, amount) in &mut freezer.products {
            *amount = products.product(&name).await?.default;
        }
        pack.push(freezer);
    }

    for freezer in pack {
        let _ = store.update(&freezer.name.clone(), freezer).await?;
    }

    Ok(HttpResponse::Ok())
}
