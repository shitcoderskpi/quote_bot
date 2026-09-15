package bot

import (
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/PaulSonOfLars/gotgbot/v2/ext"
)

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
