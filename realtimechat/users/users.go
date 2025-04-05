package users

import (
	"sync"
)

// User represents the user model with ID, First Name, and Last Name.
type User struct {
	ID        int    `json:"id"`
	FirstName string `json:"first_name"`
	LastName  string `json:"last_name"`
}

// In-memory store for users.
var users = make(map[int]User)
var idCounter int
var mu sync.Mutex

// Generate a new unique user ID
func generateUserID() int {
	mu.Lock()
	defer mu.Unlock()
	idCounter++
	return idCounter
}

// Add a user to the in-memory store and return the generated user.
func AddUser(firstName, lastName string) User {
	userID := generateUserID()
	user := User{
		ID:        userID,
		FirstName: firstName,
		LastName:  lastName,
	}
	users[userID] = user
	return user
}

// Get a user by ID
func GetUserByID(userID int) (User, bool) {
	user, exists := users[userID]
	return user, exists
}

// Login a user by matching first name and last name
func LoginUser(firstName, lastName string) (User, bool) {
	for _, user := range users {
		if user.FirstName == firstName && user.LastName == lastName {
			return user, true
		}
	}
	return User{}, false
}

func GetAllUsers() []User {
	mu.Lock()
	defer mu.Unlock()

	var userList []User
	for _, user := range users {
		userList = append(userList, user)
	}
	return userList
}


func init() {
	AddUser("a", "a")
	AddUser("b", "b")
	AddUser("sam", "sam")
	AddUser("richard", "richard")

}


