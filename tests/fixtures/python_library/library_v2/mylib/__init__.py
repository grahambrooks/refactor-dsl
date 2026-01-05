"""
MyLib v2.0.0 - A sample Python library for demonstrating API upgrades.

Breaking changes from v1.0.0:
- `User` renamed to `UserAccount` with new `email` field
- `Status` renamed to `ConnectionStatus`
- `get_user` renamed to `fetch_user`
- `process_data` renamed to `transform_data`
- `connect` now requires `port` and `timeout` parameters
- `save` no longer takes `sync` parameter
- `query` parameters reordered from (a, b, c) to (c, a, b)
- `find` now returns `Optional[Item]` instead of `Item`
- `parse` now returns `ParseResult` instead of `str`
- `deprecated_fn` has been removed
- `Utils` class renamed to `Helpers`
"""

from dataclasses import dataclass
from typing import Optional, List, Dict
from enum import Enum


@dataclass
class UserAccount:
    """A user account in the system (renamed from User)."""
    id: int
    name: str
    email: str  # New field in v2


class ConnectionStatus(Enum):
    """Connection status (renamed from Status)."""
    CONNECTED = "connected"
    DISCONNECTED = "disconnected"
    ERROR = "error"


@dataclass
class Config:
    """Configuration for the library (with new port field)."""
    host: str
    port: int  # New field in v2


@dataclass
class Item:
    """Database query result item."""
    key: str
    value: str


@dataclass
class ParseResult:
    """Result of parsing operations (new in v2)."""
    output: str
    warnings: List[str]


def fetch_user(id: int) -> Optional[UserAccount]:
    """Fetch a user by their ID (renamed from get_user)."""
    if id > 0:
        return UserAccount(id=id, name=f"User{id}", email=f"user{id}@example.com")
    return None


def transform_data(data: bytes) -> bytes:
    """Transform raw data bytes (renamed from process_data)."""
    return bytes(b + 1 for b in data)


def connect(host: str, port: int, timeout: float) -> ConnectionStatus:
    """Connect to a host with port and timeout (signature changed)."""
    if not host:
        raise ValueError("empty host")
    if port <= 0:
        raise ValueError("invalid port")
    return ConnectionStatus.CONNECTED


def save(data: str) -> None:
    """Save data (sync parameter removed)."""
    print(f"Saved: {data}")


def query(c: bool, a: str, b: int) -> List[Item]:
    """Query with reordered parameters (c, a, b instead of a, b, c)."""
    results = []
    if c:
        results.append(Item(key=a, value=str(b)))
    return results


def find(key: str) -> Optional[Item]:
    """Find an item by key (now returns Optional[Item])."""
    if not key:
        return None
    return Item(key=key, value="found")


def parse(input: str) -> ParseResult:
    """Parse a string into a ParseResult (return type changed)."""
    if not input:
        return ParseResult(output="", warnings=["empty input"])
    return ParseResult(output=input.upper(), warnings=[])


# Note: deprecated_fn has been removed in v2


class Helpers:
    """Helper functions (renamed from Utils)."""

    @staticmethod
    def helper() -> str:
        """A helper function that returns a greeting."""
        return "Hello from helpers"

    @staticmethod
    def format_user(user: UserAccount) -> str:
        """Format a user account for display."""
        return f"UserAccount(id={user.id}, name={user.name}, email={user.email})"

    @staticmethod
    def create_config_map() -> Dict[str, str]:
        """Create a config map."""
        return {"version": "2.0.0"}
