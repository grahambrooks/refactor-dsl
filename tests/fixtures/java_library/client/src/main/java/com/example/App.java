package com.example;

import java.util.List;
import java.util.Optional;

/**
 * Client application using MyLib v1.0.0
 *
 * This client needs to be upgraded to work with MyLib v2.0.0.
 * The upgrade analyzer should detect all the necessary changes.
 */
public class App {

    // Simulated types (from v1)
    static class User {
        long id;
        String name;
        User(long id, String name) {
            this.id = id;
            this.name = name;
        }
    }

    enum Status { CONNECTED, DISCONNECTED }

    static class Config {
        String host;
        Config(String host) { this.host = host; }
    }

    static class Item {
        String key;
        String value;
        Item(String key, String value) {
            this.key = key;
            this.value = value;
        }
    }

    // Simulated library functions
    static Optional<User> getUser(long id) {
        return Optional.of(new User(id, "User" + id));
    }

    static byte[] processData(byte[] data) {
        byte[] result = new byte[data.length];
        for (int i = 0; i < data.length; i++) {
            result[i] = (byte) (data[i] + 1);
        }
        return result;
    }

    static Status connect(String host) {
        return Status.CONNECTED;
    }

    static void save(String data, boolean sync) {
        System.out.println("Saved: " + data + ", sync=" + sync);
    }

    static List<Item> query(String a, int b, boolean c) {
        return c ? List.of(new Item(a, String.valueOf(b))) : List.of();
    }

    static Item find(String key) {
        return new Item(key, "found");
    }

    static String parse(String input) {
        return input.toUpperCase();
    }

    @SuppressWarnings("deprecation")
    static void deprecatedFn() {
        System.out.println("deprecated");
    }

    static class Utils {
        static String helper() { return "helper"; }
    }

    public static void main(String[] args) {
        System.out.println("=== Client Application ===\n");

        // Using getUser (should become fetchUser)
        Optional<User> user = getUser(42);
        user.ifPresent(u -> System.out.println("Found user: " + u.name));

        // Using another getUser call
        Optional<User> admin = getUser(1);
        admin.ifPresent(u -> System.out.println("Admin: " + u.name));

        // Using processData (should become transformData)
        byte[] data = {1, 2, 3, 4, 5};
        byte[] processed = processData(data);
        System.out.println("Processed data length: " + processed.length);

        // Using connect (needs port and timeout parameters)
        Config config = new Config("localhost");
        Status status = connect(config.host);
        System.out.println("Connected: " + status);

        // Using save (sync parameter should be removed)
        save("important data", true);
        save("more data", false);

        // Using query (parameters should be reordered)
        List<Item> results = query("search", 10, true);
        System.out.println("Query results: " + results.size() + " items");

        // Another query call
        List<Item> moreResults = query("filter", 5, false);
        System.out.println("More results: " + moreResults.size() + " items");

        // Using find (should return Optional<Item> now)
        Item item = find("my_key");
        System.out.println("Found item: " + item.key);

        // Using parse (return type changed)
        String result = parse("hello world");
        System.out.println("Parsed: " + result);

        // Using deprecatedFn (should be removed)
        deprecatedFn();

        // Using Utils class (should become Helpers)
        String greeting = Utils.helper();
        System.out.println("Greeting: " + greeting);

        System.out.println("\n=== Done ===");
    }
}

/** A service that uses the library. */
class UserService {
    private App.Config config;

    public UserService(String host) {
        this.config = new App.Config(host);
    }

    public Optional<App.User> getUserById(long id) {
        // Uses getUser internally
        return App.getUser(id);
    }

    public byte[] processUserData(byte[] data) {
        // Uses processData internally
        return App.processData(data);
    }

    public App.Status connectToServer() {
        // Uses connect internally
        return App.connect(config.host);
    }

    public void saveUser(App.User user) {
        // Uses save internally
        String data = user.id + ":" + user.name;
        App.save(data, true);
    }

    public List<App.Item> searchUsers(String queryStr, int limit) {
        // Uses query internally
        return App.query(queryStr, limit, true);
    }

    public App.Item findUserItem(String key) {
        // Uses find internally
        return App.find(key);
    }

    public String getHelperMessage() {
        // Uses Utils.helper internally
        return App.Utils.helper();
    }
}
