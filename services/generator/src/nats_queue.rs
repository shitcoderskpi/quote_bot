use std::collections::HashMap;
use async_nats::jetstream::context::CreateStreamError;
use async_nats::jetstream::{self, consumer::PullConsumer, consumer::pull::Config as PullConsumerConfig, stream::Config as StreamConfig};
use async_nats::{ConnectError, Subject};
use async_nats::jetstream::stream::Stream;
use futures_util::StreamExt;
use prost::bytes::Bytes;

pub struct NatsQueue {
    js: jetstream::Context,
    subjects: HashMap<String, Subject>,
    consumers: HashMap<String, PullConsumer>,
}

impl NatsQueue {
    pub async fn connect(url: &str) -> Result<Self, ConnectError> {
        let client = async_nats::connect(url).await?;
        let js = jetstream::new(client);
        Ok(Self {
            js,
            subjects: HashMap::new(),
            consumers: HashMap::new(),
        })
    }

    async fn ensure_stream(&self, queue: &str) -> Result<Stream, CreateStreamError> {
        self.js
            .get_or_create_stream(StreamConfig {
                name: queue.to_string(),
                subjects: vec![queue.to_string()],
                ..Default::default()
            })
            .await
    }

    pub async fn dequeue(&mut self, queue: &str) -> Result<Option<Bytes>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.consumers.contains_key(queue) {
            let stream = self.ensure_stream(queue).await?;
            let name = format!("generator-consumer-{}", queue);
            let consumer: PullConsumer = stream
                .get_or_create_consumer(
                    &name,
                    PullConsumerConfig {
                        durable_name: Some(name.clone()),
                        ack_policy: jetstream::consumer::AckPolicy::Explicit,
                        ..Default::default()
                    },
                )
                .await?;
            self.consumers.insert(queue.to_owned(), consumer);
        }
        let consumer = &self.consumers[queue];

        let mut messages = consumer.fetch().max_messages(1).messages().await?;

        if let Some(msg_res) = messages.next().await {
            let msg = msg_res?;
            msg.ack().await?;
            Ok(Some(msg.payload.clone()))
        } else {
            Ok(None)
        }
    }

    pub async fn enqueue(&mut self, queue: &str, payload: Bytes) -> Result<(), Box<dyn std::error::Error>> {
        let subject = match self.subjects.get(queue) {
            Some(s) => s.clone(),
            None => {
                self.ensure_stream(queue).await?;
                let s = Subject::from(queue);
                self.subjects.insert(queue.to_owned(), s.clone());
                s
            }
        };

        self.js.publish(subject, payload).await?;
        Ok(())
    }
}