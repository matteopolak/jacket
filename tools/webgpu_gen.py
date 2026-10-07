#!/usr/bin/env python3
"""Generate the WebGPU bindings from webgpu-headers' machine-readable spec (webgpu.yml).

    python3 tools/webgpu_gen.py [--yml PATH] [--check-header webgpu.h]

Reads the pinned webgpu.yml (tools/webgpu.json: revision + sha256; downloaded into
artifacts/webgpu/ unless --yml is given) and writes:

  stdlib/Extensions/WebGPU/generated.jai                      types, enums, structs, callbacks, #foreign procs
  stdlib/Extensions/WebGPU/generated_wasm.jai                 callback dispatch for the browser sandbox
  crates/jai-wasm/js/webgpu_bindings.generated.mjs layouts and call descriptors for the JS host

--check-header renders every struct, enum value and function back to C and compares them with
the webgpu.h of the same revision, which catches mistakes in the naming and layout rules below.
See docs/stdlib/webgpu.md.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / 'tools' / 'webgpu.json'
OUT_JAI = ROOT / 'stdlib' / 'Extensions' / 'WebGPU' / 'generated.jai'
OUT_JAI_WASM = ROOT / 'stdlib' / 'Extensions' / 'WebGPU' / 'generated_wasm.jai'
OUT_JS = ROOT / 'crates' / 'jai-wasm' / 'js' / 'webgpu_bindings.generated.mjs'


# ---------------------------------------------------------------------------
# A YAML subset: block mappings and sequences, `|` literal blocks, plain and quoted scalars and
# `[]`. That is all webgpu.yml uses; anything else fails loudly.
# ---------------------------------------------------------------------------

class Yaml:
    def __init__(self, text: str):
        self.lines = text.split('\n')
        self.i = 0

    def skip_blank(self):
        while self.i < len(self.lines) and (not self.lines[self.i].strip() or self.lines[self.i].lstrip().startswith('#')):
            self.i += 1

    def indent(self) -> int:
        self.skip_blank()
        if self.i >= len(self.lines):
            return -1
        line = self.lines[self.i]
        return len(line) - len(line.lstrip(' '))

    def block(self, indent: int):
        if self.indent() != indent:
            raise ValueError(f'line {self.i + 1}: expected indent {indent}')
        if self.lines[self.i].lstrip().startswith('- ') or self.lines[self.i].strip() == '-':
            return self.sequence(indent)
        return self.mapping(indent)

    def sequence(self, indent: int) -> list:
        out = []
        while self.indent() == indent and self.lines[self.i].lstrip().startswith('-'):
            line = self.lines[self.i]
            rest = line[indent + 1:].lstrip(' ')
            if not rest:
                self.i += 1
                out.append(self.block(self.indent()))
            elif re.match(r'^[A-Za-z_][\w]*:( |$)', rest):
                # `- key: value` opens a mapping whose keys sit where `key` starts.
                inner = len(line) - len(rest)
                self.lines[self.i] = ' ' * inner + rest
                out.append(self.mapping(inner))
            else:
                self.i += 1
                out.append(scalar(rest))
        return out

    def mapping(self, indent: int) -> dict:
        out = {}
        while self.indent() == indent and not self.lines[self.i].lstrip().startswith('- '):
            line = self.lines[self.i].strip()
            m = re.match(r'^([A-Za-z_][\w]*):(?: (.*))?$', line)
            if not m:
                raise ValueError(f'line {self.i + 1}: not a mapping entry: {line!r}')
            key, rest = m.group(1), (m.group(2) or '').strip()
            self.i += 1
            if rest == '|':
                out[key] = self.literal(indent)
            elif rest == '':
                nxt = self.indent()
                if nxt > indent:
                    out[key] = self.block(nxt)
                elif nxt == indent and self.lines[self.i].lstrip().startswith('- '):
                    out[key] = self.sequence(indent)
                else:
                    out[key] = None
            else:
                # Plain scalars may continue on more deeply indented lines.
                parts = [rest]
                while self.i < len(self.lines) and self.lines[self.i].strip() and \
                        len(self.lines[self.i]) - len(self.lines[self.i].lstrip(' ')) > indent and \
                        not rest.startswith(('"', "'")):
                    parts.append(self.lines[self.i].strip())
                    self.i += 1
                out[key] = scalar(' '.join(parts))
        return out

    def literal(self, indent: int) -> str:
        body = []
        while self.i < len(self.lines):
            line = self.lines[self.i]
            if line.strip() and len(line) - len(line.lstrip(' ')) <= indent:
                break
            body.append(line)
            self.i += 1
        while body and not body[-1].strip():
            body.pop()
        cut = min((len(l) - len(l.lstrip(' ')) for l in body if l.strip()), default=0)
        return '\n'.join(l[cut:] for l in body) + '\n'


def scalar(text: str):
    if text == '[]':
        return []
    if text == 'null':
        return None
    if len(text) >= 2 and text[0] == text[-1] and text[0] in '"\'':
        return text[1:-1]
    return text


def parse_yaml(text: str):
    return Yaml(text).block(0)


# ---------------------------------------------------------------------------
# Names
# ---------------------------------------------------------------------------

def pascal(name: str) -> str:
    out = ''
    for p in name.split('_'):
        # Number groups stay apart: `unorm10_10_10_2` -> `Unorm10_10_10_2`.
        if out and out[-1].isdigit() and p[:1].isdigit():
            out += '_'
        out += p[:1].upper() + p[1:]
    return out


def camel(name: str) -> str:
    first, *rest = name.split('_')
    return first + ''.join(p[:1].upper() + p[1:] for p in rest)


def ctype(name: str) -> str:
    return 'WGPU' + pascal(name)


def singular(name: str) -> str:
    if name.endswith('ies'):
        return name[:-3] + 'y'
    if name.endswith('s'):
        return name[:-1]
    return name


JAI_RESERVED = {'if', 'else', 'case', 'then', 'for', 'while', 'break', 'continue', 'return', 'defer', 'struct',
                'union', 'enum', 'enum_flags', 'cast', 'xx', 'using', 'inline', 'no_inline', 'null', 'true', 'false',
                'context', 'it', 'it_index', 'size_of', 'type_of', 'operator', 'interface', 'push_context', 'ifx',
                'remove', 'type_info', 'is_constant', 'initializer_of'}


def jai_ident(name: str) -> str:
    if name[:1].isdigit():
        return '_' + name
    if name in JAI_RESERVED:
        return name + '_'
    return name


def js_enum_string(enum: str, entry: str) -> str:
    """The WebIDL string of an enum entry (`rgba8_unorm_srgb` -> "rgba8unorm-srgb")."""
    parts = entry.lower().split('_')
    if enum != 'texture_format':
        return '-'.join(parts)
    # Texture formats join a component group ending in a digit to what follows ("r8unorm",
    # "depth24plus"), except the compressed family prefixes and ASTC block sizes.
    family = re.fullmatch(r'bc\d+h?|etc2|eac|astc', parts[0]) is not None
    out = parts[0]
    for k in range(1, len(parts)):
        prev = parts[k - 1]
        join = prev[-1].isdigit() and not re.fullmatch(r'\d+x\d+', prev) and not (k == 1 and family)
        out += parts[k] if join else '-' + parts[k]
    return out


# ---------------------------------------------------------------------------
# The model
# ---------------------------------------------------------------------------

PRIMS = {
    # yml type: (jai type, C type, size, js kind)
    'bool': ('WGPUBool', 'WGPUBool', 4, 'bool'),
    'uint16': ('u16', 'uint16_t', 2, 'u16'),
    'uint32': ('u32', 'uint32_t', 4, 'u32'),
    'uint64': ('u64', 'uint64_t', 8, 'u64'),
    'int32': ('s32', 'int32_t', 4, 'i32'),
    'usize': ('u64', 'size_t', 8, 'u64'),
    'float32': ('float32', 'float', 4, 'f32'),
    'nullable_float32': ('float32', 'float', 4, 'f32'),
    'float64_supertype': ('float64', 'double', 8, 'f64'),
    'c_void': ('void', 'void', 0, 'ptr'),
}
STRINGS = {'string_with_default_empty': 'str', 'nullable_string': 'nstr', 'out_string': 'ostr'}


class Spec:
    def __init__(self, doc: dict):
        self.doc = doc
        self.constants = {c['name']: c for c in doc['constants']}
        self.enums = {e['name']: e for e in doc['enums']}
        self.bitflags = {b['name']: b for b in doc['bitflags']}
        self.structs = {s['name']: s for s in doc['structs']}
        self.callbacks = {c['name']: c for c in doc['callbacks']}
        self.objects = {o['name']: o for o in doc['objects']}
        self.prefix = int(doc.get('enum_prefix', '0'), 0)
        self.layouts: dict[str, tuple[int, int, list]] = {}

    # Enum and bitflag values -------------------------------------------------

    def enum_values(self, name: str) -> list[tuple[str, int]]:
        out = []
        for i, entry in enumerate(self.enums[name]['entries']):
            if entry is None:
                continue
            value = int(entry['value'], 0) if 'value' in entry else i
            out.append((entry['name'], (self.prefix << 16) | value))
        return out

    def bitflag_values(self, name: str) -> list[tuple[str, int]]:
        out, by_name = [], {}
        bit = 0
        for entry in self.bitflags[name]['entries']:
            if 'value_combination' in entry:
                value = 0
                for part in entry['value_combination']:
                    value |= by_name[part]
            elif 'value' in entry:
                value = int(entry['value'], 0)
            elif entry['name'] == 'none':
                value = 0
            else:
                value = 1 << bit
                bit += 1
            by_name[entry['name']] = value
            out.append((entry['name'], value))
        return out

    # Types -------------------------------------------------------------------

    def fields(self, m: dict, args: bool = False) -> list[tuple[str, str, str, int, int]]:
        """(name, jai type, C type, size, align) for one member or argument; arrays give two."""
        t, name, pointer = m['type'], camel(m['name']), m.get('pointer')
        if t.startswith('array<'):
            inner = t[len('array<'):-1]
            jai, c, _, _ = self.scalar(inner)
            const = ' const' if pointer != 'mutable' else ''
            count = camel(singular(m['name'])) + 'Count'
            return [(count, 'u64', 'size_t', 8, 8), (name, f'*{jai}', f'{c}{const} *', 8, 8)]
        jai, c, size, align = self.scalar(t)
        if pointer:
            const = ' const' if pointer == 'immutable' else ''
            return [(name, f'*{jai}', f'{c}{const} *', 8, 8)]
        return [(name, jai, c, size, align)]

    def scalar(self, t: str) -> tuple[str, str, int, int]:
        if t in PRIMS:
            jai, c, size, _ = PRIMS[t]
            return jai, c, size, max(size, 1)
        if t in STRINGS:
            return 'WGPUStringView', 'WGPUStringView', 16, 8
        kind, _, name = t.partition('.')
        if kind == 'enum':
            return ctype(name), ctype(name), 4, 4
        if kind == 'bitflag':
            return ctype(name), ctype(name), 8, 8
        if kind == 'object':
            return ctype(name), ctype(name), 8, 8
        if kind == 'struct':
            size, align, _ = self.layout(name)
            return ctype(name), ctype(name), size, align
        if kind == 'callback':
            return ctype(name) + 'CallbackInfo', ctype(name) + 'CallbackInfo', 40 if self.callbacks[name]['style'] == 'callback_mode' else 32, 8
        raise ValueError(f'unknown type {t}')

    def struct_fields(self, name: str) -> list[tuple]:
        """(name, jai, C, size, align, member) in order, with the chain header first."""
        s = self.structs[name]
        out = []
        if s['type'] in ('extensible', 'extensible_callback_arg'):
            out.append(('nextInChain', '*WGPUChainedStruct', 'WGPUChainedStruct *', 8, 8, None))
        elif s['type'] == 'extension':
            out.append(('chain', 'WGPUChainedStruct', 'WGPUChainedStruct', 16, 8, None))
        elif s['type'] != 'standalone':
            raise ValueError(f'struct {name}: unknown kind {s["type"]}')
        for m in s.get('members') or []:
            for f in self.fields(m):
                out.append((*f, m))
        return out

    def layout(self, name: str) -> tuple[int, int, list]:
        """C layout on 64-bit targets: (size, align, [(field, offset)])."""
        if name in self.layouts:
            return self.layouts[name]
        offset, align, placed = 0, 1, []
        for f in self.struct_fields(name):
            size, a = f[3], f[4]
            offset = (offset + a - 1) // a * a
            placed.append((f, offset))
            offset += size
            align = max(align, a)
        size = (offset + align - 1) // align * align
        self.layouts[name] = (size, align, placed)
        return self.layouts[name]

    # Procedures --------------------------------------------------------------

    def functions(self) -> list[dict]:
        """Every exported function: {name, params[(name, jai, C)], ret(jai, C), ...}."""
        out = []
        for f in self.doc['functions']:
            out.append(self.function('wgpu' + pascal(f['name']), None, f))
        for s in self.doc['structs']:
            if s.get('free_members'):
                out.append({'name': f'wgpu{pascal(s["name"])}FreeMembers', 'self': None, 'method': None,
                            'params': [(camel(s['name']), ctype(s['name']), ctype(s['name']))],
                            'ret': ('', 'void'), 'spec': None, 'free': s['name']})
        for o in self.doc['objects']:
            this = (camel(o['name']), ctype(o['name']), ctype(o['name']))
            for m in o.get('methods') or []:
                out.append(self.function(f'wgpu{pascal(o["name"])}{pascal(m["name"])}', o['name'], m, this))
            for ref in ('add_ref', 'release'):
                out.append({'name': f'wgpu{pascal(o["name"])}{pascal(ref)}', 'self': o['name'], 'method': ref,
                            'params': [this], 'ret': ('', 'void'), 'spec': None})
        return out

    def function(self, name: str, obj: str | None, f: dict, this=None) -> dict:
        params = [this] if this else []
        for a in f.get('args') or []:
            params += [(n, jai, c) for n, jai, c, _, _ in self.fields(a)]
        if f.get('callback'):
            cb = f['callback'].split('.', 1)[1]
            params.append(('callbackInfo', ctype(cb) + 'CallbackInfo', ctype(cb) + 'CallbackInfo'))
            ret = ('WGPUFuture', 'WGPUFuture')
        elif f.get('returns'):
            r = f['returns']
            (_, jai, c, _, _), = self.fields({**r, 'name': 'r'})
            ret = (jai, c)
        else:
            ret = ('', 'void')
        return {'name': name, 'self': obj, 'method': f['name'], 'params': params, 'ret': ret, 'spec': f}


# ---------------------------------------------------------------------------
# Jai output
# ---------------------------------------------------------------------------

def constant_value(spec: Spec, value: str) -> tuple[str, str]:
    """(jai type, jai expression) of a constant."""
    return {
        'uint32_max': ('u32', '0xFFFF_FFFF'),
        'uint64_max': ('u64', '0xFFFF_FFFF_FFFF_FFFF'),
        'usize_max': ('u64', '0xFFFF_FFFF_FFFF_FFFF'),
        'nan': ('float32', 'WGPU_NAN32'),
    }[value]


def member_default(spec: Spec, m: dict | None, jai: str) -> str | None:
    if m is None:
        return None
    d = m.get('default')
    t = m['type']
    if t == 'nullable_string':
        return '.{ null, WGPU_STRLEN }'
    if d is None:
        return None
    d = str(d)
    if d.startswith('constant.'):
        return 'WGPU_' + d.split('.', 1)[1].upper()
    if t.startswith('enum.'):
        return '.' + jai_ident(pascal(d))
    if t.startswith('bitflag.'):
        return '.' + jai_ident(pascal(d))
    if t == 'bool':
        return {'false': '0', 'true': '1'}[d]
    if t.startswith('struct.') and d == 'zero':
        return None
    if re.fullmatch(r'-?(0x[0-9A-Fa-f]+|\d+(\.\d+)?)', d):
        return d
    raise ValueError(f'unknown default {d!r} for {m["name"]}')


def jai_param_type(jai: str) -> str:
    return jai


def gen_jai(spec: Spec, rev: str, digest: str) -> str:
    o = []
    w = o.append
    w(f'// Generated by tools/webgpu_gen.py from webgpu-native/webgpu-headers {rev}')
    w(f'// (webgpu.yml sha256 {digest}). Do not edit; see docs/stdlib/webgpu.md.')
    w('')
    w('WGPUFlags :: u64;')
    w('WGPUBool :: u32;')
    w('')
    for c in spec.doc['constants']:
        t, v = constant_value(spec, c['value'])
        w(f'WGPU_{c["name"].upper()}: {t} : {v};')
    w('')
    w('WGPUStringView :: struct {')
    w('    data: *u8;')
    w('    length: u64 = WGPU_STRLEN;')
    w('}')
    w('')
    w('WGPUChainedStruct :: struct {')
    w('    next: *WGPUChainedStruct;')
    w('    sType: WGPUSType;')
    w('}')
    w('')
    for name in spec.objects:
        w(f'{ctype(name)}Impl :: struct {{}}')
        w(f'{ctype(name)} :: *{ctype(name)}Impl;')
    w('')
    for name in spec.enums:
        w(f'{ctype(name)} :: enum u32 {{')
        for entry, value in spec.enum_values(name):
            w(f'    {jai_ident(pascal(entry))} :: 0x{value:08X};')
        w('}')
        w('')
    for name in spec.bitflags:
        w(f'{ctype(name)} :: enum_flags u64 {{')
        for entry, value in spec.bitflag_values(name):
            w(f'    {jai_ident(pascal(entry))} :: 0x{value:X};')
        w('}')
        w('')
    for name, cb in spec.callbacks.items():
        params = []
        for a in cb.get('args') or []:
            params += [f'{n}: {jai}' for n, jai, _, _, _ in spec.fields(a)]
        params += ['userdata1: *void', 'userdata2: *void']
        w(f'{ctype(name)}Callback :: #type ({", ".join(params)}) #c_call;')
    w('')
    for name, cb in spec.callbacks.items():
        w(f'{ctype(name)}CallbackInfo :: struct {{')
        w('    nextInChain: *WGPUChainedStruct;')
        if cb['style'] == 'callback_mode':
            w('    mode: WGPUCallbackMode;')
        w(f'    callback: {ctype(name)}Callback;')
        w('    userdata1: *void;')
        w('    userdata2: *void;')
        w('}')
        w('')
    for name, s in spec.structs.items():
        w(f'{ctype(name)} :: struct {{')
        for fname, jai, _, _, _, m in spec.struct_fields(name):
            default = None
            if fname == 'chain':
                default = f'.{{ null, .{pascal(name)} }}'
            elif m is not None and fname == camel(m['name']):
                default = member_default(spec, m, jai)
            w(f'    {jai_ident(fname)}: {jai}' + (f' = {default};' if default else ';'))
        w('}')
        w('')
    # On the browser sandbox these are Jai procedures in generated_wasm.jai.
    w('#if OS != .WASM {')
    for f in spec.functions():
        if f['name'] in JAI_SIDE_ON_WASM:
            w('    ' + proc_decl(f))
    w('}')
    w('')
    for f in spec.functions():
        if f['name'] not in JAI_SIDE_ON_WASM:
            w(proc_decl(f))
    w('')
    w('#scope_file')
    w('wgpu_nan32 :: () -> float32 {')
    w('    bits: u32 = 0x7FC0_0000;')
    w('    return (cast(*float32) *bits).*;')
    w('}')
    w('')
    w('WGPU_NAN32 :: #run wgpu_nan32();')
    return '\n'.join(o) + '\n'


JAI_SIDE_ON_WASM = {'wgpuInstanceProcessEvents', 'wgpuInstanceWaitAny'}


def proc_decl(f: dict) -> str:
    params = ', '.join(f'{jai_ident(n)}: {jai}' for n, jai, _ in f['params'])
    ret = f' -> {f["ret"][0]}' if f['ret'][0] else ''
    return f'{f["name"]} :: ({params}){ret} #foreign libwgpu;'


def gen_jai_wasm(spec: Spec, rev: str) -> str:
    """Callback dispatch for the browser: the JS host queues finished callbacks, Jai calls them."""
    o = []
    w = o.append
    w(f'// Generated by tools/webgpu_gen.py from webgpu-native/webgpu-headers {rev}. Do not edit.')
    w('')
    w('webgpu_dispatch_record :: (r: *Webgpu_Callback_Record) {')
    w('    message := WGPUStringView.{ r.message_data, r.message_length };')
    w('    if r.kind == {')
    for i, (name, cb) in enumerate(spec.callbacks.items(), 1):
        w(f'        case {i}; // {name}')
        w(f'            callback := cast({ctype(name)}Callback) r.callback;')
        args, slot = [], 0
        for a in cb.get('args') or []:
            t = a['type']
            if t in STRINGS:
                args.append('message')
                continue
            (_, jai, _, _, _), = spec.fields(a)
            if a.get('pointer'):
                args.append(f'cast({jai}) *r.args[{slot}]' if t.startswith('object.') else f'cast({jai}) r.args[{slot}]')
            else:
                args.append(f'cast({jai}) r.args[{slot}]')
            slot += 1
        args += ['r.userdata1', 'r.userdata2']
        w(f'            callback({", ".join(args)});')
    w('    }')
    w('}')
    return '\n'.join(o) + '\n'


# ---------------------------------------------------------------------------
# JavaScript output: layouts and call descriptors for the browser host
# ---------------------------------------------------------------------------

def js_kind(spec: Spec, t: str, pointer: str | None):
    if t in STRINGS:
        return STRINGS[t]
    if t in PRIMS:
        return 'ptr' if t == 'c_void' else PRIMS[t][3]
    kind, _, name = t.partition('.')
    if kind == 'enum':
        return ['enum', pascal(name)]
    if kind == 'bitflag':
        return 'flags'
    if kind == 'object':
        return ['objptr', pascal(name)] if pointer else ['obj', pascal(name)]
    if kind == 'struct':
        return ['ptr', pascal(name)] if pointer else ['struct', pascal(name)]
    if kind == 'callback':
        return ['callback', pascal(name)]
    raise ValueError(t)


def sentinel(spec: Spec, m: dict):
    """The value that means `undefined` in JS (the member is left out)."""
    d = str(m.get('default'))
    if d.startswith('constant.'):
        c = d.split('.', 1)[1]
        if c.endswith('_undefined') or c == 'whole_size':
            v = spec.constants[c]['value']
            return {'uint32_max': 0xFFFFFFFF, 'uint64_max': 'max64', 'usize_max': 'max64', 'nan': 'nan'}[v]
    return None


def gen_js(spec: Spec, rev: str, digest: str) -> str:
    enums = {}
    for name in spec.enums:
        table = {}
        for entry, value in spec.enum_values(name):
            if entry == 'undefined':
                continue
            if entry == 'binding_not_used':
                table[value] = {'omitParent': True}
            elif name == 'optional_bool':
                table[value] = entry == 'true'
            else:
                table[value] = js_enum_string(name, entry)
        enums[pascal(name)] = table
    structs = {}
    for name, s in spec.structs.items():
        size, _, placed = spec.layout(name)
        members = []
        by_name = {f[0]: off for f, off in placed}
        for (fname, _, _, _, _, m), off in placed:
            if m is None:
                if fname == 'nextInChain':
                    members.append(['', off, 'chain'])
                continue
            if fname != camel(m['name']):
                continue  # array counts are read with their arrays
            t = m['type']
            if t.startswith('array<'):
                inner = t[len('array<'):-1]
                count_off = by_name[camel(singular(m['name'])) + 'Count']
                members.append([fname, off, ['array', js_kind(spec, inner, None), count_off]])
                continue
            entry = [fname, off, js_kind(spec, t, m.get('pointer'))]
            sv = sentinel(spec, m)
            if sv is not None:
                entry.append(sv)
            members.append(entry)
        info = {'size': size, 'members': members}
        if s['type'] == 'extension':
            info['sType'] = dict(spec.enum_values('s_type'))[name]
        structs[pascal(name)] = info
    callbacks = {}
    for i, (name, cb) in enumerate(spec.callbacks.items(), 1):
        callbacks[pascal(name)] = {
            'id': i,
            'mode': cb['style'] == 'callback_mode',
            'args': [js_kind(spec, a['type'], a.get('pointer')) for a in cb.get('args') or []],
        }
    functions = {}
    for f in spec.functions():
        d = {}
        if f['self']:
            d['self'] = pascal(f['self'])
        if f['method']:
            d['method'] = camel(f['method'])
        if f.get('free'):
            d['free'] = pascal(f['free'])
        args = []
        fs = f['spec']
        undef = {}
        for a in (fs.get('args') or []) if fs else []:
            t = a['type']
            sv = sentinel(spec, a)
            if sv is not None:
                undef[len(args)] = sv
            if t.startswith('array<'):
                args.append(['array', js_kind(spec, t[len('array<'):-1], None)])
            else:
                args.append(js_kind(spec, t, a.get('pointer')))
        if f.get('free'):
            args.append(['struct', pascal(f['free'])])
        d['args'] = args
        if undef:
            d['undef'] = undef
        if fs and fs.get('callback'):
            d['callback'] = pascal(fs['callback'].split('.', 1)[1])
        elif fs and fs.get('returns'):
            d['ret'] = js_kind(spec, fs['returns']['type'], fs['returns'].get('pointer'))
        functions[f['name']] = d
    out = [
        f'// Generated by tools/webgpu_gen.py from webgpu-native/webgpu-headers {rev}',
        f'// (webgpu.yml sha256 {digest}). Do not edit; see docs/stdlib/webgpu.md.',
        '// Struct members: [jsName, offset, kind, sentinel?]. Function args are in C order; arrays',
        '// take two slots (count, pointer), by-value structs a pointer, and a callback-taking function',
        '// gets its callback info pointer and the WGPUFuture result pointer last.',
        f'export const REVISION = {json.dumps(rev)};',
        f'export const ENUMS = {json.dumps(enums, separators=(",", ":"))};',
        f'export const STRUCTS = {json.dumps(structs, separators=(",", ":"))};',
        f'export const CALLBACKS = {json.dumps(callbacks, separators=(",", ":"))};',
        f'export const FUNCTIONS = {json.dumps(functions, separators=(",", ":"))};',
    ]
    text = '\n'.join(out) + '\n'
    # One entry per line keeps diffs between header revisions readable.
    return re.sub(r'(?<=[{,])("(?:[A-Za-z0-9]+|wgpu[A-Za-z0-9]+)":\{)', r'\n  \1', text)


# ---------------------------------------------------------------------------
# Header check
# ---------------------------------------------------------------------------

def check_header(spec: Spec, header: str) -> int:
    text = re.sub(r'/\*.*?\*/', '', header, flags=re.S)
    text = re.sub(r'//[^\n]*', '', text)
    text = text.replace('WGPU_NULLABLE ', '')
    problems = 0

    def norm(s: str) -> str:
        return re.sub(r'\s+', ' ', s.replace('*', ' * ').replace('struct ', '')).strip()

    for m in re.finditer(r'typedef struct (WGPU\w+) \{(.*?)\} \1', text, re.S):
        name = m.group(1)
        got = [norm(l) for l in m.group(2).split(';') if l.strip()]
        key = next((k for k in spec.structs if ctype(k) == name), None)
        if key is None:
            continue
        want = [norm(f'{c} {n}') for n, _, c, _, _, _ in spec.struct_fields(key)]
        want = [w.replace('WGPUChainedStruct chain', 'WGPUChainedStruct chain') for w in want]
        if got != want:
            problems += 1
            print(f'struct {name}:\n  header {got}\n  ours   {want}')
    for m in re.finditer(r'WGPU_EXPORT (.*?) (wgpu\w+)\((.*?)\) WGPU_FUNCTION_ATTRIBUTE;', text):
        ret, name, params = norm(m.group(1)), m.group(2), m.group(3)
        f = next((f for f in spec.functions() if f['name'] == name), None)
        if f is None:
            if name != 'wgpuGetProcAddress':
                problems += 1
                print(f'function {name}: missing')
            continue
        got = [norm(p) for p in params.split(',')] if params.strip() not in ('', 'void') else []
        want = [norm(f'{c} {n}') for n, _, c in f['params']]
        if got != want or ret != norm(f['ret'][1]):
            problems += 1
            print(f'function {name}:\n  header {ret} {got}\n  ours   {f["ret"][1]} {want}')
    ours = {f['name'] for f in spec.functions()}
    theirs = set(re.findall(r'WGPU_EXPORT .*? (wgpu\w+)\(', text))
    for name in sorted(ours - theirs):
        problems += 1
        print(f'function {name}: not in header')
    values = dict(re.findall(r'(WGPU\w+_\w+) = (0x[0-9A-Fa-f]+)', text))
    for name in spec.enums:
        for entry, value in spec.enum_values(name):
            key = f'{ctype(name)}_{pascal(entry)}'
            if int(values.get(key, '-1'), 0) != value:
                problems += 1
                print(f'enum {key}: header {values.get(key)} ours {value:#x}')
    flags = dict(re.findall(r'static const (?:WGPU\w+) (WGPU\w+_\w+) = (0x[0-9A-Fa-f]+)', text))
    for name in spec.bitflags:
        for entry, value in spec.bitflag_values(name):
            key = f'{ctype(name)}_{pascal(entry)}'
            if int(flags.get(key, '-1'), 0) != value:
                problems += 1
                print(f'flag {key}: header {flags.get(key)} ours {value:#x}')
    for m in re.finditer(r'typedef void \(\*(WGPU\w+)Callback\)\((.*?)\) WGPU_FUNCTION_ATTRIBUTE;', text):
        key = next((k for k in spec.callbacks if ctype(k) == m.group(1)), None)
        if key is None:
            continue
        got = [norm(p) for p in m.group(2).split(',')]
        want = []
        for a in spec.callbacks[key].get('args') or []:
            want += [norm(f'{c} {n}') for n, _, c, _, _ in spec.fields(a)]
        want += ['void * userdata1', 'void * userdata2']
        if got != want:
            problems += 1
            print(f'callback {m.group(1)}:\n  header {got}\n  ours   {want}')
    print(f'header check: {problems} problem(s)')
    return problems


# ---------------------------------------------------------------------------

def shared_root() -> Path:
    common = subprocess.run(['git', 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                            cwd=ROOT, capture_output=True, text=True)
    return Path(common.stdout.strip()).parent if common.returncode == 0 else ROOT


def load_yml(path: str | None) -> tuple[str, str, str]:
    pin = json.loads(MANIFEST.read_text())['webgpu_headers']
    rev, digest = pin['revision'], pin['files']['webgpu.yml']
    if path:
        data = Path(path).read_bytes()
    else:
        cache = shared_root() / 'artifacts' / 'webgpu' / rev / 'webgpu.yml'
        if cache.exists() and hashlib.sha256(cache.read_bytes()).hexdigest() == digest:
            data = cache.read_bytes()
        else:
            url = f'https://raw.githubusercontent.com/{pin["repository"]}/{rev}/webgpu.yml'
            data = urllib.request.urlopen(url, timeout=60).read()
            cache.parent.mkdir(parents=True, exist_ok=True)
            cache.write_bytes(data)
    actual = hashlib.sha256(data).hexdigest()
    if actual != digest:
        sys.exit(f'webgpu.yml: sha256 {actual}, expected {digest} (tools/webgpu.json)')
    return data.decode(), rev, digest


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--yml', help='a local copy of the pinned webgpu.yml')
    ap.add_argument('--check-header', metavar='WEBGPU_H', help='compare with webgpu.h and exit')
    args = ap.parse_args()
    text, rev, digest = load_yml(args.yml)
    spec = Spec(parse_yaml(text))
    if args.check_header:
        return 1 if check_header(spec, Path(args.check_header).read_text()) else 0
    OUT_JAI.parent.mkdir(parents=True, exist_ok=True)
    OUT_JAI.write_text(gen_jai(spec, rev, digest))
    OUT_JAI_WASM.write_text(gen_jai_wasm(spec, rev))
    OUT_JS.write_text(gen_js(spec, rev, digest))
    for p in (OUT_JAI, OUT_JAI_WASM, OUT_JS):
        print(f'wrote {p.relative_to(ROOT)} ({len(p.read_text().splitlines())} lines)')
    return 0


if __name__ == '__main__':
    sys.exit(main())
