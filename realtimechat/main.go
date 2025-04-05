package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"github.com/gorilla/websocket"
	"realtimechat/users" // Replace with the correct path to users.go file
	"sync"
	"strconv"
	"github.com/rs/cors"
	"realtimechat/messages"
	"time"
)

// WebSocket upgrader
var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },

}

// A map to store the WebSocket connections associated with each userId.
var userConnections = make(map[int]*websocket.Conn)
var mu sync.Mutex // Mutex to protect access to the map

// Home page
func homePage(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "home page")
}

// Message structure to handle the incoming message
// type Message struct {
// 	UserID  int    `json:"user_id"`
// 	Content string `json:"content"`
// }

type Message struct {
	SenderID   int    `json:"sender_id"`
	ReceiverID int    `json:"receiver_id"`
	Content    string `json:"content"`
	Timestamp  string `json:"timestamp"` // Add Timestamp field

}


// WebSocket reader
// WebSocket reader
func reader(conn *websocket.Conn) {
	defer conn.Close()

	for {
		messageType, p, err := conn.ReadMessage()
		if err != nil {
			log.Println("Error reading message:", err)
			return
		}

		log.Println("Received message:", string(p))

		var msg Message
		if err := json.Unmarshal(p, &msg); err != nil {
			log.Println("Error unmarshalling message:", err)
			continue
		}

		// Add a timestamp to the message
		msg.Timestamp = time.Now().Format(time.RFC3339)

		// Save the message to in-memory message store
		messages.SaveMessage(msg.SenderID, msg.ReceiverID, msg.Content)

		// Prepare message in the correct format to send
		responseMsg := struct {
			SenderID   int    `json:"sender_id"`
			ReceiverID int    `json:"receiver_id"`
			Content    string `json:"content"`
			Timestamp  string `json:"timestamp"`
		}{
			SenderID:   msg.SenderID,
			ReceiverID: msg.ReceiverID,
			Content:    msg.Content,
			Timestamp:  msg.Timestamp,
		}

		// Deliver message if receiver is connected
		mu.Lock()
		destinationConn, exists := userConnections[msg.ReceiverID]
		mu.Unlock()

		if exists {
			log.Printf("Sending message to user %d: %s\n", msg.ReceiverID, msg.Content)
			// Marshal the response message to send back in JSON format
			respMsg, err := json.Marshal(responseMsg)
			if err != nil {
				log.Println("Error marshalling response message:", err)
				continue
			}

			if err := destinationConn.WriteMessage(messageType, respMsg); err != nil {
				log.Println("Error sending message:", err)
			}
		} else {
			log.Printf("User %d not connected\n", msg.ReceiverID)
		}
	}
}



// WebSocket endpoint
func wsEndpoint(w http.ResponseWriter, r *http.Request) {

	// Extract userId from query parameters
	userIdStr := r.URL.Query().Get("userId")
	if userIdStr == "" {
		http.Error(w, "userId is missing", http.StatusBadRequest)
		return
	}

	// Convert userId to integer
	userId, err := strconv.Atoi(userIdStr)
	if err != nil {
		http.Error(w, "Invalid userId", http.StatusBadRequest)
		return
	}

	// Upgrade the HTTP connection to a WebSocket connection
	ws, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println(err)
		http.Error(w, "Failed to upgrade to WebSocket", http.StatusInternalServerError)
		return
	}

	// Store the connection in the map
	mu.Lock()
	userConnections[userId] = ws
	mu.Unlock()

	log.Println("Client successfully connected, userId:", userId)

	// Handle reading messages from WebSocket
	reader(ws)

	// Clean up when connection is closed
	mu.Lock()
	delete(userConnections, userId)
	mu.Unlock()
}

// Register user endpoint (POST)
func registerUser(w http.ResponseWriter, r *http.Request) {
	var input struct {
		FirstName string `json:"first_name"`
		LastName  string `json:"last_name"`
	}

	// Decode the request body into input struct
	if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	// Add the new user and generate an ID
	user := users.AddUser(input.FirstName, input.LastName)

	// Send back the user info with the generated ID
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(user)
}

// Login user endpoint (POST)
func loginUser(w http.ResponseWriter, r *http.Request) {
	var input struct {
		FirstName string `json:"first_name"`
		LastName  string `json:"last_name"`
	}

	// Decode the request body into input struct
	if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	// Try to login the user by first name and last name
	user, found := users.LoginUser(input.FirstName, input.LastName)
	if !found {
		http.Error(w, "Forbidden", http.StatusForbidden)
		return
	}

	// Send back the user ID if found
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(user)
}

// Get all users endpoint (GET)
func getAllUsers(w http.ResponseWriter, r *http.Request) {
	allUsers := users.GetAllUsers()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(allUsers)
}

// GetMessagesBetweenUsers endpoint (GET)
func getMessagesBetweenUsers(w http.ResponseWriter, r *http.Request) {
	// Extract userId1 and userId2 from query parameters
	userId1Str := r.URL.Query().Get("userId1")
	userId2Str := r.URL.Query().Get("userId2")

	// Check if both userId1 and userId2 are provided
	if userId1Str == "" || userId2Str == "" {
		http.Error(w, "userId1 and userId2 are required", http.StatusBadRequest)
		return
	}

	// Convert userId1 and userId2 to integers
	userId1, err := strconv.Atoi(userId1Str)
	if err != nil {
		http.Error(w, "Invalid userId1", http.StatusBadRequest)
		return
	}

	userId2, err := strconv.Atoi(userId2Str)
	if err != nil {
		http.Error(w, "Invalid userId2", http.StatusBadRequest)
		return
	}

	// Retrieve the messages between userId1 and userId2 from the message store
	messages := messages.GetMessages(userId1, userId2)
	fmt.Println("messages request ...", messages)

	// Send the messages back as a response
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(messages)
}

func setupRoutes() {
	http.HandleFunc("/", homePage)
	http.HandleFunc("/ws", wsEndpoint)
	http.HandleFunc("/register", registerUser)  // Endpoint for registering users
	http.HandleFunc("/login", loginUser)        // New endpoint for login users
	http.HandleFunc("/users", getAllUsers) // New route to list all users
	http.HandleFunc("/messages", getMessagesBetweenUsers)  // New route to get messages between two users

}

func main() {
	fmt.Println("Server starting...")

	 // Set up CORS middleware
	 c := cors.New(cors.Options{
        AllowedOrigins: []string{"*"}, // Allow your frontend origin
        AllowedMethods: []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
        AllowedHeaders: []string{"Content-Type", "Authorization"},
        AllowCredentials: true,
    })

	setupRoutes()

	// Apply CORS middleware
    handler := c.Handler(http.DefaultServeMux)

	// Start the server
	fmt.Println("Server running on port 8082...")
	log.Fatal(http.ListenAndServe(":8082", handler))
}
