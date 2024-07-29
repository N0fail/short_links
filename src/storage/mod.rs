use ::redis::RedisError;
use async_trait::async_trait;
pub mod redis;

#[async_trait]
pub trait ShortLinkStorage: Send + Sync {
    async fn save(&self, key: &str, value: &str) -> Result<(), RedisError>;
    async fn load(&self, key: &str) -> Result<Option<String>, RedisError>;
    async fn generate_key(&self) -> Result<String, RedisError>;
}
