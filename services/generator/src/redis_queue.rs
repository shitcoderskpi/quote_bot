use redis::{AsyncCommands, Client, RedisError};

use crate::config::Config;

pub struct RedisQueue {
    client: Client,
}

impl RedisQueue {
    pub async fn connect(cfg: &Config) -> Result<Self, RedisError> {
        let url = format!("redis://{}:{}/", cfg.redis_host, cfg.redis_port);
        let client = Client::open(url.as_str())?;
        Ok(Self { client })
    }

    pub async fn dequeue(&mut self, queue: &str, timeout: f64) -> Result<Vec<u8>, RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: (String, Vec<u8>) = conn.brpop(&queue, timeout).await?;
        Ok(result.1)
    }

    pub async fn enqueue(&mut self, queue: &str, payload: &Vec<u8>) -> Result<(), RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.lpush(queue, payload).await?;
        Ok(())
    }
}
