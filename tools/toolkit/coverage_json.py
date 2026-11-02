"""Stable coverage ledgers: one declaration per line, indented surrounding indexes."""

import json


def dumps(value, *, sort_keys=False):
    def render(value, depth=0):
        pad = "  " * depth
        if isinstance(value, dict):
            if "name" in value and ("signature" in value or "spelling" in value):
                return json.dumps(value, sort_keys=sort_keys)
            if not value:
                return "{}"
            keys = sorted(value) if sort_keys else value
            return "{\n" + ",\n".join(
                "  " * (depth + 1) + json.dumps(key) + ": " + render(value[key], depth + 1)
                for key in keys
            ) + "\n" + pad + "}"
        if isinstance(value, list):
            if not value:
                return "[]"
            return "[\n" + ",\n".join(
                "  " * (depth + 1) + render(item, depth + 1) for item in value
            ) + "\n" + pad + "]"
        return json.dumps(value)

    return render(value) + "\n"
