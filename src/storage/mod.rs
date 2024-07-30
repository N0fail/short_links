use ::redis::RedisError;
use async_trait::async_trait;
pub mod redis;

#[async_trait]
pub trait ShortLinkStorage: Send + Sync {
    async fn save(&self, key: &str, value: &str) -> Result<(), RedisError>;
    async fn load(&self, key: &str) -> Result<Option<String>, RedisError>;
    async fn get_seed(&self) -> Result<u64, RedisError>;
}
