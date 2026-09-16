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

func startHandler(b *gotgbot.Bot, ctx *ext.Context) error {
	_, err := ctx.EffectiveMessage.Reply(b, fmt.Sprintf("Hello, <b>%s</b>!", html.EscapeString(ctx.EffectiveUser.FirstName)), &gotgbot.SendMessageOpts{ParseMode: "HTML"})
	return err
}

func optionalString(s string) *string {
	if s == "" {
		return nil
	}
	return proto.String(s)
}

func optionalInt32(v *int) *int32 {
	if v == nil {
		return nil
	}
	return proto.Int32(int32(*v))
}

func userAvatar(b *gotgbot.Bot, userID int64) []byte {
	photos, err := b.GetUserProfilePhotos(userID, &gotgbot.GetUserProfilePhotosOpts{Limit: 1})
	if err != nil {
		log.Printf("Failed to get profile photos for user %d: %v", userID, err)
		return nil
	}
	if photos.TotalCount == 0 {
		log.Printf("User %d does not have any photos", userID)
		return nil
	}

	file, err := b.GetFile(photos.Photos[0][0].FileId, nil)
	if err != nil {
		log.Printf("Failed to get profile photo file for user %d: %v", userID, err)
		return nil
	}

	resp, err := avatarClient.Get(file.URL(b, nil))
	if err != nil {
		log.Printf("Failed to download profile photo for user %d: %v", userID, err)
		return nil
	}
	defer resp.Body.Close()

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
		msg := ctx.EffectiveMessage
		var dpi *int
		theme := "light"

		args := ctx.Args()
		if len(args) > 1 {
			for _, arg := range args[1:] {
				argLower := strings.ToLower(arg)
				if argLower == "dark" || argLower == "light" {
					theme = argLower
				} else {
					parsed, err := strconv.Atoi(argLower)
					if err == nil {
						dpi = &parsed
					} else {
						_, _ = msg.Reply(b, "DPI must be an integer, or theme must be 'dark'/'light'.", nil)
						return nil
					}
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

		chatMember, err := b.GetChatMember(reply.Chat.Id, reply.From.Id, nil)
		if err != nil {
			log.Printf("Failed to get chat member: %v", err)
		}

		userStatus := ""
		customTitle := ""
		if chatMember != nil {
			userStatus = chatMember.GetStatus()
			if admin, ok := chatMember.(gotgbot.ChatMemberAdministrator); ok {
				customTitle = admin.CustomTitle
			}
			if owner, ok := chatMember.(gotgbot.ChatMemberOwner); ok {
				customTitle = owner.CustomTitle
			}
		}
		if userStatus == "" {
			userStatus = "member"
		}

		avatar := userAvatar(b, reply.From.Id)

		var text string
		var entities []gotgbot.MessageEntity
		if reply.Text != "" {
			text = reply.Text
			entities = reply.Entities
		} else if reply.Caption != "" {
			text = reply.Caption
			entities = reply.CaptionEntities
		}

		convertedEntities := convertEntities(text, entities)

		serializableMsg := &pb.SerializableMessage{
			GradId:     int32(reply.From.Id % 7),
			Username:   reply.From.FirstName,
			UserStatus: optionalString(customTitle),
			UserRole:   proto.String(userStatus),
			Content:    text,
			Entities:   convertedEntities,
			Image:      avatar,
			MessageId:  reply.MessageId,
			ChatId:     reply.Chat.Id,
			Dpi:        optionalInt32(dpi),
			Theme:      theme,
		}

		pbData, err := proto.Marshal(serializableMsg)
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

		resultData, err := queue.Dequeue(jobCtx, "generate:results", 0)
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
