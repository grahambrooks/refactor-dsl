# MyLib v2.0.0 - A sample Ruby library for demonstrating API upgrades.
#
# Breaking changes from v1.0.0:
# - User renamed to UserAccount with new email attribute
# - Status renamed to ConnectionStatus
# - get_user renamed to fetch_user
# - process_data renamed to transform_data
# - connect now requires port and timeout parameters
# - save no longer takes sync parameter
# - query parameters reordered from (a, b, c) to (c, a, b)
# - find now returns nil instead of raising for empty keys
# - parse now returns ParseResult instead of string
# - deprecated_fn has been removed
# - Utils renamed to Helpers

module MyLib
  # A user account in the system (renamed from User).
  class UserAccount
    attr_reader :id, :name, :email

    def initialize(id, name, email)
      @id = id
      @name = name
      @email = email
    end

    def to_s
      "UserAccount{id=#{@id}, name='#{@name}', email='#{@email}'}"
    end
  end

  # Connection status (renamed from Status).
  module ConnectionStatus
    CONNECTED = :connected
    DISCONNECTED = :disconnected
    ERROR = :error
  end

  # Configuration for the library (with new port attribute).
  class Config
    attr_reader :host, :port

    def initialize(host, port)
      @host = host
      @port = port
    end
  end

  # Database query result item.
  class Item
    attr_reader :key, :value

    def initialize(key, value)
      @key = key
      @value = value
    end
  end

  # Result of parsing operations (new in v2).
  class ParseResult
    attr_reader :output, :warnings

    def initialize(output, warnings = [])
      @output = output
      @warnings = warnings
    end
  end

  class << self
    # Fetch a user by their ID (renamed from get_user).
    def fetch_user(id)
      return nil if id <= 0
      UserAccount.new(id, "User#{id}", "user#{id}@example.com")
    end

    # Transform raw data bytes (renamed from process_data).
    def transform_data(data)
      data.bytes.map { |b| (b + 1) % 256 }
    end

    # Connect to a host with port and timeout (signature changed).
    def connect(host, port, timeout)
      raise ArgumentError, "empty host" if host.nil? || host.empty?
      raise ArgumentError, "invalid port" if port <= 0
      ConnectionStatus::CONNECTED
    end

    # Save data (sync parameter removed).
    def save(data)
      puts "Saved: #{data}"
    end

    # Query with reordered parameters (c, a, b instead of a, b, c).
    def query(c, a, b)
      results = []
      results << Item.new(a, b.to_s) if c
      results
    end

    # Find an item by key (now returns nil for empty keys).
    def find(key)
      return nil if key.nil? || key.empty?
      Item.new(key, "found")
    end

    # Parse a string into a ParseResult (return type changed).
    def parse(input)
      if input.nil? || input.empty?
        ParseResult.new("", ["empty input"])
      else
        ParseResult.new(input.upcase)
      end
    end

    # Note: deprecated_fn has been removed in v2
  end

  # Helper functions (renamed from Utils).
  module Helpers
    class << self
      # A helper function that returns a greeting.
      def helper
        "Hello from helpers"
      end

      # Format a user account for display.
      def format_user(user)
        "UserAccount(id=#{user.id}, name=#{user.name}, email=#{user.email})"
      end

      # Create a config map.
      def create_config_map
        { "version" => "2.0.0" }
      end
    end
  end
end
