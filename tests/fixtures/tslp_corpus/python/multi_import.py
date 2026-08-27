"""Comma-list imports — one statement, several names.

Hand-counted for the corpus gate. This shape is the reason `imports_in`
returns a list: tree-sitter-python attaches the `name` field to EVERY entry
below, so an extractor that asks for the field once records `os` and silently
drops `sys`.
"""

import os, sys
import json as encoder, csv
from collections import OrderedDict, defaultdict


def uses() -> tuple:
    return (os, sys, encoder, csv, OrderedDict, defaultdict)
