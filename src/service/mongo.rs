use crate::{model::Freezer, Result};
use futures::{Stream, StreamExt};
use mongodb::bson::{doc, Document};

use crate::{
    errors::{not_found, Error},
    model::Product,
};
use mongodb::Collection;

fn acquire_err<T, E: Into<Error>>(place: Result<T, E>) -> Result<T> {
    place.map_err(Into::into)
}

pub struct ProductsStore(Collection<Product>);

impl ProductsStore {
    pub fn new(collection: Collection<Product>) -> Self {
        Self(collection)
    }

    pub async fn products(&self) -> Result<impl Stream<Item = Result<Product>>> {
        Ok(self.0.find(None, None).await?.map(acquire_err))
    }

    pub async fn product_by_doc(&self, bson: Document) -> Result<Product> {
        self.0
            .find_one(bson.clone(), None)
            .await
            .map_err(Into::into)
            .transpose()
            .unwrap_or_else(|| not_found!("{bson}"))
    }

    pub async fn product(&self, name: &str) -> Result<Product> {
        self.product_by_doc(doc! { "_id": name }).await
    }
}

pub struct FreezersStore(Collection<Freezer>);

impl FreezersStore {
    pub fn new(collection: Collection<Freezer>) -> Self {
        Self(collection)
    }

    pub async fn freezers(&self) -> Result<impl Stream<Item = Result<Freezer>>> {
        Ok(self.0.find(None, None).await?.map(acquire_err))
    }

    pub async fn freezer_by_doc(&self, bson: Document) -> Result<Freezer> {
        self.0
            .find_one(bson.clone(), None)
            .await
            .map_err(Into::into)
            .transpose()
            .unwrap_or_else(|| not_found!("{bson}"))
    }

    pub async fn freezer(&self, name: &str) -> Result<Freezer> {
        self.freezer_by_doc(doc! { "_id": name }).await
    }

    pub async fn update_by_doc(&self, bson: Document, freezer: Freezer) -> Result<Freezer> {
        let _ = self
            .0
            .update_one(
                bson.clone(),
                doc! {
                    "$set": freezer.into_doc(),
                },
                None,
            )
            .await?;
        self.freezer_by_doc(bson.clone()).await
    }

    pub async fn update(&self, name: &str, freezer: Freezer) -> Result<Freezer> {
        self.update_by_doc(
            doc! {
                "_id": name,
            },
            freezer,
        )
        .await
    }
}
