package main

import (
	"log"
	"os"
)

type Config struct {
	BotToken  string
	RedisHost string
	LogPath   string
}

func LoadConfig() Config {

	botToken := os.Getenv("BOT_TOKEN")
	if botToken == "" {
		log.Fatal("BOT_TOKEN is not set in environment")
	}

	redisHost := os.Getenv("REDIS_HOST")
	if redisHost == "" {
		redisHost = "localhost"
	}

	logPath := os.Getenv("LOG_PATH")

	return Config{
		BotToken:  botToken,
		RedisHost: redisHost,
		LogPath:   logPath,
	}
}
