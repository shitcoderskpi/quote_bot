package bot

import (
	"context"
	"strings"
	"time"

	"github.com/redis/go-redis/v9"
)

type Queue interface {
	Enqueue(ctx context.Context, name string, data []byte) error
	Dequeue(ctx context.Context, name string, timeout time.Duration) ([]byte, error)
	Close() error
}

type RedisQueue struct {
	client *redis.Client
}

func NewRedisQueue(host string) *RedisQueue {
	if !strings.Contains(host, ":") {
		host = host + ":6379"
	}

	rdb := redis.NewClient(&redis.Options{
		Addr:     host,
		Password: "",
		DB:       0,
	})

	return &RedisQueue{
		client: rdb,
	}
}

func (r *RedisQueue) Enqueue(ctx context.Context, name string, data []byte) error {
	return r.client.LPush(ctx, name, data).Err()
}

func (r *RedisQueue) Dequeue(ctx context.Context, name string, timeout time.Duration) ([]byte, error) {
	res, err := r.client.BRPop(ctx, timeout, name).Result()
	if err != nil {
		return nil, err
	}
	if len(res) == 2 {
		return []byte(res[1]), nil
	}
	return nil, nil
}

func (r *RedisQueue) Close() error {
	return r.client.Close()
}
