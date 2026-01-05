# Client application using MyLib v1.0.0
#
# This client needs to be upgraded to work with MyLib v2.0.0.
# The upgrade analyzer should detect all the necessary changes.

# Simulated types (from v1)
class User
  attr_reader :id, :name

  def initialize(id, name)
    @id = id
    @name = name
  end
end

module Status
  CONNECTED = :connected
  DISCONNECTED = :disconnected
end

class Config
  attr_reader :host

  def initialize(host)
    @host = host
  end
end

class Item
  attr_reader :key, :value

  def initialize(key, value)
    @key = key
    @value = value
  end
end

# Simulated library functions
def get_user(id)
  User.new(id, "User#{id}")
end

def process_data(data)
  data.bytes.map { |b| (b + 1) % 256 }
end

def connect(host)
  Status::CONNECTED
end

def save(data, sync)
  puts "Saved: #{data}, sync=#{sync}"
end

def query(a, b, c)
  c ? [Item.new(a, b.to_s)] : []
end

def find(key)
  Item.new(key, "found")
end

def parse(input)
  input.upcase
end

def deprecated_fn
  puts "deprecated"
end

module Utils
  def self.helper
    "helper"
  end
end

def main
  puts "=== Client Application ===\n"

  # Using get_user (should become fetch_user)
  user = get_user(42)
  puts "Found user: #{user.name}"

  # Using another get_user call
  admin = get_user(1)
  puts "Admin: #{admin.name}"

  # Using process_data (should become transform_data)
  data = [1, 2, 3, 4, 5].pack("C*")
  processed = process_data(data)
  puts "Processed data: #{processed}"

  # Using connect (needs port and timeout parameters)
  config = Config.new("localhost")
  status = connect(config.host)
  puts "Connected: #{status}"

  # Using save (sync parameter should be removed)
  save("important data", true)
  save("more data", false)

  # Using query (parameters should be reordered)
  results = query("search", 10, true)
  puts "Query results: #{results.length} items"

  # Another query call
  more_results = query("filter", 5, false)
  puts "More results: #{more_results.length} items"

  # Using find (should return nil for empty keys now)
  item = find("my_key")
  puts "Found item: #{item.key}"

  # Using parse (return type changed)
  result = parse("hello world")
  puts "Parsed: #{result}"

  # Using deprecated_fn (should be removed)
  deprecated_fn

  # Using Utils module (should become Helpers)
  greeting = Utils.helper
  puts "Greeting: #{greeting}"

  puts "\n=== Done ==="
end

# A service that uses the library
class UserService
  def initialize(host)
    @config = Config.new(host)
  end

  def get_user_by_id(id)
    # Uses get_user internally
    get_user(id)
  end

  def process_user_data(data)
    # Uses process_data internally
    process_data(data)
  end

  def connect_to_server
    # Uses connect internally
    connect(@config.host)
  end

  def save_user(user)
    # Uses save internally
    data = "#{user.id}:#{user.name}"
    save(data, true)
  end

  def search_users(query_str, limit)
    # Uses query internally
    query(query_str, limit, true)
  end

  def find_user_item(key)
    # Uses find internally
    find(key)
  end

  def get_helper_message
    # Uses Utils.helper internally
    Utils.helper
  end
end

main if __FILE__ == $PROGRAM_NAME
