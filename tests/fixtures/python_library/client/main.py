"""
Client application using mylib v1.0.0

This client needs to be upgraded to work with mylib v2.0.0.
The upgrade analyzer should detect all the necessary changes.
"""

from dataclasses import dataclass
from typing import Optional, List
from enum import Enum


# Simulated types (from v1)
@dataclass
class User:
    id: int
    name: str


class Status(Enum):
    CONNECTED = "connected"
    DISCONNECTED = "disconnected"


@dataclass
class Config:
    host: str


@dataclass
class Item:
    key: str
    value: str


# Simulated library functions
def get_user(id: int) -> Optional[User]:
    return User(id=id, name=f"User{id}") if id > 0 else None


def process_data(data: bytes) -> bytes:
    return bytes(b + 1 for b in data)


def connect(host: str) -> Status:
    return Status.CONNECTED


def save(data: str, sync: bool) -> None:
    print(f"Saved: {data}, sync={sync}")


def query(a: str, b: int, c: bool) -> List[Item]:
    return [Item(key=a, value=str(b))] if c else []


def find(key: str) -> Item:
    return Item(key=key, value="found")


def parse(input: str) -> str:
    return input.upper()


def deprecated_fn() -> None:
    print("deprecated")


class Utils:
    @staticmethod
    def helper() -> str:
        return "helper"


def main():
    print("=== Client Application ===\n")

    # Using get_user (should become fetch_user)
    user: Optional[User] = get_user(42)
    if user:
        print(f"Found user: {user}")

    # Using another get_user call
    admin = get_user(1)
    if admin:
        print(f"Admin: {admin}")

    # Using process_data (should become transform_data)
    data = bytes([1, 2, 3, 4, 5])
    processed = process_data(data)
    print(f"Processed data: {list(processed)}")

    # Using connect (needs port and timeout parameters)
    config = Config(host="localhost")
    status: Status = connect(config.host)
    print(f"Connected: {status}")

    # Using save (sync parameter should be removed)
    save("important data", True)
    save("more data", False)

    # Using query (parameters should be reordered)
    results = query("search", 10, True)
    print(f"Query results: {len(results)} items")

    # Another query call
    more_results = query("filter", 5, False)
    print(f"More results: {len(more_results)} items")

    # Using find (should return Optional[Item] now)
    item: Item = find("my_key")
    print(f"Found item: {item.key}")

    # Using parse (return type changed)
    result: str = parse("hello world")
    print(f"Parsed: {result}")

    # Using deprecated_fn (should be removed)
    deprecated_fn()

    # Using Utils class (should become Helpers)
    greeting = Utils.helper()
    print(f"Greeting: {greeting}")

    print("\n=== Done ===")


class UserService:
    """A service that uses the library."""

    def __init__(self, host: str):
        self.config = Config(host=host)

    def get_user_by_id(self, id: int) -> Optional[User]:
        # Uses get_user internally
        return get_user(id)

    def process_user_data(self, data: bytes) -> bytes:
        # Uses process_data internally
        return process_data(data)

    def connect_to_server(self) -> Status:
        # Uses connect internally
        return connect(self.config.host)

    def save_user(self, user: User) -> None:
        # Uses save internally
        data = f"{user.id}:{user.name}"
        save(data, True)

    def search_users(self, query_str: str, limit: int) -> List[Item]:
        # Uses query internally
        return query(query_str, limit, True)

    def find_user_item(self, key: str) -> Item:
        # Uses find internally
        return find(key)

    def get_helper_message(self) -> str:
        # Uses Utils.helper internally
        return Utils.helper()


if __name__ == "__main__":
    main()
