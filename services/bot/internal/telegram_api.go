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

func (c *senderCache) get(u *gotgbot.User, tag string) *sender {
	if s, ok := c.byID[u.Id]; ok {
		return s
	}

	status, title := "member", ""
	if tag != "" {
		title = tag
	}
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

func quotedMessages(b *gotgbot.Bot, first *gotgbot.Message, count int) ([]gotgbot.Message, map[int64]string) {
	single := []gotgbot.Message{*first}
	if !IsLocalAPI(b) {
		if count > 1 {
			log.Printf("multi-quote requested but IsLocalAPI=false; quoting one message")
		}
		return single, nil
	}

	ids := make([]int64, 0, count)
	for i := 0; i < count && first.MessageId-int64(i) > 0; i++ {
		ids = append(ids, first.MessageId-int64(i))
	}
	got, tags, err := GetMessagesWithTags(b, first.Chat.Id, ids)
	if err != nil {
		log.Printf("getMessages failed, quoting a single message: %v", err)
		return single, nil
	}
	hasFirst := false
	for i := range got {
		if got[i].MessageId == first.MessageId {
			hasFirst = true
			break
		}
	}
	if !hasFirst {
		got = append(got, *first)
	}

	sort.Slice(got, func(i, j int) bool { return got[i].MessageId < got[j].MessageId })
	return got, tags
}

func GetMessagesWithTags(b *gotgbot.Bot, chatID int64, ids []int64) ([]gotgbot.Message, map[int64]string, error) {
	if len(ids) == 0 {
		return nil, nil, nil
	}
	if len(ids) > 50 {
		return nil, nil, fmt.Errorf("message_ids exceeds maximum limit of 50 (got %d)", len(ids))
	}
	raw, err := b.Request("getMessages", map[string]any{
		"chat_id":     chatID,
		"message_ids": ids,
	}, nil)
	if err != nil {
		return nil, nil, err
	}

	var msgs []gotgbot.Message
	if err := json.Unmarshal(raw, &msgs); err != nil {
		return nil, nil, fmt.Errorf("unmarshal getMessages: %w", err)
	}
	var extra []struct {
		MessageId int64  `json:"message_id"`
		SenderTag string `json:"sender_tag"`
	}
	if err := json.Unmarshal(raw, &extra); err != nil {
		return nil, nil, fmt.Errorf("unmarshal sender_tag: %w", err)
	}
	tags := make(map[int64]string, len(extra))
	for _, e := range extra {
		if e.SenderTag != "" {
			tags[e.MessageId] = e.SenderTag
		}
	}
	return msgs, tags, nil
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
