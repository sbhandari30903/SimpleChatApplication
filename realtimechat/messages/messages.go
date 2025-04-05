package messages

import (
	"fmt"
	"sync"
	"time"
)

// Message represents a chat message with metadata.
type Message struct {
	SenderID   int       `json:"sender_id"`
	ReceiverID int       `json:"receiver_id"`
	Content    string    `json:"content"`
	Timestamp  time.Time `json:"timestamp"`
}

var (
	messageStore = make(map[string][]Message)
	msgMutex     sync.RWMutex
)

// getMessageKey ensures consistent key for both sender-receiver and receiver-sender
func getMessageKey(id1, id2 int) string {
	if id1 < id2 {
		return fmt.Sprintf("%d_%d", id1, id2)
	}
	return fmt.Sprintf("%d_%d", id2, id1)
}

// SaveMessage adds a message to the in-memory message store.
func SaveMessage(senderID, receiverID int, content string) {
	msg := Message{
		SenderID:   senderID,
		ReceiverID: receiverID,
		Content:    content,
		Timestamp:  time.Now(),
	}

	key := getMessageKey(senderID, receiverID)

	msgMutex.Lock()
	messageStore[key] = append(messageStore[key], msg)
	msgMutex.Unlock()
}

// GetMessages retrieves the chat history between two users.
func GetMessages(userID1, userID2 int) []Message {
	key := getMessageKey(userID1, userID2)

	msgMutex.RLock()
	messages := messageStore[key]
	msgMutex.RUnlock()

	
	return messages
}
