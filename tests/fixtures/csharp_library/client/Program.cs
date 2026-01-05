// Client application using MyLib v1.0.0
//
// This client needs to be upgraded to work with MyLib v2.0.0.
// The upgrade analyzer should detect all the necessary changes.

using System;
using System.Collections.Generic;

namespace Client
{
    // Simulated types (from v1)
    class User
    {
        public long Id { get; }
        public string Name { get; }
        public User(long id, string name) { Id = id; Name = name; }
    }

    enum Status { Connected, Disconnected }

    class Config
    {
        public string Host { get; }
        public Config(string host) { Host = host; }
    }

    class Item
    {
        public string Key { get; }
        public string Value { get; }
        public Item(string key, string value) { Key = key; Value = value; }
    }

    // Simulated library functions
    static class Library
    {
        public static User? GetUser(long id) => new User(id, $"User{id}");
        public static byte[] ProcessData(byte[] data)
        {
            var result = new byte[data.Length];
            for (int i = 0; i < data.Length; i++) result[i] = (byte)(data[i] + 1);
            return result;
        }
        public static Status Connect(string host) => Status.Connected;
        public static void Save(string data, bool sync) => Console.WriteLine($"Saved: {data}, sync={sync}");
        public static List<Item> Query(string a, int b, bool c) =>
            c ? new List<Item> { new Item(a, b.ToString()) } : new List<Item>();
        public static Item Find(string key) => new Item(key, "found");
        public static string Parse(string input) => input.ToUpper();
        [Obsolete] public static void DeprecatedFn() => Console.WriteLine("deprecated");
    }

    static class Utils
    {
        public static string Helper() => "helper";
    }

    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("=== Client Application ===\n");

            // Using GetUser (should become FetchUser)
            var user = Library.GetUser(42);
            if (user != null)
            {
                Console.WriteLine($"Found user: {user.Name}");
            }

            // Using another GetUser call
            var admin = Library.GetUser(1);
            if (admin != null)
            {
                Console.WriteLine($"Admin: {admin.Name}");
            }

            // Using ProcessData (should become TransformData)
            byte[] data = { 1, 2, 3, 4, 5 };
            var processed = Library.ProcessData(data);
            Console.WriteLine($"Processed data length: {processed.Length}");

            // Using Connect (needs port and timeout parameters)
            var config = new Config("localhost");
            var status = Library.Connect(config.Host);
            Console.WriteLine($"Connected: {status}");

            // Using Save (sync parameter should be removed)
            Library.Save("important data", true);
            Library.Save("more data", false);

            // Using Query (parameters should be reordered)
            var results = Library.Query("search", 10, true);
            Console.WriteLine($"Query results: {results.Count} items");

            // Another Query call
            var moreResults = Library.Query("filter", 5, false);
            Console.WriteLine($"More results: {moreResults.Count} items");

            // Using Find (should return Item? now)
            var item = Library.Find("my_key");
            Console.WriteLine($"Found item: {item.Key}");

            // Using Parse (return type changed)
            var result = Library.Parse("hello world");
            Console.WriteLine($"Parsed: {result}");

            // Using DeprecatedFn (should be removed)
            #pragma warning disable CS0618
            Library.DeprecatedFn();
            #pragma warning restore CS0618

            // Using Utils class (should become Helpers)
            var greeting = Utils.Helper();
            Console.WriteLine($"Greeting: {greeting}");

            Console.WriteLine("\n=== Done ===");
        }
    }

    /// <summary>A service that uses the library.</summary>
    class UserService
    {
        private Config _config;

        public UserService(string host)
        {
            _config = new Config(host);
        }

        public User? GetUserById(long id)
        {
            // Uses GetUser internally
            return Library.GetUser(id);
        }

        public byte[] ProcessUserData(byte[] data)
        {
            // Uses ProcessData internally
            return Library.ProcessData(data);
        }

        public Status ConnectToServer()
        {
            // Uses Connect internally
            return Library.Connect(_config.Host);
        }

        public void SaveUser(User user)
        {
            // Uses Save internally
            var data = $"{user.Id}:{user.Name}";
            Library.Save(data, true);
        }

        public List<Item> SearchUsers(string queryStr, int limit)
        {
            // Uses Query internally
            return Library.Query(queryStr, limit, true);
        }

        public Item FindUserItem(string key)
        {
            // Uses Find internally
            return Library.Find(key);
        }

        public string GetHelperMessage()
        {
            // Uses Utils.Helper internally
            return Utils.Helper();
        }
    }
}
