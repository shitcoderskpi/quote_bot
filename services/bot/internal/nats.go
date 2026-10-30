package bot

import (
	"context"
	"fmt"

	"github.com/nats-io/nats.go"
	"github.com/nats-io/nats.go/jetstream"
)

type Queue interface {
	Enqueue(ctx context.Context, name string, data []byte) error
	Dequeue(ctx context.Context, name string) ([]byte, error)
	Close()
}

type NatsQueue struct {
	js jetstream.JetStream
}

func NewNatsQueue(url string) (*NatsQueue, error) {
	nc, err := nats.Connect(url)
	if err != nil {
		return nil, err
	}

	js, err := jetstream.New(nc)
	if err != nil {
		return nil, err
	}

	return &NatsQueue{
		js,
	}, nil
}

func (q *NatsQueue) ensureStream(ctx context.Context, name string) (jetstream.Stream, error) {
	stream, err := q.js.CreateOrUpdateStream(ctx, jetstream.StreamConfig{
		Name:     name,
		Subjects: []string{name},
	})
	if err != nil {
		return nil, err
	}
	return stream, nil
}

func (q *NatsQueue) Dequeue(ctx context.Context, name string) ([]byte, error) {
	stream, err := q.ensureStream(ctx, name)
	if err != nil {
		return nil, err
	}

	consumer, err := stream.CreateOrUpdateConsumer(ctx, jetstream.ConsumerConfig{
		Durable:   fmt.Sprintf("generator-consumer-%s", name),
		AckPolicy: jetstream.AckExplicitPolicy,
	})
	if err != nil {
		return nil, err
	}

	msgs, err := consumer.Fetch(1)
	if err != nil {
		return nil, err
	}

	for msg := range msgs.Messages() {
		if err := msg.Ack(); err != nil {
			return nil, err
		}
		return msg.Data(), nil
	}

	if err := msgs.Error(); err != nil {
		return nil, err
	}

	return nil, nil
}

func (q *NatsQueue) Enqueue(ctx context.Context, name string, data []byte) error {
	if _, err := q.ensureStream(ctx, name); err != nil {
		return err
	}

	_, err := q.js.Publish(ctx, name, data)
	return err
}

func (q *NatsQueue) Close() {
	q.js.Conn().Close()
}
