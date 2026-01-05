// Package mylib v1.0.0 - A sample Go library for demonstrating API upgrades.
package mylib

import (
	"errors"
	"fmt"
)

// User represents a user in the system.
type User struct {
	ID   int64
	Name string
}

// Status represents connection status.
type Status int

const (
	StatusConnected Status = iota
	StatusDisconnected
	StatusError
)

// Config holds configuration for the library.
type Config struct {
	Host string
}

// Item represents a database query result item.
type Item struct {
	Key   string
	Value string
}

// GetUser gets a user by their ID.
func GetUser(id int64) (*User, error) {
	if id <= 0 {
		return nil, errors.New("invalid id")
	}
	return &User{ID: id, Name: fmt.Sprintf("User%d", id)}, nil
}

// ProcessData processes raw data bytes.
func ProcessData(data []byte) []byte {
	result := make([]byte, len(data))
	for i, b := range data {
		result[i] = b + 1
	}
	return result
}

// Connect connects to a host.
func Connect(host string) (Status, error) {
	if host == "" {
		return StatusError, errors.New("empty host")
	}
	return StatusConnected, nil
}

// Save saves data with optional sync.
func Save(data string, sync bool) error {
	if sync {
		// Force sync
	}
	fmt.Printf("Saved: %s\n", data)
	return nil
}

// Query queries with multiple parameters.
func Query(a string, b int, c bool) []Item {
	var results []Item
	if c {
		results = append(results, Item{Key: a, Value: fmt.Sprintf("%d", b)})
	}
	return results
}

// Find finds an item by key.
func Find(key string) Item {
	return Item{Key: key, Value: "found"}
}

// Parse parses a string.
func Parse(input string) (string, error) {
	if input == "" {
		return "", errors.New("empty input")
	}
	return fmt.Sprintf("%s", input), nil
}

// DeprecatedFn is deprecated and will be removed.
// Deprecated: Use FetchUser instead.
func DeprecatedFn() {
	fmt.Println("This function is deprecated")
}

// Utils provides utility functions.
type Utils struct{}

// Helper returns a greeting.
func (u Utils) Helper() string {
	return "Hello from utils"
}

// FormatUser formats a user for display.
func (u Utils) FormatUser(user *User) string {
	return fmt.Sprintf("User(id=%d, name=%s)", user.ID, user.Name)
}

// CreateConfigMap creates a config map.
func (u Utils) CreateConfigMap() map[string]string {
	return map[string]string{"version": "1.0.0"}
}
