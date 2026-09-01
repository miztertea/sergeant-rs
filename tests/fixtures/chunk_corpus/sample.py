# A small synthetic Python fixture for the chunker corpus. ASCII-only.

import json
from dataclasses import dataclass, field


@dataclass
class Entry:
    key: str
    value: int
    tag: str | None = None


class Ledger:
    """A tiny in-memory ledger, only for corpus shape."""

    def __init__(self):
        self.entries: dict[str, Entry] = {}
        self.total = 0

    def insert(self, entry: Entry) -> None:
        if entry.key in self.entries:
            raise KeyError(f"duplicate key: {entry.key}")
        self.entries[entry.key] = entry
        self.total += entry.value

    def remove(self, key: str) -> Entry:
        if key not in self.entries:
            raise KeyError(f"no such entry: {key}")
        entry = self.entries.pop(key)
        self.total -= entry.value
        return entry

    def tagged(self, tag: str) -> list[Entry]:
        return [entry for entry in self.entries.values() if entry.tag == tag]

    def to_json(self) -> str:
        return json.dumps(
            {key: entry.value for key, entry in self.entries.items()}
        )


def describe(error: KeyError) -> str:
    return f"ledger error: {error}"


def load_from_dict(data: dict) -> Ledger:
    ledger = Ledger()
    for key, value in data.items():
        ledger.insert(Entry(key=key, value=value))
    return ledger


if __name__ == "__main__":
    ledger = Ledger()
    ledger.insert(Entry(key="a", value=10))
    ledger.insert(Entry(key="b", value=20, tag="important"))
    print(ledger.to_json())
