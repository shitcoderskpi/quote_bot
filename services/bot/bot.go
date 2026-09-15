package main

import (
	"bytes"
	"context"
	"fmt"
	"html"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"
	"unicode/utf16"

	pb "bot/proto"

	"github.com/PaulSonOfLars/gotgbot/v2"
	"github.com/PaulSonOfLars/gotgbot/v2/ext"
	"github.com/klauspost/compress/zstd"
	"google.golang.org/protobuf/proto"
)

func convertEntities(text string, entities []gotgbot.MessageEntity) []*pb.Entity {
	if len(entities) == 0 {
		return nil
	}
	var result []*pb.Entity
	utf16Text := utf16.Encode([]rune(text))

	for _, ent := range entities {
		valid := false
		switch ent.Type {
		case "bold",
			"italic",
			"underline",
			"strikethrough",
			"code",
			"pre",
			"text_link",
			"url",
			"mention",
			"bot_command",
			"hashtag",
			"cashtag",
			"email",
			"phone_number",
			"text_mention",
			"spoiler":
			valid = true
		}
		if !valid {
			continue
		}

		if int(ent.Offset) > len(utf16Text) {
			continue
		}
		end := min(int(ent.Offset+ent.Length), len(utf16Text))

		prefix := utf16Text[:ent.Offset]
		entity := utf16Text[ent.Offset:end]

		prefixStr := string(utf16.Decode(prefix))
		entityStr := string(utf16.Decode(entity))

		byteOffset := len(prefixStr)
		byteLength := len(entityStr)

		result = append(result, &pb.Entity{
			Type:   ent.Type,
			Offset: int32(byteOffset),
			Length: int32(byteLength),
		})
	}
	return result
}

func startHandler(b *gotgbot.Bot, ctx *ext.Context) error {
	_, err := ctx.EffectiveMessage.Reply(b, fmt.Sprintf("Hello, <b>%s</b>!", html.EscapeString(ctx.EffectiveUser.FirstName)), &gotgbot.SendMessageOpts{ParseMode: "HTML"})
	return err
}

func quoteHandler(redisQueue *RedisQueue) func(b *gotgbot.Bot, ctx *ext.Context) error {
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

		var avatar bytes.Buffer
		photos, err := b.GetUserProfilePhotos(reply.From.Id, &gotgbot.GetUserProfilePhotosOpts{Limit: 1})
		if err == nil && photos.TotalCount > 0 {
			photo := photos.Photos[0][0]
			file, err := b.GetFile(photo.FileId, nil)
			if err == nil {
				resp, err := http.Get(file.URL(b, nil))
				if err == nil {
					defer func(Body io.ReadCloser) {
						err := Body.Close()
						if err != nil {
							log.Panicf("Failed to close response body: %v", err)
						}
					}(resp.Body)
					_, err := io.Copy(&avatar, resp.Body)
					if err != nil {
						return err
					}
				}
			}
		} else {
			log.Printf("User %d does not have any photos or failed to get them", reply.From.Id)
		}

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

		var customTitlePtr *string
		if customTitle != "" {
			customTitlePtr = &customTitle
		}

		var dpiPtr *int32
		if dpi != nil {
			val := int32(*dpi)
			dpiPtr = &val
		}

		serializableMsg := &pb.SerializableMessage{
			GradId:     int32(reply.From.Id % 7),
			Username:   reply.From.FirstName,
			UserStatus: customTitlePtr,
			UserRole:   &userStatus,
			Content:    text,
			Entities:   convertedEntities,
			Image:      avatar.Bytes(),
			MessageId:  reply.MessageId,
			ChatId:     reply.Chat.Id,
			Dpi:        dpiPtr,
			Theme:      theme,
		}

		pbData, err := proto.Marshal(serializableMsg)
		if err != nil {
			log.Printf("Failed to marshal proto: %v", err)
			return err
		}

		encoder, _ := zstd.NewWriter(nil, zstd.WithEncoderLevel(zstd.SpeedBestCompression))
		compressedData := encoder.EncodeAll(pbData, make([]byte, 0, len(pbData)))

		startMs := time.Now().UnixMilli()
		ctxBg := context.Background()

		err = redisQueue.Enqueue(ctxBg, "generate:jobs", compressedData)
		if err != nil {
			log.Printf("Failed to enqueue job: %v", err)
			return err
		}

		resultData, err := redisQueue.Dequeue(ctxBg, "generate:results", 0)
		if err != nil {
			log.Printf("Failed to dequeue result: %v", err)
			return err
		}

		decoder, _ := zstd.NewReader(nil)
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
