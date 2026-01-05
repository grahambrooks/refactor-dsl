// Package mylib v2.0.0 - A sample Go library for demonstrating API upgrades.
//
// Breaking changes from v1.0.0:
// - User renamed to UserAccount with new Email field
// - Status renamed to ConnectionStatus
// - GetUser renamed to FetchUser
// - ProcessData renamed to TransformData
// - Connect now requires port and timeout parameters
// - Save no longer takes sync parameter
// - Query parameters reordered from (a, b, c) to (c, a, b)
// - Find now returns (*Item, error) instead of Item
// - Parse now returns ParseResult instead of (string, error)
// - DeprecatedFn has been removed
// - Utils renamed to Helpers
package mylib

import (
	"errors"
	"fmt"
	"time"
)

// UserAccount represents a user account in the system (renamed from User).
type UserAccount struct {
	ID    int64
	Name  string
	Email string // New field in v2
}

// ConnectionStatus represents connection status (renamed from Status).
type ConnectionStatus int

const (
	ConnectionStatusConnected ConnectionStatus = iota
	ConnectionStatusDisconnected
	ConnectionStatusError
)

// Config holds configuration for the library (with new Port field).
type Config struct {
	Host string
	Port int // New field in v2
}

// Item represents a database query result item.
type Item struct {
	Key   string
	Value string
}

// ParseResult represents the result of parsing operations (new in v2).
type ParseResult struct {
	Output   string
	Warnings []string
}

// FetchUser fetches a user by their ID (renamed from GetUser).
func FetchUser(id int64) (*UserAccount, error) {
	if id <= 0 {
		return nil, errors.New("invalid id")
	}
	return &UserAccount{
		ID:    id,
		Name:  fmt.Sprintf("User%d", id),
		Email: fmt.Sprintf("user%d@example.com", id),
	}, nil
}

// TransformData transforms raw data bytes (renamed from ProcessData).
func TransformData(data []byte) []byte {
	result := make([]byte, len(data))
	for i, b := range data {
		result[i] = b + 1
	}
	return result
}

// Connect connects to a host with port and timeout (signature changed).
func Connect(host string, port int, timeout time.Duration) (ConnectionStatus, error) {
	if host == "" {
		return ConnectionStatusError, errors.New("empty host")
	}
	if port <= 0 {
		return ConnectionStatusError, errors.New("invalid port")
	}
	_ = timeout
	return ConnectionStatusConnected, nil
}

// Save saves data (sync parameter removed).
func Save(data string) error {
	fmt.Printf("Saved: %s\n", data)
	return nil
}

// Query queries with reordered parameters (c, a, b instead of a, b, c).
func Query(c bool, a string, b int) []Item {
	var results []Item
	if c {
		results = append(results, Item{Key: a, Value: fmt.Sprintf("%d", b)})
	}
	return results
}

// Find finds an item by key (now returns *Item, error).
func Find(key string) (*Item, error) {
	if key == "" {
		return nil, errors.New("empty key")
	}
	return &Item{Key: key, Value: "found"}, nil
}

// Parse parses a string into a ParseResult (return type changed).
func Parse(input string) ParseResult {
	if input == "" {
		return ParseResult{Output: "", Warnings: []string{"empty input"}}
	}
	return ParseResult{Output: input, Warnings: nil}
}

// Note: DeprecatedFn has been removed in v2

// Helpers provides helper functions (renamed from Utils).
type Helpers struct{}

// Helper returns a greeting.
func (h Helpers) Helper() string {
	return "Hello from helpers"
}

// FormatUser formats a user account for display.
func (h Helpers) FormatUser(user *UserAccount) string {
	return fmt.Sprintf("UserAccount(id=%d, name=%s, email=%s)", user.ID, user.Name, user.Email)
}

// CreateConfigMap creates a config map.
func (h Helpers) CreateConfigMap() map[string]string {
	return map[string]string{"version": "2.0.0"}
}
