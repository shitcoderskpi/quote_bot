package bot

import (
	"fmt"
	"log"
	"net"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/PaulSonOfLars/gotgbot/v2"
	"github.com/PaulSonOfLars/gotgbot/v2/ext"
)

func isEndpointReachable(rawURL string, timeout time.Duration) bool {
	u, err := url.Parse(rawURL)
	if err != nil || u.Host == "" {
		return false
	}
	host := u.Host
	if !strings.Contains(host, ":") {
		if u.Scheme == "https" {
			host += ":443"
		} else {
			host += ":80"
		}
	}
	conn, err := net.DialTimeout("tcp", host, timeout)
	if err != nil {
		return false
	}
	_ = conn.Close()
	return true
}

func initBot(token string, configuredApiUrl string) (*gotgbot.Bot, error) {
	candidateUrl := strings.TrimSpace(configuredApiUrl)
	if candidateUrl == "" {
		candidateUrl = "http://localhost:8081"
	}

	lower := strings.ToLower(candidateUrl)
	shouldTryLocal := !strings.Contains(lower, "api.telegram.org")

	if shouldTryLocal {
		if isEndpointReachable(candidateUrl, 1*time.Second) {
			localOpts := &gotgbot.BotOpts{
				BotClient: &gotgbot.BaseBotClient{
					Client: http.Client{},
					DefaultRequestOpts: &gotgbot.RequestOpts{
						APIURL: candidateUrl,
					},
				},
				RequestOpts: &gotgbot.RequestOpts{
					Timeout: 5 * time.Second,
					APIURL:  candidateUrl,
				},
			}

			b, err := gotgbot.NewBot(token, localOpts)
			if err == nil {
				log.Printf("Connected to local Telegram Bot API at %s (@%s)", candidateUrl, b.User.Username)
				return b, nil
			}
			log.Printf("Failed to initialize bot with local API at %s (%v). Falling back to official remote API...", candidateUrl, err)
		} else {
			log.Printf("No local Telegram Bot API running at %s. Using official remote API...", candidateUrl)
		}
	} else {
		log.Printf("Using official remote Telegram Bot API.")
	}

	b, err := gotgbot.NewBot(token, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create bot using official remote API: %w", err)
	}

	log.Printf("Connected to official remote Telegram Bot API (@%s)", b.User.Username)
	return b, nil
}

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

	var queue, err = NewNatsQueue(config.NatsUrl)
	if err != nil {
		log.Fatalf("Failed to create nats queue: %v", err)
	}
	defer func() {
		queue.Close()
	}()

	b, err := initBot(config.BotToken, config.BotApiUrl)
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
