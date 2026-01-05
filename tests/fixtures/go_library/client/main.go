// Client application using mylib v1.0.0
//
// This client needs to be upgraded to work with mylib v2.0.0.
// The upgrade analyzer should detect all the necessary changes.
package main

import (
	"fmt"
)

// Simulated types (from v1)
type User struct {
	ID   int64
	Name string
}

type Status int

const (
	StatusConnected Status = iota
	StatusDisconnected
)

type Config struct {
	Host string
}

type Item struct {
	Key   string
	Value string
}

// Simulated library functions
func GetUser(id int64) (*User, error) {
	return &User{ID: id, Name: fmt.Sprintf("User%d", id)}, nil
}

func ProcessData(data []byte) []byte {
	result := make([]byte, len(data))
	for i, b := range data {
		result[i] = b + 1
	}
	return result
}

func Connect(host string) (Status, error) {
	return StatusConnected, nil
}

func Save(data string, sync bool) error {
	fmt.Printf("Saved: %s, sync=%v\n", data, sync)
	return nil
}

func Query(a string, b int, c bool) []Item {
	if c {
		return []Item{{Key: a, Value: fmt.Sprintf("%d", b)}}
	}
	return nil
}

func Find(key string) Item {
	return Item{Key: key, Value: "found"}
}

func Parse(input string) (string, error) {
	return input, nil
}

func DeprecatedFn() {
	fmt.Println("deprecated")
}

type Utils struct{}

func (u Utils) Helper() string {
	return "helper"
}

func main() {
	fmt.Println("=== Client Application ===\n")

	// Using GetUser (should become FetchUser)
	user, _ := GetUser(42)
	fmt.Printf("Found user: %+v\n", user)

	// Using another GetUser call
	admin, _ := GetUser(1)
	fmt.Printf("Admin: %+v\n", admin)

	// Using ProcessData (should become TransformData)
	data := []byte{1, 2, 3, 4, 5}
	processed := ProcessData(data)
	fmt.Printf("Processed data: %v\n", processed)

	// Using Connect (needs port and timeout parameters)
	config := Config{Host: "localhost"}
	status, _ := Connect(config.Host)
	fmt.Printf("Connected: %v\n", status)

	// Using Save (sync parameter should be removed)
	Save("important data", true)
	Save("more data", false)

	// Using Query (parameters should be reordered)
	results := Query("search", 10, true)
	fmt.Printf("Query results: %d items\n", len(results))

	// Another Query call
	moreResults := Query("filter", 5, false)
	fmt.Printf("More results: %d items\n", len(moreResults))

	// Using Find (should return (*Item, error) now)
	item := Find("my_key")
	fmt.Printf("Found item: %s\n", item.Key)

	// Using Parse (return type changed)
	result, _ := Parse("hello world")
	fmt.Printf("Parsed: %s\n", result)

	// Using DeprecatedFn (should be removed)
	DeprecatedFn()

	// Using Utils (should become Helpers)
	utils := Utils{}
	greeting := utils.Helper()
	fmt.Printf("Greeting: %s\n", greeting)

	fmt.Println("\n=== Done ===")
}

// UserService is a service that uses the library.
type UserService struct {
	config Config
}

func NewUserService(host string) *UserService {
	return &UserService{config: Config{Host: host}}
}

func (s *UserService) GetUserByID(id int64) (*User, error) {
	// Uses GetUser internally
	return GetUser(id)
}

func (s *UserService) ProcessUserData(data []byte) []byte {
	// Uses ProcessData internally
	return ProcessData(data)
}

func (s *UserService) ConnectToServer() (Status, error) {
	// Uses Connect internally
	return Connect(s.config.Host)
}

func (s *UserService) SaveUser(user *User) error {
	// Uses Save internally
	data := fmt.Sprintf("%d:%s", user.ID, user.Name)
	return Save(data, true)
}

func (s *UserService) SearchUsers(queryStr string, limit int) []Item {
	// Uses Query internally
	return Query(queryStr, limit, true)
}

func (s *UserService) FindUserItem(key string) Item {
	// Uses Find internally
	return Find(key)
}

func (s *UserService) GetHelperMessage() string {
	// Uses Utils.Helper internally
	utils := Utils{}
	return utils.Helper()
}
