use crate::{
    storage::ShortLinkStorage,
    utils::{SHORT_LINK_ALPHABET, SHORT_LINK_LEN},
};
use async_trait::async_trait;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;
use redis::{aio::MultiplexedConnection, AsyncCommands, RedisError};

#[async_trait]
impl ShortLinkStorage for MultiplexedConnection {
    async fn save(&self, key: &str, value: &str) -> Result<(), RedisError> {
        self.clone().set(key, value).await
    }
    async fn load(&self, key: &str) -> Result<Option<String>, RedisError> {
        self.clone().get(key).await
    }
    async fn generate_key(&self) -> Result<String, RedisError> {
        let mut con = self.clone();
        match con.incr::<_, u64, u64>("url_id", 1u64).await {
            Ok(id) => {
                let mut rng = ChaCha8Rng::seed_from_u64(id);
                let sample = SHORT_LINK_ALPHABET.as_bytes();
                let key = sample
                    .choose_multiple(&mut rng, SHORT_LINK_LEN)
                    .copied()
                    .collect();
                Ok(String::from_utf8(key).unwrap())
            }
            Err(e) => {
                //todo logging
                eprintln!("Unable to create id {e}");
                Err(e)
            }
        }
    }
}
