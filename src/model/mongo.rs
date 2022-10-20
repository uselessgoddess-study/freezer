struct MongoClient(mongodb::Client);

impl MongoClient {
    const DB: &'static str = "products_info";
    const FREEZERS: &'static str = "freezers";
    const PRODUCTS: &'static str = "products";
    const MODELS: &'static str = "models";

    pub async fn freezers(&self) {}
}
