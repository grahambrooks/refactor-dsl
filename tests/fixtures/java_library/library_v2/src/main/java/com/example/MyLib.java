package com.example;

import java.time.Duration;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * MyLib v2.0.0 - A sample Java library for demonstrating API upgrades.
 *
 * Breaking changes from v1.0.0:
 * - User renamed to UserAccount with new email field
 * - Status renamed to ConnectionStatus
 * - getUser renamed to fetchUser
 * - processData renamed to transformData
 * - connect now requires port and timeout parameters
 * - save no longer takes sync parameter
 * - query parameters reordered from (a, b, c) to (c, a, b)
 * - find now returns Optional<Item> instead of Item
 * - parse now returns ParseResult instead of String
 * - deprecatedFn has been removed
 * - Utils renamed to Helpers
 */
public class MyLib {

    /** A user account in the system (renamed from User). */
    public static class UserAccount {
        private final long id;
        private final String name;
        private final String email; // New field in v2

        public UserAccount(long id, String name, String email) {
            this.id = id;
            this.name = name;
            this.email = email;
        }

        public long getId() { return id; }
        public String getName() { return name; }
        public String getEmail() { return email; }

        @Override
        public String toString() {
            return "UserAccount{id=" + id + ", name='" + name + "', email='" + email + "'}";
        }
    }

    /** Connection status (renamed from Status). */
    public enum ConnectionStatus {
        CONNECTED,
        DISCONNECTED,
        ERROR
    }

    /** Configuration for the library (with new port field). */
    public static class Config {
        private final String host;
        private final int port; // New field in v2

        public Config(String host, int port) {
            this.host = host;
            this.port = port;
        }

        public String getHost() { return host; }
        public int getPort() { return port; }
    }

    /** Database query result item. */
    public static class Item {
        private final String key;
        private final String value;

        public Item(String key, String value) {
            this.key = key;
            this.value = value;
        }

        public String getKey() { return key; }
        public String getValue() { return value; }
    }

    /** Result of parsing operations (new in v2). */
    public static class ParseResult {
        private final String output;
        private final List<String> warnings;

        public ParseResult(String output, List<String> warnings) {
            this.output = output;
            this.warnings = warnings;
        }

        public String getOutput() { return output; }
        public List<String> getWarnings() { return warnings; }
    }

    /** Fetch a user by their ID (renamed from getUser). */
    public static Optional<UserAccount> fetchUser(long id) {
        if (id > 0) {
            return Optional.of(new UserAccount(id, "User" + id, "user" + id + "@example.com"));
        }
        return Optional.empty();
    }

    /** Transform raw data bytes (renamed from processData). */
    public static byte[] transformData(byte[] data) {
        byte[] result = new byte[data.length];
        for (int i = 0; i < data.length; i++) {
            result[i] = (byte) (data[i] + 1);
        }
        return result;
    }

    /** Connect to a host with port and timeout (signature changed). */
    public static ConnectionStatus connect(String host, int port, Duration timeout) throws Exception {
        if (host == null || host.isEmpty()) {
            throw new Exception("empty host");
        }
        if (port <= 0) {
            throw new Exception("invalid port");
        }
        return ConnectionStatus.CONNECTED;
    }

    /** Save data (sync parameter removed). */
    public static void save(String data) throws Exception {
        System.out.println("Saved: " + data);
    }

    /** Query with reordered parameters (c, a, b instead of a, b, c). */
    public static List<Item> query(boolean c, String a, int b) {
        List<Item> results = new ArrayList<>();
        if (c) {
            results.add(new Item(a, String.valueOf(b)));
        }
        return results;
    }

    /** Find an item by key (now returns Optional<Item>). */
    public static Optional<Item> find(String key) {
        if (key == null || key.isEmpty()) {
            return Optional.empty();
        }
        return Optional.of(new Item(key, "found"));
    }

    /** Parse a string into a ParseResult (return type changed). */
    public static ParseResult parse(String input) {
        if (input == null || input.isEmpty()) {
            List<String> warnings = new ArrayList<>();
            warnings.add("empty input");
            return new ParseResult("", warnings);
        }
        return new ParseResult(input.toUpperCase(), new ArrayList<>());
    }

    // Note: deprecatedFn has been removed in v2

    /** Helper functions (renamed from Utils). */
    public static class Helpers {
        /** A helper function that returns a greeting. */
        public static String helper() {
            return "Hello from helpers";
        }

        /** Format a user account for display. */
        public static String formatUser(UserAccount user) {
            return "UserAccount(id=" + user.getId() + ", name=" + user.getName() + ", email=" + user.getEmail() + ")";
        }

        /** Create a config map. */
        public static Map<String, String> createConfigMap() {
            Map<String, String> map = new HashMap<>();
            map.put("version", "2.0.0");
            return map;
        }
    }
}
