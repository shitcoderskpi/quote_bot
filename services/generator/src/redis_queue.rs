use redis::Commands;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError};

use crate::config::Config;

pub struct RedisQueue {
    client: Client,
    conn: MultiplexedConnection,
}

impl RedisQueue {
    pub async fn connect(cfg: &Config) -> Result<Self, RedisError> {
        let url = format!("redis://{}:{}/", cfg.redis_host, cfg.redis_port);
        let client = Client::open(url.as_str())?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { client, conn })
    }

    pub async fn dequeue(&mut self, queue: String, timeout: f64) -> Result<Option<Vec<u8>>, RedisError> {
        let client = self.client.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = client.get_connection()?;
            let result: Option<(String, Vec<u8>)> = conn.brpop(&queue, timeout)?;
            Ok(result.map(|(_key, payload)| payload))
        }).await.unwrap()
    }

    pub async fn enqueue(&mut self, queue: &str, payload: Vec<u8>) -> Result<(), RedisError> {
        let _: () = self.conn.lpush(queue, payload).await?;
        Ok(())
    }
}
