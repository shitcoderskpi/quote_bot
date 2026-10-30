package bot

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"html"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"bot/pb"

	"github.com/PaulSonOfLars/gotgbot/v2"
	"github.com/PaulSonOfLars/gotgbot/v2/ext"
	"github.com/klauspost/compress/zstd"
	"google.golang.org/protobuf/proto"
)

var avatarClient = &http.Client{Timeout: 10 * time.Second}

type senderIndexer map[int64]uint32

const botAPIDataDir = "/data"

func readLocalFile(token, filePath string) ([]byte, bool) {
	p := filePath
	if !filepath.IsAbs(p) {
		p = filepath.Join(botAPIDataDir, token, filepath.Clean("/"+filePath))
	}
	data, err := os.ReadFile(p)
	if err != nil {
		log.Printf("avatar: read %q: %v", p, err)
		return nil, false
	}
	return data, true
}

func (s senderIndexer) of(userID int64) uint32 {
	if idx, ok := s[userID]; ok {
		return idx
	}
	idx := uint32(len(s))
	s[userID] = idx
	return idx
}

func startHandler(b *gotgbot.Bot, ctx *ext.Context) error {
	_, err := ctx.EffectiveMessage.Reply(b, fmt.Sprintf("Hello, <b>%s</b>!", html.EscapeString(ctx.EffectiveUser.FirstName)), &gotgbot.SendMessageOpts{ParseMode: "HTML"})
	return err
}

func optionalString(s string) *string {
	if s == "" {
		return nil
	}
	return new(s)
}

func optionalUint32(v *int) *uint32 {
	if v == nil {
		return nil
	}
	u := uint32(*v)
	return &u
}

func userAvatar(b *gotgbot.Bot, userID int64) []byte {
	photos, err := b.GetUserProfilePhotos(userID, &gotgbot.GetUserProfilePhotosOpts{Limit: 1})
	if err != nil {
		log.Printf("Failed to get profile photos for user %d: %v", userID, err)
		return nil
	}
	if photos.TotalCount == 0 || len(photos.Photos) == 0 || len(photos.Photos[0]) == 0 {
		log.Printf("User %d does not have any photos", userID)
		return nil
	}

	sizes := photos.Photos[0]
	file, err := b.GetFile(sizes[0].FileId, nil)
	if err != nil {
		log.Printf("Failed to get profile photo file for user %d: %v", userID, err)
		return nil
	}

	if data, ok := readLocalFile(b.Token, file.FilePath); ok {
		return data
	}
	log.Printf("avatar: local read failed for %q, trying HTTP", file.FilePath)

	resp, err := avatarClient.Get(file.URL(b, nil))
	if err != nil {
		log.Printf("Failed to download profile photo for user %d: %v", userID, err)
		return nil
	}
	defer func(Body io.ReadCloser) {
		if err := Body.Close(); err != nil {
			log.Printf("Failed to close response body reader: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		log.Printf("Failed to download profile photo for user %d: status %d", userID, resp.StatusCode)
		return nil
	}

	var avatar bytes.Buffer
	if _, err := io.Copy(&avatar, io.LimitReader(resp.Body, 5<<20)); err != nil {
		log.Printf("Failed to read profile photo for user %d: %v", userID, err)
		return nil
	}
	return avatar.Bytes()
}

func quoteHandler(queue Queue) func(b *gotgbot.Bot, ctx *ext.Context) error {
	return func(b *gotgbot.Bot, ctx *ext.Context) error {
		const maxQuoteMessages = 50
		msg := ctx.EffectiveMessage
		var dpi *int
		count := 1
		theme := "light"

		args := ctx.Args()
		if len(args) > 1 {
			for _, arg := range args[1:] {
				argLower := strings.ToLower(arg)
				switch {
				case argLower == "dark" || argLower == "light":
					theme = argLower
				case strings.HasPrefix(argLower, "x"):
					n, err := strconv.Atoi(argLower[1:])
					if err != nil || n < 1 {
						_, _ = msg.Reply(b, "Message count must look like `xN` e.g. `x3`.", nil)
						return nil
					}
					count = min(n, maxQuoteMessages)
				default:
					parsed, err := strconv.Atoi(argLower)
					if err != nil {
						_, _ = msg.Reply(b, "DPI must be an integer, or theme must be 'dark'/'light'.", nil)
						return nil
					}
					parsed = min(max(parsed, 72), 300)
					dpi = &parsed
				}
			}
		}

		reply := msg.ReplyToMessage
		if reply == nil {
			_, _ = msg.Reply(b, "Please specify a message for quote generation", nil)
			return nil
		}
		if reply.From == nil {
			_, _ = msg.Reply(b, "Can't quote messages without an author", nil)
			return nil
		}

		msgs, tags := quotedMessages(b, reply, count)

		userTag := map[int64]string{}
		for _, m := range msgs {
			if m.From != nil && tags[m.MessageId] != "" {
				userTag[m.From.Id] = tags[m.MessageId]
			}
		}

		cache := newSenderCache(b, reply.Chat.Id)
		var qmsgs []*pb.QuoteMessage
		var senderOf []*sender

		for _, m := range msgs {
			if m.From == nil {
				continue
			}
			text, ents := messageText(&m)
			if text == "" {
				continue
			}
			s := cache.get(m.From, userTag[m.From.Id])
			senderOf = append(senderOf, s)
			qmsgs = append(qmsgs, &pb.QuoteMessage{
				MessageId: m.MessageId,
				Author:    s.author,
				Content:   text,
				Entities:  convertEntities(text, ents),
			})
		}
		if len(qmsgs) == 0 {
			_, _ = msg.Reply(b, "Nothing to quote: no text messages found.", nil)
			return nil
		}

		for i, qm := range qmsgs {
			if i+1 == len(qmsgs) || senderOf[i+1] != senderOf[i] {
				qm.Image = senderOf[i].avatar
			}
		}

		quoteReq := &pb.QuoteRequest{
			ChatId:   reply.Chat.Id,
			Theme:    theme,
			Dpi:      optionalUint32(dpi),
			Messages: qmsgs,
		}

		pbData, err := proto.Marshal(quoteReq)
		if err != nil {
			log.Printf("Failed to marshal proto: %v", err)
			return err
		}

		encoder, err := zstd.NewWriter(nil, zstd.WithEncoderLevel(zstd.SpeedBestCompression))
		if err != nil {
			log.Printf("Failed to create zstd encoder: %v", err)
			return err
		}
		compressedData := encoder.EncodeAll(pbData, make([]byte, 0, len(pbData)))

		startMs := time.Now().UnixMilli()
		jobCtx, cancel := context.WithTimeout(context.Background(), time.Minute)
		defer cancel()

		err = queue.Enqueue(jobCtx, "generate:jobs", compressedData)
		if err != nil {
			log.Printf("Failed to enqueue job: %v", err)
			return err
		}

		resultData, err := queue.Dequeue(jobCtx, "generate:results")
		if err != nil {
			if errors.Is(err, context.DeadlineExceeded) || errors.Is(err, context.Canceled) {
				_, _ = msg.Reply(b, "Quote generation timed out", nil)
				return nil
			}
			log.Printf("Failed to dequeue result: %v", err)
			return err
		}

		decoder, err := zstd.NewReader(nil)
		if err != nil {
			log.Printf("Failed to create zstd decoder: %v", err)
			return err
		}
		defer decoder.Close()
		decompressedData, err := decoder.DecodeAll(resultData, nil)
		if err != nil {
			log.Printf("Failed to decompress result: %v", err)
			return err
		}

		var resultImg pb.ResultImage
		err = proto.Unmarshal(decompressedData, &resultImg)
		if err != nil {
			log.Printf("Failed to unmarshal result: %v", err)
			return err
		}

		log.Printf("\x1b[37;41mTime spent: %d ms\x1b[0m", time.Now().UnixMilli()-startMs)

		chatID := reply.Chat.Id
		if resultImg.ChatId != 0 {
			chatID = resultImg.ChatId
		}

		msgID := reply.MessageId
		if resultImg.MessageId != 0 {
			msgID = resultImg.MessageId
		}

		_, err = b.SendSticker(chatID, gotgbot.InputFileByReader("quote.webp", bytes.NewReader(resultImg.Image)),
			&gotgbot.SendStickerOpts{
				ReplyParameters: &gotgbot.ReplyParameters{MessageId: msgID},
			})
		if err != nil {
			log.Printf("Failed to send sticker: %v", err)
		}

		return nil
	}
}
