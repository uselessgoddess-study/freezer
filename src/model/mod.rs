use mongodb::bson::{oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub year: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Freezer {
    #[serde(rename = "_id")]
    pub name: String,

    pub model: Model,
    pub owner: Option<String>,
    pub products: HashMap<String, usize>,
}

impl Freezer {
    pub fn into_doc(self) -> Document {
        // SAFETY: `Freezer` is the correct bson
        unsafe { mongodb::bson::to_document(&self).unwrap_unchecked() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    #[serde(rename = "_id")]
    pub name: String,
    pub default: usize,
}
