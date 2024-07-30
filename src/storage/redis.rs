use crate::storage::ShortLinkStorage;
use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, AsyncCommands, RedisError};

#[async_trait]
impl ShortLinkStorage for MultiplexedConnection {
    async fn save(&self, key: &str, value: &str) -> Result<(), RedisError> {
        self.clone().set(key, value).await
    }
    async fn load(&self, key: &str) -> Result<Option<String>, RedisError> {
        self.clone().get(key).await
    }
    async fn get_seed(&self) -> Result<u64, RedisError> {
        let mut con = self.clone();
        con.incr::<_, u64, u64>("url_id", 1u64).await
    }
}
