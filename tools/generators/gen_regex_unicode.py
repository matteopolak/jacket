#!/usr/bin/env python3
"""Generate stdlib/Extensions/Regex/unicode_data.jai from the Unicode Character Database."""
import argparse
import hashlib
import io
import os
from pathlib import Path
import sys
import unicodedata
import urllib.request
import zipfile

UCD_VERSION = "17.0.0"
UCD_URL = f"https://www.unicode.org/Public/{UCD_VERSION}/ucd/UCD.zip"
UCD_SHA256 = "2066d1909b2ea93916ce092da1c0ee4808ea3ef8407c94b4f14f5b7eb263d28e"
ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "stdlib" / "Extensions" / "Regex" / "unicode_data.jai"
MAX_CP = 0x10FFFF
LINE_WIDTH = 110

# Six bits per character: five payload bits (low group first) and a continuation bit.
ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"

BINARY_SOURCES = [
    "PropList.txt",
    "DerivedCoreProperties.txt",
    "emoji/emoji-data.txt",
    "DerivedNormalizationProps.txt",
    "extracted/DerivedBinaryProperties.txt",
]


class Source:
    """Reads UCD files from an extracted directory or from the zip."""

    def __init__(self, directory=None, archive=None):
        self.directory = directory
        self.archive = archive

    def read(self, name):
        if self.directory is not None:
            return (self.directory / name).read_text(encoding="utf-8")
        return self.archive.read(name).decode("utf-8")


def cache_path():
    base = os.environ.get("XDG_CACHE_HOME") or str(Path.home() / ".cache")
    return Path(base) / "jaic" / f"ucd-{UCD_VERSION}.zip"


def verified_zip(path):
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != UCD_SHA256:
        raise SystemExit(f"{path}: SHA-256 {digest} does not match the pinned {UCD_SHA256}")
    return zipfile.ZipFile(io.BytesIO(path.read_bytes()))


def open_source(args):
    if args.ucd:
        return Source(directory=Path(args.ucd))
    path = Path(args.zip) if args.zip else cache_path()
    if not path.exists():
        if args.zip:
            raise SystemExit(f"{path}: no such file")
        path.parent.mkdir(parents=True, exist_ok=True)
        print(f"downloading {UCD_URL}", file=sys.stderr)
        partial = path.with_suffix(".part")
        with urllib.request.urlopen(UCD_URL, timeout=120) as response:
            partial.write_bytes(response.read())
        partial.replace(path)
    return Source(archive=verified_zip(path))


def data_lines(text):
    """Yields the fields of every non-comment line, split on `;`."""
    for line in text.splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            yield [field.strip() for field in line.split(";")]


def parse_span(text):
    if ".." in text:
        lo, hi = text.split("..")
        return int(lo, 16), int(hi, 16)
    value = int(text, 16)
    return value, value


def normalize_ranges(ranges):
    out = []
    for lo, hi in sorted(ranges):
        if out and lo <= out[-1][1] + 1:
            if hi > out[-1][1]:
                out[-1] = (out[-1][0], hi)
        else:
            out.append((lo, hi))
    return out


def complement(ranges):
    out = []
    next_free = 0
    for lo, hi in ranges:
        if lo > next_free:
            out.append((next_free, lo - 1))
        next_free = hi + 1
    if next_free <= MAX_CP:
        out.append((next_free, MAX_CP))
    return out


def union(*sets):
    return normalize_ranges([r for s in sets for r in s])


def loose(name):
    """UAX44-LM3 without the symbolic exceptions: case, spaces, `_` and `-` do not matter."""
    return "".join(ch for ch in name.lower() if ch not in " _-")


def runs_of(labels):
    """Run-length encodes a per-code-point list into (lo, hi, label) triples."""
    start = 0
    for cp in range(1, len(labels) + 1):
        if cp == len(labels) or labels[cp] != labels[start]:
            yield start, cp - 1, labels[start]
            start = cp


def collect(by_label):
    return {label: normalize_ranges(spans) for label, spans in by_label.items()}


# ---------------------------------------------------------------- parsing the UCD


def read_general_category(source):
    labels = ["Cn"] * (MAX_CP + 1)
    first = None
    for fields in data_lines(source.read("UnicodeData.txt")):
        cp, name, gc = int(fields[0], 16), fields[1], fields[2]
        if name.endswith(", First>"):
            first = cp
            continue
        if name.endswith(", Last>"):
            for k in range(first, cp + 1):
                labels[k] = gc
            continue
        labels[cp] = gc
    return labels


def read_aliases(source):
    """Returns (property aliases by section, value aliases) from the two alias files."""
    properties = {}
    section = None
    for line in source.read("PropertyAliases.txt").splitlines():
        stripped = line.strip()
        if stripped.startswith("# ") and stripped.endswith(" Properties"):
            section = stripped[2:-len(" Properties")]
            continue
        body = line.split("#", 1)[0].strip()
        if body:
            names = [f.strip() for f in body.split(";")]
            properties.setdefault(section, []).append(names)
    values = {}
    for fields in data_lines(source.read("PropertyValueAliases.txt")):
        values.setdefault(fields[0], []).append(fields[1:])
    return properties, values


def read_scripts(source, long_by_short):
    labels = ["Unknown"] * (MAX_CP + 1)
    for fields in data_lines(source.read("Scripts.txt")):
        lo, hi = parse_span(fields[0])
        for cp in range(lo, hi + 1):
            labels[cp] = fields[1]
    extensions = [None] * (MAX_CP + 1)
    for fields in data_lines(source.read("ScriptExtensions.txt")):
        lo, hi = parse_span(fields[0])
        names = tuple(sorted(long_by_short[code] for code in fields[1].split()))
        for cp in range(lo, hi + 1):
            extensions[cp] = names
    return labels, extensions


def read_binary(source, binary_long_names):
    found = {}
    skipped = set()
    for name in BINARY_SOURCES:
        for fields in data_lines(source.read(name)):
            if len(fields) != 2:
                continue  # a property with a value, not a binary one
            prop = fields[1]
            if prop not in binary_long_names:
                skipped.add(prop)
                continue
            found.setdefault(prop, []).append(parse_span(fields[0]))
    return {prop: normalize_ranges(spans) for prop, spans in found.items()}, skipped


def read_folding(source):
    parent = {}

    def find(x):
        while parent.setdefault(x, x) != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for fields in data_lines(source.read("CaseFolding.txt")):
        if fields[1] in ("C", "S"):
            a, b = find(int(fields[0], 16)), find(int(fields[2], 16))
            if a != b:
                parent[max(a, b)] = min(a, b)
    groups = {}
    for x in list(parent):
        groups.setdefault(find(x), []).append(x)
    return sorted(sorted(members) for members in groups.values())


# ---------------------------------------------------------------- building the sets


def build_sets(source):
    properties, values = read_aliases(source)

    gc_labels = read_general_category(source)
    gc_sets = collect(_spans(runs_of(gc_labels)))

    gc_members = {}
    for aliases in values["gc"]:
        short = aliases[0]
        if len(short) == 1:
            gc_members[short] = [g for g in gc_sets if g[0] == short]
        elif short == "LC":
            gc_members[short] = ["Lu", "Ll", "Lt"]
    sets = {}  # (kind, canonical) -> ranges
    for aliases in values["gc"]:
        short = aliases[0]
        if short in gc_members:
            sets[("g", short)] = union(*(gc_sets.get(g, []) for g in gc_members[short]))
        else:
            sets[("g", short)] = gc_sets.get(short, [])
    names = {}  # (kind, normalized name) -> set key
    for aliases in values["gc"]:
        for name in aliases:
            names[("g", loose(name))] = ("g", aliases[0])

    long_by_short = {}
    for aliases in values["sc"]:
        for name in aliases:
            long_by_short[name] = aliases[1]
    sc_labels, scx_labels = read_scripts(source, long_by_short)
    sc_sets = collect(_spans(runs_of(sc_labels)))
    scx_spans = {}
    for cp in range(MAX_CP + 1):
        for script in scx_labels[cp] or (sc_labels[cp],):
            scx_spans.setdefault(script, []).append((cp, cp))
    scx_sets = collect(scx_spans)
    for aliases in values["sc"]:
        long = aliases[1]
        sets[("s", long)] = sc_sets.get(long, [])
        sets[("x", long)] = scx_sets.get(long, [])
        for name in aliases:
            names[("s", loose(name))] = ("s", long)
            names[("x", loose(name))] = ("x", long)

    binary_long = {aliases[1] for aliases in properties["Binary"]}
    binary, skipped = read_binary(source, binary_long)
    for aliases in properties["Binary"]:
        long = aliases[1]
        if long not in binary:
            continue
        sets[("b", long)] = binary[long]
        for name in aliases:
            names[("b", loose(name))] = ("b", long)

    sets[("m", "Any")] = [(0, MAX_CP)]
    sets[("m", "ASCII")] = [(0, 0x7F)]
    sets[("m", "Assigned")] = complement(sets[("g", "Cn")])
    for key in ("any", "ascii", "assigned"):
        names[("m", key)] = ("m", {"any": "Any", "ascii": "ASCII", "assigned": "Assigned"}[key])

    perl = {
        "digit": sets[("g", "Nd")],
        "space": sets[("b", "White_Space")],
        "word": union(
            sets[("b", "Alphabetic")],
            sets[("g", "M")],
            sets[("g", "Nd")],
            sets[("g", "Pc")],
            sets[("b", "Join_Control")],
        ),
    }
    for name, ranges in perl.items():
        sets[("r", name)] = ranges

    # Property names accepted before `=`; the value says which table the value is looked up in.
    property_kinds = {}
    for aliases in [a for section in properties.values() for a in section]:
        kind = {"General_Category": "g", "Script": "s", "Script_Extensions": "x"}.get(aliases[1])
        if kind:
            for name in aliases:
                property_kinds[loose(name)] = kind
    if sorted(set(property_kinds.values())) != ["g", "s", "x"]:
        raise SystemExit("PropertyAliases.txt: gc, sc or scx not found")
    for name, kind in property_kinds.items():
        names[("P", name)] = kind

    return sets, names, skipped, gc_labels


def _spans(runs):
    by_label = {}
    for lo, hi, label in runs:
        by_label.setdefault(label, []).append((lo, hi))
    return by_label


# ---------------------------------------------------------------- encoding


def varint(n):
    out = []
    while True:
        group = n & 31
        n >>= 5
        if n:
            out.append(ALPHABET[group | 32])
        else:
            out.append(ALPHABET[group])
            return "".join(out)


def encode_ranges(ranges):
    out = [varint(len(ranges))]
    next_free = 0
    for lo, hi in ranges:
        out.append(varint(lo - next_free))
        out.append(varint(hi - lo))
        next_free = hi + 2
    return "".join(out)


def encode_orbits(orbits):
    out = [varint(len(orbits))]
    previous = 0
    for members in orbits:
        out.append(varint(len(members) - 2))
        out.append(varint(members[0] - previous))
        previous = members[0]
        for a, b in zip(members, members[1:]):
            out.append(varint(b - a - 1))
    return "".join(out)


class Reader:
    def __init__(self, text, pos):
        self.text = text
        self.pos = pos

    def varint(self):
        n = shift = 0
        while True:
            if self.text[self.pos] == "\n":
                self.pos += 1
            value = ALPHABET.index(self.text[self.pos])
            self.pos += 1
            n |= (value & 31) << shift
            shift += 5
            if not value & 32:
                return n


def decode_ranges(blob, pos):
    reader = Reader(blob, pos)
    out = []
    next_free = 0
    for _ in range(reader.varint()):
        lo = next_free + reader.varint()
        hi = lo + reader.varint()
        out.append((lo, hi))
        next_free = hi + 2
    return out


def decode_orbits(blob, pos):
    reader = Reader(blob, pos)
    out = []
    previous = 0
    for _ in range(reader.varint()):
        n = reader.varint() + 2
        first = previous + reader.varint()
        previous = first
        members = [first]
        for _ in range(n - 1):
            members.append(members[-1] + reader.varint() + 1)
        out.append(members)
    return out


def raw_offset(flat_offset):
    """Offset in the text with a newline after every LINE_WIDTH characters."""
    return flat_offset + flat_offset // LINE_WIDTH


def render(sets, names, orbits):
    flat = []
    flat_size = 0
    offsets = {}  # tuple of ranges -> raw offset
    key_offset = {}
    for key in sorted(sets):
        ranges = tuple(sets[key])
        if ranges not in offsets:
            payload = encode_ranges(ranges)
            offsets[ranges] = raw_offset(flat_size)
            flat.append(payload)
            flat_size += len(payload)
        key_offset[key] = offsets[ranges]
    fold_offset = raw_offset(flat_size)
    flat.append(encode_orbits(orbits))
    blob = "".join(flat)
    lines = [blob[i : i + LINE_WIDTH] for i in range(0, len(blob), LINE_WIDTH)]
    blob_text = "\n".join(lines) + "\n"

    entries = {}
    for (kind, name), target in names.items():
        value = target if kind == "P" else str(key_offset[target])
        entries[kind + name] = value
    index_lines = [f"{k}={entries[k]}" for k in sorted(entries, key=lambda k: k.encode())]
    index_text = "\n".join(index_lines) + "\n"

    perl = {n: key_offset[("r", n)] for n in ("digit", "space", "word")}
    out = f"""// Generated by tools/gen_regex_unicode.py from the Unicode Character Database {UCD_VERSION}.
// Do not edit; see docs/tools/regex-unicode-tables.md.

#scope_module

UNICODE_VERSION :: "{UCD_VERSION}";

UNICODE_PERL_DIGIT_OFFSET :: {perl["digit"]};
UNICODE_PERL_SPACE_OFFSET :: {perl["space"]};
UNICODE_PERL_WORD_OFFSET :: {perl["word"]};
UNICODE_FOLD_OFFSET :: {fold_offset};

// One line per name: a kind letter, the loosely normalized name, `=`, then a decimal offset into
// UNICODE_RANGE_DATA (kind P names a table kind instead). Sorted by byte order for binary search.
UNICODE_NAME_INDEX :: #string END_OF_UNICODE_NAME_INDEX
{index_text}END_OF_UNICODE_NAME_INDEX

// Base-64 digits (five payload bits, low group first, bit 5 continues); line breaks are skipped.
UNICODE_RANGE_DATA :: #string END_OF_UNICODE_RANGE_DATA
{blob_text}END_OF_UNICODE_RANGE_DATA
"""
    return out, blob_text, index_text, key_offset, fold_offset


# ---------------------------------------------------------------- checks


def verify(sets, names, orbits, rendered, gc_labels):
    _, blob_text, index_text, key_offset, fold_offset = rendered
    for key, ranges in sets.items():
        if decode_ranges(blob_text, key_offset[key]) != ranges:
            raise SystemExit(f"round trip failed for {key}")
    if decode_orbits(blob_text, fold_offset) != orbits:
        raise SystemExit("round trip failed for the fold table")
    for line in index_text.splitlines():
        if not line.count("=") == 1:
            raise SystemExit(f"bad index line {line!r}")
    gc_groups = [k for (kind, k) in sets if kind == "g" and len(k) == 2 and k != "LC"]
    covered = 0
    for g in gc_groups:
        covered += sum(hi - lo + 1 for lo, hi in sets[("g", g)])
    if covered != MAX_CP + 1:
        raise SystemExit("general categories do not partition the code space")
    for members in orbits:
        if len(set(members)) != len(members):
            raise SystemExit("duplicate in a fold orbit")
    if unicodedata.unidata_version == UCD_VERSION:
        for cp in range(MAX_CP + 1):
            expected = unicodedata.category(chr(cp))
            if gc_labels[cp] != expected:
                raise SystemExit(f"U+{cp:04X}: category {gc_labels[cp]}, unicodedata says {expected}")
        print("general categories agree with unicodedata", file=sys.stderr)
    else:
        print(
            f"note: unicodedata is {unicodedata.unidata_version}, not {UCD_VERSION}; "
            "skipping the category cross-check",
            file=sys.stderr,
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    where = parser.add_mutually_exclusive_group()
    where.add_argument("--ucd", help="directory with the extracted UCD files")
    where.add_argument("--zip", help="UCD.zip (hash is verified)")
    parser.add_argument("--check", action="store_true", help="fail if the committed file differs")
    parser.add_argument("--output", default=str(OUTPUT))
    args = parser.parse_args()

    source = open_source(args)
    sets, names, skipped, gc_labels = build_sets(source)
    orbits = read_folding(source)
    rendered = render(sets, names, orbits)
    verify(sets, names, orbits, rendered, gc_labels)
    text = rendered[0]
    if skipped:
        print("ignored (no alias entry): " + ", ".join(sorted(skipped)), file=sys.stderr)

    output = Path(args.output)
    if args.check:
        if not output.exists() or output.read_text(encoding="utf-8") != text:
            print(f"{output} is out of date; run tools/gen_regex_unicode.py", file=sys.stderr)
            return 1
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text, encoding="utf-8")
    print(f"wrote {output} ({len(text)} bytes)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
