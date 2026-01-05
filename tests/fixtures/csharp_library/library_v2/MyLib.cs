// MyLib v2.0.0 - A sample C# library for demonstrating API upgrades.
//
// Breaking changes from v1.0.0:
// - User renamed to UserAccount with new Email property
// - Status renamed to ConnectionStatus
// - GetUser renamed to FetchUser
// - ProcessData renamed to TransformData
// - Connect now requires port and timeout parameters
// - Save no longer takes sync parameter
// - Query parameters reordered from (a, b, c) to (c, a, b)
// - Find now returns Item? instead of Item
// - Parse now returns ParseResult instead of string
// - DeprecatedFn has been removed
// - Utils renamed to Helpers

using System;
using System.Collections.Generic;

namespace MyLib
{
    /// <summary>A user account in the system (renamed from User).</summary>
    public class UserAccount
    {
        public long Id { get; }
        public string Name { get; }
        public string Email { get; } // New property in v2

        public UserAccount(long id, string name, string email)
        {
            Id = id;
            Name = name;
            Email = email;
        }

        public override string ToString() => $"UserAccount{{Id={Id}, Name='{Name}', Email='{Email}'}}";
    }

    /// <summary>Connection status (renamed from Status).</summary>
    public enum ConnectionStatus
    {
        Connected,
        Disconnected,
        Error
    }

    /// <summary>Configuration for the library (with new Port property).</summary>
    public class Config
    {
        public string Host { get; }
        public int Port { get; } // New property in v2

        public Config(string host, int port)
        {
            Host = host;
            Port = port;
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

    /// <summary>Result of parsing operations (new in v2).</summary>
    public class ParseResult
    {
        public string Output { get; }
        public List<string> Warnings { get; }

        public ParseResult(string output, List<string> warnings)
        {
            Output = output;
            Warnings = warnings;
        }
    }

    /// <summary>Main library class.</summary>
    public static class Library
    {
        /// <summary>Fetch a user by their ID (renamed from GetUser).</summary>
        public static UserAccount? FetchUser(long id)
        {
            if (id > 0)
            {
                return new UserAccount(id, $"User{id}", $"user{id}@example.com");
            }
            return null;
        }

        /// <summary>Transform raw data bytes (renamed from ProcessData).</summary>
        public static byte[] TransformData(byte[] data)
        {
            var result = new byte[data.Length];
            for (int i = 0; i < data.Length; i++)
            {
                result[i] = (byte)(data[i] + 1);
            }
            return result;
        }

        /// <summary>Connect to a host with port and timeout (signature changed).</summary>
        public static ConnectionStatus Connect(string host, int port, TimeSpan timeout)
        {
            if (string.IsNullOrEmpty(host))
            {
                throw new ArgumentException("empty host");
            }
            if (port <= 0)
            {
                throw new ArgumentException("invalid port");
            }
            return ConnectionStatus.Connected;
        }

        /// <summary>Save data (sync parameter removed).</summary>
        public static void Save(string data)
        {
            Console.WriteLine($"Saved: {data}");
        }

        /// <summary>Query with reordered parameters (c, a, b instead of a, b, c).</summary>
        public static List<Item> Query(bool c, string a, int b)
        {
            var results = new List<Item>();
            if (c)
            {
                results.Add(new Item(a, b.ToString()));
            }
            return results;
        }

        /// <summary>Find an item by key (now returns Item?).</summary>
        public static Item? Find(string key)
        {
            if (string.IsNullOrEmpty(key))
            {
                return null;
            }
            return new Item(key, "found");
        }

        /// <summary>Parse a string into a ParseResult (return type changed).</summary>
        public static ParseResult Parse(string input)
        {
            if (string.IsNullOrEmpty(input))
            {
                return new ParseResult("", new List<string> { "empty input" });
            }
            return new ParseResult(input.ToUpper(), new List<string>());
        }

        // Note: DeprecatedFn has been removed in v2
    }

    /// <summary>Helper functions (renamed from Utils).</summary>
    public static class Helpers
    {
        /// <summary>A helper function that returns a greeting.</summary>
        public static string Helper()
        {
            return "Hello from helpers";
        }

        /// <summary>Format a user account for display.</summary>
        public static string FormatUser(UserAccount user)
        {
            return $"UserAccount(id={user.Id}, name={user.Name}, email={user.Email})";
        }

        /// <summary>Create a config map.</summary>
        public static Dictionary<string, string> CreateConfigMap()
        {
            return new Dictionary<string, string> { { "version", "2.0.0" } };
        }
    }
}
