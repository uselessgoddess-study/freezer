use bytes::Bytes;
use format_bytes::format_bytes;
use redis::{aio::Connection, AsyncCommands, Commands, RedisResult};

pub struct ImageStore(Connection);

impl ImageStore {
    pub fn new(client: Connection) -> Self {
        Self(client)
    }

    fn image_path(key: &[u8]) -> Vec<u8> {
        format_bytes!(b"freezers:{}:image", key)
    }

    pub async fn image(&mut self, uuid: &[u8]) -> RedisResult<Bytes> {
        self.0.get(uuid).await
    }

    pub async fn load_image(&mut self, uuid: &[u8], bytes: &[u8]) -> RedisResult<Bytes> {
        self.0.set(uuid, bytes).await
    }
}
