use crate::utils::redis::Client;
use format_bytes::format_bytes;
use mongodb::bson::oid::ObjectId;
use redis::{AsyncCommands, Commands, RedisResult};

#[derive(Clone)]
pub struct ImageStore(Client);

impl ImageStore {
    pub fn new(client: Client) -> Self {
        Self(client)
    }

    fn image_path(key: &[u8]) -> Vec<u8> {
        format_bytes!(b"freezers:{}:image", key)
    }

    pub async fn image(&self, uuid: &[u8]) -> RedisResult<Vec<u8>> {
        self.0.clone().get(uuid).await
    }

    pub async fn load_image(&self, uuid: &[u8], bytes: &[u8]) -> RedisResult<Vec<u8>> {
        self.0.clone().set(uuid, bytes).await
    }
}
