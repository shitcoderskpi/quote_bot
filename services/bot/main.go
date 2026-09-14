package main

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/PaulSonOfLars/gotgbot/v2"
	"github.com/PaulSonOfLars/gotgbot/v2/ext"
	"github.com/PaulSonOfLars/gotgbot/v2/ext/handlers"
)

func main() {
	config := LoadConfig()

	if config.LogPath != "" {
		if err := os.MkdirAll(config.LogPath, 0755); err != nil {
			log.Fatalf("error creating log directory: %v", err)
		}
		f, err := os.OpenFile(config.LogPath+"bot-service.log", os.O_RDWR|os.O_CREATE|os.O_APPEND, 0666)
		if err != nil {
			log.Fatalf("error opening log file: %v", err)
		}
		defer func(f *os.File) {
			err := f.Close()
			if err != nil {
				log.Fatalf("Failed to close log file: %v", err)
			}
		}(f)
		log.SetOutput(f)
	}
	log.SetFlags(log.Ldate | log.Ltime | log.Lshortfile)

	redisQueue := NewRedisQueue(config.RedisHost)
	defer func(redisQueue *RedisQueue) {
		err := redisQueue.Close()
		if err != nil {
			log.Printf("Failed to close Redis queue: %v", err)
		}
	}(redisQueue)

	ctx := context.Background()
	_ = redisQueue.Delete(ctx, "generate:jobs")
	_ = redisQueue.Delete(ctx, "generate:results")

	b, err := gotgbot.NewBot(config.BotToken, nil)
	if err != nil {
		log.Fatalf("failed to create bot: %v", err)
	}

	dispatcher := ext.NewDispatcher(&ext.DispatcherOpts{
		Error: func(b *gotgbot.Bot, ctx *ext.Context, err error) ext.DispatcherAction {
			log.Printf("an error occurred while handling update: %v", err)
			return ext.DispatcherActionNoop
		},
		MaxRoutines: ext.DefaultMaxRoutines,
	})

	updater := ext.NewUpdater(dispatcher, nil)

	dispatcher.AddHandler(handlers.NewCommand("start", startHandler))
	dispatcher.AddHandler(handlers.NewCommand("q", quoteHandler(redisQueue)))
	dispatcher.AddHandler(handlers.NewCommand("quote", quoteHandler(redisQueue)))

	err = updater.StartPolling(b, &ext.PollingOpts{
		DropPendingUpdates: true,
		GetUpdatesOpts: &gotgbot.GetUpdatesOpts{
			Timeout: 9,
			RequestOpts: &gotgbot.RequestOpts{
				Timeout: time.Second * 10,
			},
		},
	})
	if err != nil {
		log.Fatalf("failed to start polling: %v", err)
	}
	log.Printf("%s has been started...\n", b.User.Username)

	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	<-c

	log.Println("Stopping bot...")
	err = updater.Stop()
	if err != nil {
		log.Printf("failed to stop updater: %v", err)
	}
	log.Println("Bot stopped")
}
