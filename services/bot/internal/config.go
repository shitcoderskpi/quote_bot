package bot

import (
	"log"
	"os"
)

type Config struct {
	BotToken  string
	RedisHost string
	LogPath   string
}

func envDefault(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}

func LoadConfig() Config {
	botToken := os.Getenv("BOT_TOKEN")
	if botToken == "" {
		log.Fatal("BOT_TOKEN is not set in environment")
	}

	return Config{
		BotToken:  botToken,
		RedisHost: envDefault("REDIS_HOST", "localhost"),
		LogPath:   os.Getenv("LOG_PATH"),
	}
}
