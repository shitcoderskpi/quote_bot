package main

import (
	"log"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	"github.com/PaulSonOfLars/gotgbot/v2"
	"github.com/PaulSonOfLars/gotgbot/v2/ext"
	"github.com/PaulSonOfLars/gotgbot/v2/ext/handlers"
)

func setupLogging(logPath string) *os.File {
	log.SetFlags(log.Ldate | log.Ltime | log.Lshortfile)
	if logPath == "" {
		return nil
	}

	if err := os.MkdirAll(logPath, 0755); err != nil {
		log.Fatalf("error creating log directory: %v", err)
	}

	f, err := os.OpenFile(filepath.Join(logPath, "bot-service.log"), os.O_RDWR|os.O_CREATE|os.O_APPEND, 0666)
	if err != nil {
		log.Fatalf("error opening log file: %v", err)
	}

	log.SetOutput(f)

	return f
}

func newDispatcher(queue Queue) *ext.Dispatcher {
	dispatcher := ext.NewDispatcher(&ext.DispatcherOpts{
		Error: func(b *gotgbot.Bot, ctx *ext.Context, err error) ext.DispatcherAction {
			log.Printf("an error occurred while handling update: %v", err)
			return ext.DispatcherActionNoop
		},
		MaxRoutines: ext.DefaultMaxRoutines,
	})

	quote := quoteHandler(queue)
	dispatcher.AddHandler(handlers.NewCommand("start", startHandler))
	dispatcher.AddHandler(handlers.NewCommand("q", quote))
	dispatcher.AddHandler(handlers.NewCommand("quote", quote))

	return dispatcher
}

func waitForShutdown(updater *ext.Updater) {
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	<-c

	log.Println("Stopping bot...")
	if err := updater.Stop(); err != nil {
		log.Printf("failed to stop updater: %v", err)
	}
	log.Println("Bot stopped")
}

func main() {
	config := LoadConfig()

	logFile := setupLogging(config.LogPath)
	if logFile != nil {
		defer func() {
			if err := logFile.Close(); err != nil {
				log.Printf("Failed to close log file: %v", err)
			}
		}()
	}

	var queue Queue = NewRedisQueue(config.RedisHost)
	defer func() {
		if err := queue.Close(); err != nil {
			log.Printf("Failed to close Redis queue: %v", err)
		}
	}()

	b, err := gotgbot.NewBot(config.BotToken, nil)
	if err != nil {
		log.Fatalf("failed to create bot: %v", err)
	}

	updater := ext.NewUpdater(newDispatcher(queue), nil)
	if err := updater.StartPolling(b, &ext.PollingOpts{
		DropPendingUpdates: true,
		GetUpdatesOpts: &gotgbot.GetUpdatesOpts{
			Timeout: 9,
			RequestOpts: &gotgbot.RequestOpts{
				Timeout: time.Second * 10,
			},
		},
	}); err != nil {
		log.Fatalf("failed to start polling: %v", err)
	}

	log.Printf("%s has been started...\n", b.User.Username)
	waitForShutdown(updater)
}
