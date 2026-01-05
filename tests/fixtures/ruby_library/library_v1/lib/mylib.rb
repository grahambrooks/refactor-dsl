# MyLib v1.0.0 - A sample Ruby library for demonstrating API upgrades.

module MyLib
  # A user in the system.
  class User
    attr_reader :id, :name

    def initialize(id, name)
      @id = id
      @name = name
    end

    def to_s
      "User{id=#{@id}, name='#{@name}'}"
    end
  end

  # Connection status.
  module Status
    CONNECTED = :connected
    DISCONNECTED = :disconnected
    ERROR = :error
  end

  # Configuration for the library.
  class Config
    attr_reader :host

    def initialize(host)
      @host = host
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

  class << self
    # Get a user by their ID.
    def get_user(id)
      return nil if id <= 0
      User.new(id, "User#{id}")
    end

    # Process raw data bytes.
    def process_data(data)
      data.bytes.map { |b| (b + 1) % 256 }
    end

    # Connect to a host.
    def connect(host)
      raise ArgumentError, "empty host" if host.nil? || host.empty?
      Status::CONNECTED
    end

    # Save data with optional sync.
    def save(data, sync)
      # Force sync if requested
      puts "Saved: #{data}"
    end

    # Query with multiple parameters.
    def query(a, b, c)
      results = []
      results << Item.new(a, b.to_s) if c
      results
    end

    # Find an item by key.
    def find(key)
      Item.new(key, "found")
    end

    # Parse a string.
    def parse(input)
      raise ArgumentError, "empty input" if input.nil? || input.empty?
      input.upcase
    end

    # @deprecated Use fetch_user instead
    def deprecated_fn
      warn "This function is deprecated"
    end
  end

  # Utility functions.
  module Utils
    class << self
      # A helper function that returns a greeting.
      def helper
        "Hello from utils"
      end

      # Format a user for display.
      def format_user(user)
        "User(id=#{user.id}, name=#{user.name})"
      end

      # Create a config map.
      def create_config_map
        { "version" => "1.0.0" }
      end
    end
  end
end
