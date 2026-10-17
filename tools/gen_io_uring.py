#!/usr/bin/env python3
"""Generate stdlib/Extensions/Io_Uring/bindings.jai from the kernel's UAPI header linux/io_uring.h.

    python3 tools/gen_io_uring.py                 # fetch the pinned header, write bindings.jai
    python3 tools/gen_io_uring.py --header PATH   # use a header on disk (a newer kernel, say)
    python3 tools/gen_io_uring.py --check         # fail when bindings.jai differs from the output

The header is pinned by kernel tag and sha256 below; --header skips the pin (the output says which
file it came from) and is how you regenerate for a kernel that added something. Updating the pin is
two edits: TAG and SHA256 (`sha256sum io_uring.h` of the file at that tag). A kernel only ever adds
to this header, so regenerating is additive; the layout checks the generator writes (`#assert
size_of/offset_of`) fail the build of the module if jaic lays a struct out differently from C.

The parser handles the subset of C this header uses: #define of integer expressions, enums,
structs and unions (anonymous ones nested, `__DECLARE_FLEX_ARRAY`, zero-length arrays, packed
unions) and the fixed-width kernel integer types. Anything else stops the run with a message, so a
new construct in a future header is a visible task, not a silent gap. Offsets and sizes are
computed here with C's layout rules and compared with the sizes the kernel's own ABI is known for
(KNOWN_SIZES); the same numbers become the `#assert`s in the output.

Run `just fmt stdlib/Extensions/Io_Uring` afterwards if the output changed shape: the generator
writes jaifmt's style, so a clean run needs no formatting.
"""
from __future__ import annotations

import argparse
import hashlib
import re
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / 'stdlib' / 'Extensions' / 'Io_Uring' / 'bindings.jai'

TAG = 'v7.0'
URL = f'https://raw.githubusercontent.com/torvalds/linux/{TAG}/include/uapi/linux/io_uring.h'
SHA256 = 'dd09db3c165c95d24cd1cf0229b94ea9ab06e0764e7c67e0b047148d053f5e36'

# Fixed-width kernel integers: (Jai type, size, alignment). `long long`-sized fields of other
# kernel headers appear only through EXTERNAL_STRUCTS.
SCALARS = {
    '__u8': ('u8', 1), '__s8': ('s8', 1),
    '__u16': ('u16', 2), '__s16': ('s16', 2),
    '__u32': ('u32', 4), '__s32': ('s32', 4),
    '__u64': ('u64', 8), '__s64': ('s64', 8), '__aligned_u64': ('u64', 8),
}

# Structs of other kernel headers that io_uring.h uses: (fields, size, alignment). The text is the
# Jai declaration. linux/time_types.h: `__kernel_time64_t tv_sec; long long tv_nsec;`.
EXTERNAL_STRUCTS = {
    '__kernel_timespec': ([('tv_sec', 's64'), ('tv_nsec', 's64')], 16, 8),
}

# Enums whose members are narrower than the u32 default, because a struct field or the SQE holds
# them in fewer bytes.
ENUM_TYPES = {'io_uring_op': 'u8'}

# Enums whose member names lose a prefix that is not the longest common one.
ENUM_PREFIXES = {'zcrx_ctrl_op': 'ZCRX_CTRL_', 'zcrx_features': 'ZCRX_FEATURE_'}

# `~` and negative results of macros: the C type decides how wide the result is.
MACRO_WIDTH = {'IORING_FILE_INDEX_ALLOC': 32, 'IORING_ZCRX_AREA_MASK': 64}
MACRO_SIGNED = {'IORING_REGISTER_FILES_SKIP': 's32'}

# Macros that are not part of the interface.
SKIPPED_MACROS = {'LINUX_IO_URING_H', 'UAPI_LINUX_IO_URING_H_SKIP_LINUX_TIME_TYPES_H'}

# Sizes the kernel ABI fixes (BUILD_BUG_ONs in the kernel, static asserts in liburing); a mismatch
# means this parser laid a struct out wrongly.
KNOWN_SIZES = {
    'io_uring_sqe': 64, 'io_uring_cqe': 16, 'io_uring_params': 120,
    'io_sqring_offsets': 40, 'io_cqring_offsets': 40, 'io_uring_probe_op': 8,
    'io_uring_probe': 16, 'io_uring_buf': 16, 'io_uring_buf_reg': 40,
    'io_uring_getevents_arg': 24, 'io_uring_rsrc_update2': 32, 'io_uring_sync_cancel_reg': 64,
    'io_uring_recvmsg_out': 16, 'io_uring_reg_wait': 64, 'io_uring_restriction': 16,
}

# Names Jai reserves; a C member with one of them gets a trailing underscore.
JAI_KEYWORDS = {'struct', 'union', 'enum', 'if', 'else', 'for', 'while', 'return', 'cast', 'context',
                'defer', 'using', 'null', 'true', 'false', 'inline', 'it', 'size_of', 'type_of'}


class GenError(Exception):
    pass


# ---------------------------------------------------------------------------------------------
# Reading the header
# ---------------------------------------------------------------------------------------------

def strip_comments(text: str) -> str:
    text = re.sub(r'/\*.*?\*/', ' ', text, flags=re.S)
    return re.sub(r'//[^\n]*', '', text)


def items(text: str) -> list[tuple]:
    """The header's macros and top-level declarations, in source order:
    ('define', name, expression) and ('decl', text)."""
    result: list[tuple] = []
    buffer: list[str] = []
    skipping = False
    lines = text.replace('\\\n', ' ').split('\n')
    for line in lines:
        stripped = line.strip()
        if stripped.startswith('#'):
            directive = stripped[1:].strip()
            if directive.startswith('ifdef __cplusplus'):
                skipping = True
            elif skipping and directive.startswith('endif'):
                skipping = False
            elif not skipping and directive.startswith('define'):
                parts = directive.split(None, 2)
                if len(parts) == 3 and '(' not in parts[1]:
                    result.append(('define', parts[1], parts[2].strip()))
                elif len(parts) == 2:
                    result.append(('define', parts[1], ''))
            continue
        if skipping or not stripped:
            continue
        buffer.append(stripped)
        joined = ' '.join(buffer)
        if joined.count('{') == joined.count('}') and joined.endswith(';'):
            result.append(('decl', joined))
            buffer = []
    if buffer:
        raise GenError(f'unterminated declaration: {" ".join(buffer)[:80]}')
    return result


TOKEN = re.compile(r'0[xX][0-9a-fA-F]+[uUlL]*|\d+[uUlL]*|[A-Za-z_]\w*|<<|>>|\S')


def tokenize(text: str) -> list[str]:
    return TOKEN.findall(text)


def evaluate(expression: str, env: dict[str, int], what: str) -> int:
    """A C integer constant expression over literals and constants already known."""
    text = re.sub(r'\(\s*__[us](?:8|16|32|64)\s*\)', '', expression)
    out = []
    for token in tokenize(text):
        if re.fullmatch(r'0[xX][0-9a-fA-F]+[uUlL]*|\d+[uUlL]*', token):
            out.append(str(int(token.rstrip('uUlL'), 0)))
        elif re.fullmatch(r'[A-Za-z_]\w*', token):
            if token not in env:
                raise GenError(f'{what}: unknown name {token} in `{expression}`')
            out.append(str(env[token]))
        elif token in '()|&^~+-*' or token in ('<<', '>>'):
            out.append(token)
        else:
            raise GenError(f'{what}: cannot evaluate `{expression}` (at {token!r})')
    try:
        return int(eval(' '.join(out), {'__builtins__': {}}, {}))  # noqa: S307 (tokens checked above)
    except Exception as error:
        raise GenError(f'{what}: cannot evaluate `{expression}`: {error}') from error


# ---------------------------------------------------------------------------------------------
# The model
# ---------------------------------------------------------------------------------------------

class Field:
    def __init__(self, name: str, ctype: str, dims: list[int | None]):
        self.name, self.ctype, self.dims = name, ctype, dims


class Aggregate:
    """A struct or union body: members are Fields and anonymous Aggregates."""

    def __init__(self, kind: str, name: str | None, members: list):
        self.kind, self.name, self.members = kind, name, members


def parse_aggregate_body(tokens: list[str], pos: int, kind: str) -> tuple[list, int]:
    """Members from tokens[pos] up to the matching `}`; returns them and the index after it."""
    members: list = []
    while tokens[pos] != '}':
        token = tokens[pos]
        if token in ('struct', 'union') and tokens[pos + 1] == '{':
            body, pos = parse_aggregate_body(tokens, pos + 2, token)
            if tokens[pos:pos + 7] == ['__attribute__', '(', '(', 'packed', ')', ')', ';'][:7]:
                pos += 6
            elif tokens[pos] == '__attribute__':
                end = pos
                while tokens[end] != ';':
                    end += 1
                pos = end
            if tokens[pos] != ';':
                raise GenError(f'a named nested aggregate is not supported: {tokens[pos]}')
            members.append(Aggregate(token, None, body))
            pos += 1
            continue
        if token == '__DECLARE_FLEX_ARRAY':
            end = tokens.index(')', pos)
            inner = tokens[pos + 2:end]
            comma = inner.index(',')
            members.append(Field(inner[comma + 1], ' '.join(inner[:comma]), [0]))
            pos = end + 2
            continue
        end = pos
        while tokens[end] != ';':
            end += 1
        declaration = tokens[pos:end]
        dims: list[int | None] = []
        while declaration and declaration[-1] == ']':
            open_at = len(declaration) - 1 - declaration[::-1].index('[')
            size_tokens = declaration[open_at + 1:-1]
            dims.insert(0, int(''.join(size_tokens), 0) if size_tokens else 0)
            declaration = declaration[:open_at]
        if len(declaration) < 2:
            raise GenError(f'cannot read member `{" ".join(tokens[pos:end])}`')
        if ',' in declaration:
            raise GenError(f'several declarators in one member are not supported: {" ".join(declaration)}')
        members.append(Field(declaration[-1], ' '.join(declaration[:-1]), dims))
        pos = end + 1
    return members, pos + 1


def parse_declaration(text: str) -> tuple[str, str | None, object]:
    """('enum' | 'struct' | 'union', name, body): the members for an aggregate, (name, value
    expression) pairs for an enum."""
    tokens = tokenize(text)
    kind = tokens[0]
    if kind not in ('enum', 'struct', 'union'):
        raise GenError(f'unsupported declaration: {text[:80]}')
    name = tokens[1] if tokens[1] != '{' else None
    start = tokens.index('{') + 1
    if kind == 'enum':
        end = len(tokens) - 1
        while tokens[end] != '}':
            end -= 1
        body, entry, depth = [], [], 0
        for token in tokens[start:end] + [',']:
            if token == '(':
                depth += 1
            elif token == ')':
                depth -= 1
            if token == ',' and depth == 0:
                if entry:
                    if len(entry) > 1 and entry[1] != '=':
                        raise GenError(f'cannot read enum entry {" ".join(entry)}')
                    body.append((entry[0], ' '.join(entry[2:]) if len(entry) > 2 else None))
                entry = []
            else:
                entry.append(token)
        return kind, name, body
    members, pos = parse_aggregate_body(tokens, start, kind)
    if tokens[pos:] != [';']:
        raise GenError(f'unsupported text after `{name}`: {" ".join(tokens[pos:])[:60]}')
    return kind, name, members


# ---------------------------------------------------------------------------------------------
# Layout
# ---------------------------------------------------------------------------------------------

class Layout:
    def __init__(self):
        self.structs: dict[str, Aggregate] = {}
        self.cache: dict[str, tuple[int, int]] = {}

    def scalar(self, ctype: str) -> tuple[int, int]:
        if ctype in SCALARS:
            size = SCALARS[ctype][1]
            return size, size
        if ctype.startswith('struct '):
            return self.of_struct(ctype[7:])
        raise GenError(f'unknown type {ctype}')

    def of_struct(self, name: str) -> tuple[int, int]:
        if name in EXTERNAL_STRUCTS:
            return EXTERNAL_STRUCTS[name][1], EXTERNAL_STRUCTS[name][2]
        if name not in self.cache:
            if name not in self.structs:
                raise GenError(f'struct {name} used before it is declared')
            size, align, _ = self.of_aggregate(self.structs[name], 0)
            self.cache[name] = (size, align)
        return self.cache[name]

    def of_aggregate(self, agg: Aggregate, base: int) -> tuple[int, int, list[tuple[str, int]]]:
        """(size, alignment, [(member name, absolute offset)]) of an aggregate placed at `base`,
        with the members of anonymous aggregates listed as if they were direct members."""
        offsets: list[tuple[str, int]] = []
        cursor, align, largest = 0, 1, 0
        for member in agg.members:
            if isinstance(member, Aggregate):
                inner_align = self._aggregate_align(member)
                start = cursor if agg.kind == 'struct' else 0
                start = -(-start // inner_align) * inner_align
                size, inner_align, inner = self.of_aggregate(member, base + start)
                offsets += inner
                member_size, member_align, member_offset = size, inner_align, start
            else:
                size, inner_align = self.scalar(member.ctype)
                for dim in member.dims:
                    size *= dim
                member_align = inner_align
                member_offset = -(-(cursor if agg.kind == 'struct' else 0) // member_align) * member_align
                offsets.append((member.name, base + member_offset))
                member_size = size
            align = max(align, member_align)
            if agg.kind == 'struct':
                cursor = member_offset + member_size
            else:
                largest = max(largest, member_size)
        size = cursor if agg.kind == 'struct' else largest
        return -(-size // align) * align, align, offsets

    def _aggregate_align(self, agg: Aggregate) -> int:
        return self.of_aggregate(agg, 0)[1]


# ---------------------------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------------------------

def jai_name(name: str) -> str:
    return name + '_' if name in JAI_KEYWORDS else name


def jai_type(field: Field) -> str:
    ctype = field.ctype
    if ctype in SCALARS:
        base = SCALARS[ctype][0]
    elif ctype.startswith('struct '):
        base = ctype[7:]
    else:
        raise GenError(f'unknown type {ctype}')
    for dim in reversed(field.dims):
        base = f'[{dim}] {base}'
    return base


def emit_members(members: list, indent: int, lines: list[str]) -> None:
    pad = '    ' * indent
    for member in members:
        if isinstance(member, Aggregate):
            lines.append(f'{pad}{member.kind} {{')
            emit_members(member.members, indent + 1, lines)
            lines.append(f'{pad}}};')
        else:
            lines.append(f'{pad}{jai_name(member.name)}: {jai_type(member)};')


def literal(value: int) -> str:
    return str(value) if value < 1024 else hex(value)


def enum_prefix(name: str, names: list[str]) -> str:
    if name in ENUM_PREFIXES:
        return ENUM_PREFIXES[name]
    candidates = [n for n in names if not n.startswith('__')]
    prefix = candidates[0]
    for other in candidates[1:]:
        while not other.startswith(prefix):
            prefix = prefix[:-1]
    cut = prefix.rfind('_')
    return prefix[:cut + 1] if cut >= 0 else ''


def generate(header_text: str, origin: str) -> str:
    env: dict[str, int] = {}
    layout = Layout()
    sections: list[str] = []
    constants: list[str] = []
    enums: list[str] = []
    structs: list[str] = []
    asserts: list[str] = []
    external = ['__kernel_timespec :: struct {', '    tv_sec: s64;', '    tv_nsec: s64;', '}']

    for item in items(strip_comments(header_text)):
        if item[0] == 'define':
            _, name, expression = item
            if name in SKIPPED_MACROS or not expression:
                continue
            value = evaluate(expression, env, name)
            if name in MACRO_WIDTH:
                value &= (1 << MACRO_WIDTH[name]) - 1
            if value < 0 and name not in MACRO_SIGNED:
                raise GenError(f'{name} is negative; give it a width or a signed type')
            env[name] = value
            if name in MACRO_SIGNED:
                constants.append(f'{name}: {MACRO_SIGNED[name]} : {value};')
            elif value >= 1 << 63:
                constants.append(f'{name}: u64 : {hex(value)};')
            else:
                constants.append(f'{name} :: {literal(value)};')
            continue

        kind, name, body = parse_declaration(item[1])
        if kind == 'enum':
            value, resolved = 0, []
            for member, expression in body:
                if expression is not None:
                    value = evaluate(expression, env, member)
                env[member] = value
                resolved.append((member, value))
                value += 1
            if name is None:
                constants += [f'{m} :: {literal(v)};' for m, v in resolved]
                continue
            prefix = enum_prefix(name, [m for m, _ in resolved])
            lines = [f'{name} :: enum {ENUM_TYPES.get(name, "u32")} {{']
            for member, number in resolved:
                short = member
                for lead in (prefix, '__' + prefix):
                    if prefix and member.startswith(lead):
                        short = member[len(lead):]
                        break
                if not short or short[0].isdigit():
                    short = member
                lines.append(f'    {short} :: {literal(number)};')
            lines.append('}')
            enums.append('\n'.join(lines))
            continue

        aggregate = Aggregate(kind, name, body)
        layout.structs[name] = aggregate
        size, _, offsets = layout.of_aggregate(aggregate, 0)
        if name in KNOWN_SIZES and KNOWN_SIZES[name] != size:
            raise GenError(f'{name} comes out {size} bytes, but the kernel ABI is {KNOWN_SIZES[name]}')
        lines = [f'{name} :: {kind} {{']
        emit_members(body, 1, lines)
        lines.append('}')
        structs.append('\n'.join(lines))
        asserts.append(f'#assert size_of({name}) == {size};')
        asserts += [f'#assert offset_of({name}, "{jai_name(member)}") == {offset};' for member, offset in offsets]

    for name in KNOWN_SIZES:
        if name not in layout.structs:
            raise GenError(f'{name} is not in the header any more; update KNOWN_SIZES')

    header = f'''// Kernel ABI for io_uring: constants, enums and structs.
//
// GENERATED by tools/gen_io_uring.py from {origin}
// (SPDX-License-Identifier: (GPL-2.0 WITH Linux-syscall-note) OR MIT). Do not edit by hand:
// change the generator and rerun it; docs/extensions/io-uring.md says how.
//
// The structs keep the kernel's names and layout. The `#assert`s at the end check jaic's
// layout of each one against the offsets C gives them. Enums drop the prefix their members share
// (`io_uring_op.WRITE` is IORING_OP_WRITE); every other `#define` is a constant of the same name.
// The flexible array members of C (`ops[]`, `big_cqe[]`, `bufs[]`) are `[0]` arrays at the end of
// their struct: the entries start right after it.
'''
    parts = [header.rstrip('\n'), '\n'.join(constants), '\n\n'.join(enums), '\n'.join(external),
             '\n\n'.join(structs), '\n'.join(asserts)]
    return '\n\n'.join(parts) + '\n'


# ---------------------------------------------------------------------------------------------
# Command line
# ---------------------------------------------------------------------------------------------

def fetch() -> str:
    data = urllib.request.urlopen(URL, timeout=60).read()
    digest = hashlib.sha256(data).hexdigest()
    if digest != SHA256:
        raise GenError(f'{URL} has sha256 {digest}, expected {SHA256}')
    return data.decode()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--header', type=Path, help='read this io_uring.h instead of fetching the pinned one')
    parser.add_argument('--check', action='store_true', help='compare with the committed bindings instead of writing')
    parser.add_argument('--out', type=Path, default=OUTPUT)
    args = parser.parse_args()
    try:
        if args.header:
            text = args.header.read_text()
            digest = hashlib.sha256(text.encode()).hexdigest()
            origin = (f'linux/io_uring.h at Linux {TAG}' if digest == SHA256
                      else f'{args.header.name} (sha256 {digest[:12]})')
        else:
            text = fetch()
            origin = f'linux/io_uring.h at Linux {TAG}'
        output = generate(text, origin)
    except GenError as error:
        print(f'gen_io_uring: {error}', file=sys.stderr)
        return 1
    if args.check:
        if not args.out.exists() or args.out.read_text() != output:
            print(f'{args.out} is out of date; run tools/gen_io_uring.py', file=sys.stderr)
            return 1
        return 0
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(output)
    print(f'wrote {args.out}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
