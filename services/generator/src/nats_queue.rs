use async_nats::jetstream::context::CreateStreamError;
use async_nats::jetstream::{self, consumer::PullConsumer, consumer::pull::Config as PullConsumerConfig, stream::Config as StreamConfig};
use async_nats::ConnectError;
use async_nats::jetstream::stream::Stream;
use futures_util::StreamExt;

pub struct NatsQueue {
    js: jetstream::Context,
}

impl NatsQueue {
    pub async fn connect(url: &str) -> Result<Self, ConnectError> {
        let client = async_nats::connect(url).await?;
        let js = jetstream::new(client);
        Ok(Self {
            js,
        })
    }
    async fn ensure_stream(&mut self, queue: &str) -> Result<Stream, CreateStreamError> {
        let stream = self.js.get_or_create_stream(StreamConfig {
            name: queue.to_string(),
            subjects: vec![queue.to_string()],
            ..Default::default()
        }).await?;
        Ok(stream)
    }

    pub async fn dequeue(&mut self, queue: &str) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let stream = self.ensure_stream(queue).await?;

        let consumer: PullConsumer = stream.get_or_create_consumer(
            format!("generator-consumer-{}", queue).as_str(),
            PullConsumerConfig {
                durable_name: Some(format!("generator-consumer-{}", queue)),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            }).await?;

        let mut messages = consumer.fetch().max_messages(1).messages().await?;

        if let Some(msg_res) = messages.next().await {
            let msg = msg_res?;
            msg.ack().await?;
            Ok(Some(msg.payload.to_vec()))
        } else {
            Ok(None)
        }
    }

    pub async fn enqueue(&mut self, queue: &str, payload: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_stream(queue).await?;
        self.js.publish(queue.to_string(), payload.into()).await?;
        Ok(())
    }
}
