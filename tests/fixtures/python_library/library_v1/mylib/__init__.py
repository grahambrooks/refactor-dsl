"""
MyLib v1.0.0 - A sample Python library for demonstrating API upgrades.
"""

from dataclasses import dataclass
from typing import Optional, List, Dict
from enum import Enum


@dataclass
class User:
    """A user in the system."""
    id: int
    name: str


class Status(Enum):
    """Connection status."""
    CONNECTED = "connected"
    DISCONNECTED = "disconnected"
    ERROR = "error"


@dataclass
class Config:
    """Configuration for the library."""
    host: str


@dataclass
class Item:
    """Database query result item."""
    key: str
    value: str


def get_user(id: int) -> Optional[User]:
    """Get a user by their ID."""
    if id > 0:
        return User(id=id, name=f"User{id}")
    return None


def process_data(data: bytes) -> bytes:
    """Process raw data bytes."""
    return bytes(b + 1 for b in data)


def connect(host: str) -> Status:
    """Connect to a host."""
    if not host:
        raise ValueError("empty host")
    return Status.CONNECTED


def save(data: str, sync: bool) -> None:
    """Save data with optional sync."""
    if sync:
        pass  # Force sync
    print(f"Saved: {data}")


def query(a: str, b: int, c: bool) -> List[Item]:
    """Query with multiple parameters."""
    results = []
    if c:
        results.append(Item(key=a, value=str(b)))
    return results


def find(key: str) -> Item:
    """Find an item by key."""
    return Item(key=key, value="found")


def parse(input: str) -> str:
    """Parse a string."""
    if not input:
        raise ValueError("empty input")
    return input.upper()


def deprecated_fn() -> None:
    """This function is deprecated and will be removed."""
    print("This function is deprecated")


class Utils:
    """Utility functions."""

    @staticmethod
    def helper() -> str:
        """A helper function that returns a greeting."""
        return "Hello from utils"

    @staticmethod
    def format_user(user: User) -> str:
        """Format a user for display."""
        return f"User(id={user.id}, name={user.name})"

    @staticmethod
    def create_config_map() -> Dict[str, str]:
        """Create a config map."""
        return {"version": "1.0.0"}
