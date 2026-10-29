package bot

import (
	"encoding/json"
	"fmt"

	"github.com/PaulSonOfLars/gotgbot/v2"
)

type ForkUser struct {
	gotgbot.User
	EmojiStatusCustomEmojiId string `json:"emoji_status_custom_emoji_id,omitempty"`
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
