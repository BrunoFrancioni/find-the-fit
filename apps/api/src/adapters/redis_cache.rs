use crate::ports::CachePort;
use anyhow::Result;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

#[derive(Clone)]
pub struct RedisCacheAdapter {
    connection: ConnectionManager,
}

impl RedisCacheAdapter {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;
        Ok(Self { connection })
    }
}

#[async_trait::async_trait]
impl CachePort for RedisCacheAdapter {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.connection.clone();
        let value: Option<String> = conn.get(key).await?;
        Ok(value)
    }

    async fn set_ex(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.connection.clone();
        let _: () = conn.set_ex(key, value, ttl_seconds).await?;
        Ok(())
    }
}
