package bot

import (
	"log"
	"os"
	"path/filepath"
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
