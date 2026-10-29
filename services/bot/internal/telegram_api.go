package bot

import (
	"bot/pb"
	"encoding/json"
	"fmt"
	"log"
	"sort"

	"github.com/PaulSonOfLars/gotgbot/v2"
)

type ForkUser struct {
	gotgbot.User
	EmojiStatusCustomEmojiId string `json:"emoji_status_custom_emoji_id,omitempty"`
}

type sender struct {
	index  uint32
	author *pb.Author
	avatar []byte
}

type senderCache struct {
	b      *gotgbot.Bot
	chatID int64
	byID   map[int64]*sender
}

func newSenderCache(b *gotgbot.Bot, chatID int64) *senderCache {
	return &senderCache{b: b, chatID: chatID, byID: map[int64]*sender{}}
}

func (c *senderCache) get(u *gotgbot.User) *sender {
	if s, ok := c.byID[u.Id]; ok {
		return s
	}

	status, title := "member", ""
	if cm, err := c.b.GetChatMember(c.chatID, u.Id, nil); err != nil {
		log.Printf("Failed to get chat member %d: %v", u.Id, err)
	} else {
		if st := cm.GetStatus(); st != "" {
			status = st
		}
		switch m := cm.(type) {
		case gotgbot.ChatMemberAdministrator:
			title = m.CustomTitle
		case gotgbot.ChatMemberOwner:
			title = m.CustomTitle
		}
	}

	var emoji string
	if IsLocalAPI(c.b) {
		if info, err := GetUserInfo(c.b, u.Id); err != nil {
			log.Printf("Failed to get user info %d: %v", u.Id, err)
		} else {
			emoji = info.EmojiStatusCustomEmojiId
		}
	}

	s := &sender{
		index: uint32(len(c.byID)),
		author: &pb.Author{
			ColorIndex:  int32(u.Id % 7),
			Username:    u.FirstName,
			UserStatus:  optionalString(title),
			UserRole:    &status,
			EmojiStatus: optionalString(emoji),
		},
		avatar: userAvatar(c.b, u.Id),
	}
	s.author.SenderIndex = s.index
	c.byID[u.Id] = s
	return s
}

func messageText(m *gotgbot.Message) (string, []gotgbot.MessageEntity) {
	if m.Text != "" {
		return m.Text, m.Entities
	}
	return m.Caption, m.CaptionEntities
}

func quotedMessages(b *gotgbot.Bot, first *gotgbot.Message, count int) []gotgbot.Message {
	out := []gotgbot.Message{*first}
	if count <= 1 || !IsLocalAPI(b) {
		if count > 1 {
			log.Printf("multi-quote requested but IsLocalAPI=false; quoting one message")
		}
		return out
	}

	ids := make([]int64, 0, count-1)
	for i := 1; i < count; i++ {
		ids = append(ids, first.MessageId+int64(i))
	}
	got, err := GetMessages(b, first.Chat.Id, ids)
	if err != nil {
		log.Printf("getMessages failed, quoting a single message: %v", err)
		return out
	}
	sort.Slice(got, func(i, j int) bool { return got[i].MessageId < got[j].MessageId })
	return append(out, got...)
}

func GetMessages(b *gotgbot.Bot, chatID int64, messageIDs []int64) ([]gotgbot.Message, error) {
	if len(messageIDs) == 0 {
		return nil, nil
	}
	if len(messageIDs) > 50 {
		return nil, fmt.Errorf("message_ids exceeds maximum limit of 50 (got %d)", len(messageIDs))
	}

	raw, err := b.Request("getMessages", map[string]any{
		"chat_id":     chatID,
		"message_ids": messageIDs,
	}, nil)
	if err != nil {
		return nil, err
	}

	var messages []gotgbot.Message
	if err := json.Unmarshal(raw, &messages); err != nil {
		return nil, fmt.Errorf("failed to unmarshal getMessages response: %w", err)
	}

	return messages, nil
}

func GetUserInfo(b *gotgbot.Bot, userID int64) (*ForkUser, error) {
	raw, err := b.Request("getUserInfo", map[string]any{
		"user_id": userID,
	}, nil)
	if err != nil {
		return nil, err
	}

	var user ForkUser
	if err := json.Unmarshal(raw, &user); err != nil {
		return nil, fmt.Errorf("failed to unmarshal getUserInfo response: %w", err)
	}

	return &user, nil
}

func IsLocalAPI(b *gotgbot.Bot) bool {
	if b == nil {
		return false
	}
	if baseClient, ok := b.BotClient.(*gotgbot.BaseBotClient); ok {
		return baseClient.DefaultRequestOpts != nil && baseClient.DefaultRequestOpts.APIURL != ""
	}
	return false
}
