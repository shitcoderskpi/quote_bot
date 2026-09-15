package bot

import (
	"unicode/utf8"

	"bot/pb"

	"github.com/PaulSonOfLars/gotgbot/v2"
)

func supportedEntityType(t string) bool {
	switch t {
	case "bold", "italic", "underline", "strikethrough", "code", "pre",
		"text_link", "url", "mention", "bot_command", "hashtag",
		"cashtag", "email", "phone_number", "text_mention", "spoiler":
		return true
	}
	return false
}

func convertEntities(text string, entities []gotgbot.MessageEntity) []*pb.Entity {
	if len(entities) == 0 {
		return nil
	}

	u16ToByte := []int{0}
	for i, r := range text {
		next := i + utf8.RuneLen(r)
		u16ToByte = append(u16ToByte, next)
		if r > 0xffff {
			u16ToByte = append(u16ToByte, next)
		}
	}

	result := make([]*pb.Entity, 0, len(entities))
	for _, ent := range entities {
		if !supportedEntityType(ent.Type) {
			continue
		}

		start := int(ent.Offset)
		if start >= len(u16ToByte) {
			continue
		}
		end := min(start+int(ent.Length), len(u16ToByte)-1)

		result = append(result, &pb.Entity{
			Type:   ent.Type,
			Offset: int32(u16ToByte[start]),
			Length: int32(u16ToByte[end] - u16ToByte[start]),
		})
	}
	return result
}
