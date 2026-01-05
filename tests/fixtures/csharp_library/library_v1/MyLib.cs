// MyLib v1.0.0 - A sample C# library for demonstrating API upgrades.

using System;
using System.Collections.Generic;

namespace MyLib
{
    /// <summary>A user in the system.</summary>
    public class User
    {
        public long Id { get; }
        public string Name { get; }

        public User(long id, string name)
        {
            Id = id;
            Name = name;
        }

        public override string ToString() => $"User{{Id={Id}, Name='{Name}'}}";
    }

    /// <summary>Connection status.</summary>
    public enum Status
    {
        Connected,
        Disconnected,
        Error
    }

    /// <summary>Configuration for the library.</summary>
    public class Config
    {
        public string Host { get; }

        public Config(string host)
        {
            Host = host;
        }
    }

    /// <summary>Database query result item.</summary>
    public class Item
    {
        public string Key { get; }
        public string Value { get; }

        public Item(string key, string value)
        {
            Key = key;
            Value = value;
        }
    }

    /// <summary>Main library class.</summary>
    public static class Library
    {
        /// <summary>Get a user by their ID.</summary>
        public static User? GetUser(long id)
        {
            if (id > 0)
            {
                return new User(id, $"User{id}");
            }
            return null;
        }

        /// <summary>Process raw data bytes.</summary>
        public static byte[] ProcessData(byte[] data)
        {
            var result = new byte[data.Length];
            for (int i = 0; i < data.Length; i++)
            {
                result[i] = (byte)(data[i] + 1);
            }
            return result;
        }

        /// <summary>Connect to a host.</summary>
        public static Status Connect(string host)
        {
            if (string.IsNullOrEmpty(host))
            {
                throw new ArgumentException("empty host");
            }
            return Status.Connected;
        }

        /// <summary>Save data with optional sync.</summary>
        public static void Save(string data, bool sync)
        {
            if (sync)
            {
                // Force sync
            }
            Console.WriteLine($"Saved: {data}");
        }

        /// <summary>Query with multiple parameters.</summary>
        public static List<Item> Query(string a, int b, bool c)
        {
            var results = new List<Item>();
            if (c)
            {
                results.Add(new Item(a, b.ToString()));
            }
            return results;
        }

        /// <summary>Find an item by key.</summary>
        public static Item Find(string key)
        {
            return new Item(key, "found");
        }

        /// <summary>Parse a string.</summary>
        public static string Parse(string input)
        {
            if (string.IsNullOrEmpty(input))
            {
                throw new ArgumentException("empty input");
            }
            return input.ToUpper();
        }

        /// <summary>This function is deprecated and will be removed.</summary>
        [Obsolete("Use FetchUser instead")]
        public static void DeprecatedFn()
        {
            Console.WriteLine("This function is deprecated");
        }
    }

    /// <summary>Utility functions.</summary>
    public static class Utils
    {
        /// <summary>A helper function that returns a greeting.</summary>
        public static string Helper()
        {
            return "Hello from utils";
        }

        /// <summary>Format a user for display.</summary>
        public static string FormatUser(User user)
        {
            return $"User(id={user.Id}, name={user.Name})";
        }

        /// <summary>Create a config map.</summary>
        public static Dictionary<string, string> CreateConfigMap()
        {
            return new Dictionary<string, string> { { "version", "1.0.0" } };
        }
    }
}
