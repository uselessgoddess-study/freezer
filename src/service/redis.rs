use crate::{auth::Role, Result};
use bytes::Bytes;
use format_bytes::format_bytes;
use redis::{aio::Connection, AsyncCommands, Commands};

pub struct ImageStore(Connection);

impl ImageStore {
    pub fn new(client: Connection) -> Self {
        Self(client)
    }

    fn image_path(key: &[u8]) -> Vec<u8> {
        format_bytes!(b"freezers:{}:image", key)
    }

    pub async fn image(&mut self, uuid: &[u8]) -> Result<Option<Bytes>> {
        self.0.get(uuid).await.map_err(Into::into)
    }

    pub async fn load(&mut self, uuid: &[u8], bytes: &[u8]) -> Result<()> {
        self.0.set(uuid, bytes).await.map_err(Into::into)
    }

    pub async fn remove(&mut self, uuid: &[u8]) -> Result<()> {
        self.0.del(uuid).await.map_err(Into::into)
    }
}

pub struct RoleStore(Connection);

impl RoleStore {
    pub fn new(client: Connection) -> Self {
        Self(client)
    }

    pub async fn role(&mut self, login: &str) -> Result<Option<Role>> {
        self.0.get(login).await.map_err(Into::into)
    }

    pub async fn set_role(&mut self, login: &str, role: Role) -> Result<Option<Role>> {
        self.0.set(login, role).await.map_err(Into::into)
    }
}
