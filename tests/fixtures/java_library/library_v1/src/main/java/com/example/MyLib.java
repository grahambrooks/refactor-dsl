package com.example;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * MyLib v1.0.0 - A sample Java library for demonstrating API upgrades.
 */
public class MyLib {

    /** A user in the system. */
    public static class User {
        private final long id;
        private final String name;

        public User(long id, String name) {
            this.id = id;
            this.name = name;
        }

        public long getId() { return id; }
        public String getName() { return name; }

        @Override
        public String toString() {
            return "User{id=" + id + ", name='" + name + "'}";
        }
    }

    /** Connection status. */
    public enum Status {
        CONNECTED,
        DISCONNECTED,
        ERROR
    }

    /** Configuration for the library. */
    public static class Config {
        private final String host;

        public Config(String host) {
            this.host = host;
        }

        public String getHost() { return host; }
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

    /** Get a user by their ID. */
    public static Optional<User> getUser(long id) {
        if (id > 0) {
            return Optional.of(new User(id, "User" + id));
        }
        return Optional.empty();
    }

    /** Process raw data bytes. */
    public static byte[] processData(byte[] data) {
        byte[] result = new byte[data.length];
        for (int i = 0; i < data.length; i++) {
            result[i] = (byte) (data[i] + 1);
        }
        return result;
    }

    /** Connect to a host. */
    public static Status connect(String host) throws Exception {
        if (host == null || host.isEmpty()) {
            throw new Exception("empty host");
        }
        return Status.CONNECTED;
    }

    /** Save data with optional sync. */
    public static void save(String data, boolean sync) throws Exception {
        if (sync) {
            // Force sync
        }
        System.out.println("Saved: " + data);
    }

    /** Query with multiple parameters. */
    public static List<Item> query(String a, int b, boolean c) {
        List<Item> results = new ArrayList<>();
        if (c) {
            results.add(new Item(a, String.valueOf(b)));
        }
        return results;
    }

    /** Find an item by key. */
    public static Item find(String key) {
        return new Item(key, "found");
    }

    /** Parse a string. */
    public static String parse(String input) throws Exception {
        if (input == null || input.isEmpty()) {
            throw new Exception("empty input");
        }
        return input.toUpperCase();
    }

    /** @deprecated Use fetchUser instead */
    @Deprecated
    public static void deprecatedFn() {
        System.out.println("This function is deprecated");
    }

    /** Utility functions. */
    public static class Utils {
        /** A helper function that returns a greeting. */
        public static String helper() {
            return "Hello from utils";
        }

        /** Format a user for display. */
        public static String formatUser(User user) {
            return "User(id=" + user.getId() + ", name=" + user.getName() + ")";
        }

        /** Create a config map. */
        public static Map<String, String> createConfigMap() {
            Map<String, String> map = new HashMap<>();
            map.put("version", "1.0.0");
            return map;
        }
    }
}
