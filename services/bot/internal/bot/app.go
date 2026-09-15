package bot

import (
	"log"
	"time"

	"github.com/PaulSonOfLars/gotgbot/v2"
	"github.com/PaulSonOfLars/gotgbot/v2/ext"
)

func Run() {
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
