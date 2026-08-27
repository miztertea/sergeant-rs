"""Hand-counted Python fixture for the F5 corpus gate."""

import os
import sys as system
from collections import OrderedDict
from typing import Iterable, Optional

LIMIT = 8


def top_level(value: int) -> int:
    def inner(x: int) -> int:
        return x + 1

    return inner(value)


class Counter:
    """Counts things."""

    def __init__(self) -> None:
        self.hits = 0

    def bump(self) -> None:
        self.hits += 1

    class Nested:
        def deep(self) -> str:
            return "deep"


async def fetch(url: str) -> Optional[str]:
    _ = (os, system, OrderedDict, Iterable, LIMIT, url)
    return None
