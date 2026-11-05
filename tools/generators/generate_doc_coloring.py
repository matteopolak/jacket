#!/usr/bin/env python3
"""Generate browser coloring from the producer's declarative spec, with no installed npm dependency.

    PYTHONPATH=tools python3 tools/generators/generate_doc_coloring.py --check
"""
import argparse
import hashlib
import json
from toolkit.repo import ROOT

OUTPUTS = ["website/docs/src/generated/doc-coloring.ts"]
KINDS = ["line_comment", "nested_comment", "quoted", "note", "directive", "range", "number", "identifier", "plain"]


def generate(check=False, *, root=ROOT):
    """Fail closed on unsupported operations; --check compares without writing any output."""
    source = (root / "crates/jacket/src/doc/coloring/spec.json").read_bytes()
    spec = json.loads(source)
    if spec.get("schema") != "jai-doc-coloring/1" or [rule["kind"] for rule in spec["rules"]] != KINDS:
        raise ValueError("unsupported coloring schema or ordered operation inventory")
    if set(spec) != {"schema", "identifier", "keywords", "types", "rules"}:
        raise ValueError("unknown coloring spec field")
    if spec["identifier"] != {"nonAscii": True, "excluded": ["\u00a0", "\ufeff"]}:
        raise ValueError("unsupported identifier policy")
    fields = {
        "line_comment": {"kind", "open", "role"},
        "nested_comment": {"kind", "open", "close", "role"},
        "quoted": {"kind", "open", "escape", "role"},
        "note": {"kind", "open", "lineBounded", "paint", "semantic"},
        "directive": {"kind", "open", "horizontal", "hereName", "hereRole", "role"},
        "range": {"kind", "open"}, "number": {"kind", "role", "radixPrefixes"},
        "identifier": {"kind"}, "plain": {"kind"},
    }
    for rule in spec["rules"]:
        if set(rule) != fields[rule["kind"]]:
            raise ValueError("missing or unknown coloring operation field")
        for key in ["open", "close", "escape"]:
            if key in rule and (not isinstance(rule[key], str) or not rule[key] or not rule[key].isascii()):
                raise ValueError("coloring delimiters must be nonempty ASCII strings")
        if rule["kind"] in ["quoted", "note", "directive"] and len(rule["open"]) != 1:
            raise ValueError("quoted/note/directive openers must be single characters")
        if rule["kind"] == "quoted" and (len(rule["escape"]) != 1 or rule["escape"] == rule["open"]):
            raise ValueError("quoted escape must be one distinct character")
        if rule["kind"] == "nested_comment" and rule["open"] == rule["close"]:
            raise ValueError("nested comment delimiters must differ")
        if rule["kind"] == "note" and (rule["lineBounded"] is not True or rule["paint"] != "lexical" or rule["semantic"] is not False):
            raise ValueError("notes require line-bounded lexical paint without semantic names")
        if rule["kind"] == "directive" and (rule["horizontal"] != " \t" or rule["hereName"] != "string" or rule["hereRole"] != "s"):
            raise ValueError("unsupported here-string header policy")
        if rule["kind"] == "number" and rule["radixPrefixes"] != ["0x", "0b", "0h"]:
            raise ValueError("unsupported numeric prefix policy")
    if any(rule.get("role") not in [None, "k", "t", "s", "n", "c", "d"] for rule in spec["rules"]):
        raise ValueError("unsupported coloring role")
    for key in ["keywords", "types"]:
        if len(spec[key]) != len(set(spec[key])) or any(not isinstance(word, str) or not word for word in spec[key]):
            raise ValueError("coloring word lists must contain unique nonempty strings")
    if set(spec["keywords"]) & set(spec["types"]):
        raise ValueError("coloring keyword/type sets overlap")
    template = (root / "tools/generators/templates/doc-coloring.ts").read_text()
    output = template.replace("__COLORING_SPEC__", json.dumps(spec, ensure_ascii=True, separators=(",", ":")))
    output = "// Spec SHA-256: " + hashlib.sha256(source).hexdigest() + "\n" + output
    path = root / OUTPUTS[0]
    if check:
        if not path.is_file() or path.read_text() != output:
            raise ValueError("documentation coloring is stale; run generate_doc_coloring.py")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(output)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    generate(parser.parse_args().check)
