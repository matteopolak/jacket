#!/usr/bin/env python3
"""Generate corpus-shaped Jai programs of a requested size for compile-speed benchmarks.

    python3 tools/benchgen.py --lines 60000 --seed 1 --out /tmp/gen60k [--files 8] [--shape tools/corpus-shape.json]

Unlike `tools/jaigen.py` (a differential tester that stresses semantics), this generator stresses the
compiler's throughput on code that looks like real programs: procedure bodies, struct and enum
declarations, control flow, polymorphism and compile-time code are sampled from the distributions that
`tools/jaistats.py` measured on the upstream corpus (`tools/corpus-shape.json`). The output is
deterministic for a seed, uses only `Basic`, `Math`, `String` and `Hash_Table`, links, and runs: `main`
calls a driver per group of procedures, folds every result into a checksum and prints it, so nothing is
dead. Compilers must print the same checksum.

`--out DIR` writes `main.jai` (plus `part_N.jai` files loaded by it when `--files` is above 1). The line
count is for the whole output including blank lines; roughly 85% of the lines are code.
"""
from __future__ import annotations

import argparse
import json
import random
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INT_TYPES = ['int', 's32', 'u32', 'u8', 'u16', 's64']
VERBS = ['update', 'compute', 'process', 'build', 'scan', 'merge', 'reduce', 'fold', 'check', 'find', 'apply',
         'collect', 'resolve', 'emit', 'visit', 'parse', 'step', 'mix', 'pack', 'walk']
NOUNS = ['node', 'entry', 'token', 'chunk', 'frame', 'state', 'item', 'range', 'block', 'table', 'cell',
         'event', 'layer', 'queue', 'slot', 'span', 'value', 'record', 'group', 'path']
MASK = 7  # fixed arrays hold 8 elements; indexes are `& 7`


def pick(rng: random.Random, weights: dict):
    return rng.choices(list(weights), weights=list(weights.values()))[0]


def bucket(rng: random.Random, edges: list[int], share: list[float]) -> int:
    k = rng.choices(range(len(edges)), weights=share)[0]
    hi = edges[k + 1] - 1 if k + 1 < len(edges) else edges[k] * 2
    return rng.randint(edges[k], max(edges[k], hi))


class Var:
    def __init__(self, name, kind, ty=None, size=0):
        # kind: int float bool string struct enum arr dyn; ty: declared type name
        self.name, self.kind, self.ty, self.size = name, kind, ty, size
        self.mutable = True


class Struct:
    def __init__(self, name, fields):
        self.name, self.fields = name, fields  # fields: [(name, kind, ty, size)]


class Proc:
    def __init__(self, name, params, ret, cost):
        self.name, self.params, self.ret, self.cost = name, params, ret, cost


class Gen:
    def __init__(self, seed: int, shape: dict, trace: bool = False):
        self.trace = trace
        self.r = random.Random(seed)
        self.shape = shape
        self.n = 0
        self.structs: list[Struct] = []
        self.enums: dict[str, list[str]] = {}
        self.procs: list[Proc] = []
        self.poly_boxes: list[str] = []
        self.poly_procs: list[tuple[str, str]] = []  # (name, kind)
        self.runs: list[str] = []
        self.lines: list[str] = []
        self.flow = shape['flow_per_statement']

    def uid(self, prefix):
        self.n += 1
        return f'{prefix}_{self.n}'

    def chance(self, p):
        return self.r.random() < p

    # ---- declarations -------------------------------------------------------------------------
    def field_kind(self):
        k = pick(self.r, {'int': 22, 's32': 14, 'u32': 14, 'u8': 14, 'u16': 7, 'bool': 13, 'float64': 10,
                          'string': 8, 'enum': 4, 'arr': 4, 'struct': 4})
        if k == 'enum' and not self.enums: k = 'int'
        if k == 'struct' and not self.structs: k = 'int'
        return k

    def emit_enum(self):
        name = self.uid(self.r.choice(NOUNS).capitalize() + '_Kind')
        count = max(2, min(40, int(self.r.lognormvariate(1.8, 0.8))))
        members = [f'{self.r.choice(["A", "B", "K", "M"])}{i}' for i in range(count)]
        base = '' if count < 200 else ' u16'
        self.lines += [f'{name} :: enum{base} {{'] + [f'    {m};' for m in members] + ['}', '']
        self.enums[name] = members

    def emit_struct(self):
        sh = self.shape['struct_fields_hist']
        count = max(1, bucket(self.r, sh['edges'], sh['share']))
        name = self.uid(self.r.choice(NOUNS).capitalize())
        fields, lines, names = [], [f'{name} :: struct {{'], set()
        for _ in range(count):
            fname = f'{self.r.choice("abcdefghijklmnopqrstuvw")}{len(fields)}{self.r.choice(["x", "id", "n", "_v"])}'
            if fname in names: continue
            names.add(fname)
            k = self.field_kind()
            if k in INT_TYPES or k == 'int':
                fields.append((fname, 'int', k, 0))
                init = f' = {self.r.randint(0, 9)}' if self.chance(0.3) else ''
                lines.append(f'    {fname}: {k}{init};')
            elif k == 'bool':
                fields.append((fname, 'bool', 'bool', 0)); lines.append(f'    {fname}: bool;')
            elif k == 'float64':
                fields.append((fname, 'float', 'float64', 0)); lines.append(f'    {fname}: float64 = {self.r.randint(0, 9)}.5;')
            elif k == 'string':
                fields.append((fname, 'string', 'string', 0)); lines.append(f'    {fname}: string;')
            elif k == 'enum':
                e = self.r.choice(list(self.enums)); fields.append((fname, 'enum', e, 0)); lines.append(f'    {fname}: {e};')
            elif k == 'arr':
                fields.append((fname, 'arr', 'int', 8)); lines.append(f'    {fname}: [8] int;')
            else:
                s = self.r.choice(self.structs); fields.append((fname, 'struct', s.name, 0)); lines.append(f'    {fname}: {s.name};')
        if not fields:
            fields.append(('v', 'int', 'int', 0)); lines.append('    v: int;')
        self.lines += lines + ['}', '']
        self.structs.append(Struct(name, fields))

    def emit_poly_struct(self):
        name = self.uid('Box')
        self.lines += [f'{name} :: struct(T: Type) {{', '    items: [8] T;', '    count: int;', '}', '']
        push, total = self.uid('box_push'), self.uid('box_sum')
        self.lines += [f'{push} :: (b: *{name}($T), v: T) {{', '    b.items[b.count & 7] = v;', '    b.count += 1;', '}', '',
                       f'{total} :: (b: {name}($T)) -> T {{', '    t: T;', '    for 0..7 t += b.items[it];', '    return t;', '}', '']
        self.poly_boxes.append((name, push, total))

    def emit_poly_proc(self):
        name = self.uid(self.r.choice(['pick', 'blend', 'clampv', 'scale']))
        form = self.r.randint(0, 2)
        if form == 0:
            body = ['    if a > b return a;', '    return b;']; sig = '(a: $T, b: T) -> T'
        elif form == 1:
            body = ['    r := a;', '    for 1..3 r = r + b;', '    return r;']; sig = '(a: $T, b: T) -> T'
        else:
            body = ['    t: T;', '    t = a * b + a;', '    return t;']; sig = '(a: $T, b: T) -> T'
        self.lines += [f'{name} :: {sig} {{'] + body + ['}', '']
        self.poly_procs.append((name, 'num'))

    def emit_run_table(self):
        name = self.uid('TABLE')
        fn = self.uid('make_table')
        n = self.r.choice([16, 32, 64])
        self.lines += [f'{fn} :: () -> [{n}] int {{', f'    t: [{n}] int;', f'    for i: 0..{n - 1} t[i] = (i * {self.r.randint(3, 31)} + {self.r.randint(1, 9)}) ^ (i >> 1);',
                       '    return t;', '}', '', f'{name} :: #run {fn}();', '']
        self.runs.append(name)
        return n

    def emit_insert(self):
        name = self.uid('gen')
        k = self.r.randint(2, 6)
        body = ' '.join(f'ins_{name}_{i} :: (x: int) -> int {{ return x * {i + 2} + {i}; }}' for i in range(k))
        self.lines += [f'#insert "{body}";', '']
        self.insert_procs = getattr(self, 'insert_procs', []) + [f'ins_{name}_{i}' for i in range(k)]

    def emit_macro(self):
        name = self.uid('bump')
        self.lines += [f'{name} :: (n: int) #expand {{', '    `local_acc += n * 3;', '    `local_acc ^= n;', '}', '']
        self.macros = getattr(self, 'macros', []) + [name]

    # ---- expressions --------------------------------------------------------------------------
    def leaves(self, v: Var, kind):
        """(access path, type) for every field of struct variable v with the wanted kind."""
        out = []
        def walk(prefix, s: Struct, depth):
            for fname, fk, ty, size in s.fields:
                if fk == 'struct' and depth < 2:
                    walk(f'{prefix}.{fname}', next(x for x in self.structs if x.name == ty), depth + 1)
                elif fk == kind:
                    out.append((f'{prefix}.{fname}', ty, size))
        walk(v.name, next(x for x in self.structs if x.name == v.ty), 0)
        return out

    def int_atoms(self, scope):
        atoms = []
        for v in scope:
            if v.kind == 'int':
                atoms.append(v.name if v.ty == 'int' else f'(cast(int) {v.name})')
            elif v.kind == 'struct':
                for path, ty, _ in self.leaves(v, 'int'):
                    atoms.append(path if ty == 'int' else f'(cast(int) {path})')
                for path, ty, _ in self.leaves(v, 'arr'):
                    atoms.append(f'{path}[{self.r.randint(0, MASK)}]')
            elif v.kind in ('arr', 'dyn') and v.kind == 'arr':
                atoms.append(f'{v.name}[{self.r.randint(0, MASK)}]')
            elif v.kind == 'string':
                atoms.append(f'{v.name}.count')
            elif v.kind == 'enum':
                atoms.append(f'(cast(int) {v.name})')
        return atoms

    def iexpr(self, scope, depth=0):
        r = self.r
        ops = self.shape['binops_per_statement']['hist']
        if depth == 0:
            n = r.choices(range(len(ops)), weights=ops)[0]
            n = n if n else (1 if self.chance(0.3) else 0)
        else:
            n = 0
        atoms = self.int_atoms(scope)
        def atom():
            if atoms and not self.chance(0.2):
                return r.choice(atoms)
            return str(r.randint(0, 99))
        e = atom()
        last = '+'
        for _ in range(n):
            op = pick(r, {'+': 25, '-': 15, '*': 10, '&': 8, '|': 4, '^': 6, '/': 5, '%': 6, '<<': 3, '>>': 4})
            if op in '/%':
                rhs = str(r.randint(1, 97))
            elif op in ('<<', '>>'):
                rhs = str(r.randint(1, 5))
            else:
                rhs = atom()
            if op in '+-' and last in '+-' and self.chance(0.5):
                e = f'{e} {op} {rhs}'
            else:
                e = f'({e} {op} {rhs})'
            last = op
        return e

    def fexpr(self, scope):
        fl = []
        for v in scope:
            if v.kind == 'float': fl.append(v.name)
            elif v.kind == 'struct': fl += [p for p, _, _ in self.leaves(v, 'float')]
        a = self.r.choice(fl) if fl and not self.chance(0.25) else f'{self.r.randint(0, 20)}.25'
        b = f'cast(float64) (({self.iexpr(scope, 1)}) & 255)' if self.chance(0.5) else f'{self.r.randint(1, 9)}.5'
        op = pick(self.r, {'+': 4, '-': 3, '*': 3})
        return f'{a} {op} {b}'

    def bexpr(self, scope, depth=0):
        r = self.r
        form = r.choices(['cmp', 'flag', 'and', 'or', 'str'], weights=[60, 14, 8, 8, 5])[0]
        bools = [v.name for v in scope if v.kind == 'bool'] + [p for v in scope if v.kind == 'struct' for p, _, _ in self.leaves(v, 'bool')]
        strs = [v.name for v in scope if v.kind == 'string']
        if form == 'flag' and bools: return ('!' if self.chance(0.3) else '') + r.choice(bools)
        if form in ('and', 'or') and depth < 1:
            return f'{self.bexpr(scope, 1)} {"&&" if form == "and" else "||"} {self.bexpr(scope, 1)}'
        if form == 'str' and strs:
            return pick(r, {f'{r.choice(strs)} == "ab"': 1, f'contains({r.choice(strs)}, "b")': 1, f'begins_with({r.choice(strs)}, "a")': 1})
        op = r.choice(['<', '>', '<=', '>=', '==', '!='])
        return f'{self.iexpr(scope, 1)} {op} {self.iexpr(scope, 1)}'

    def sexpr(self, scope):
        strs = [v.name for v in scope if v.kind == 'string']
        base = self.r.choice(strs) if strs and self.chance(0.6) else f'"s{self.r.randint(0, 99)}"'
        return pick(self.r, {base: 3, f'tprint("%-%", {base}, {self.iexpr(scope, 1)})': 3, f'tprint("%:%", {self.iexpr(scope, 1)}, {base})': 1})

    # ---- statements ---------------------------------------------------------------------------
    def stmts(self, ctx, target, out, indent, depth):
        """Append statements to `out` until at least `target` lines were added."""
        start = len(out)
        while len(out) - start < target:
            self.stmt(ctx, out, indent, depth)

    def stmt(self, ctx, out, indent, depth):
        r, pad = self.r, '    ' * indent
        scope = ctx['scope']
        f = self.flow
        nest_ok = depth < 3
        weights = {'assign': 38, 'decl': 14, 'call': 11, 'if': 100 * f['if'] if nest_ok else 0,
                   'for': 100 * f['for'] * 0.9 if nest_ok else 0, 'while': 100 * f['while'] if nest_ok else 0,
                   'switch': 100 * (f['switch'] + f['case'] * 0.15) if nest_ok else 0,
                   'string': 3, 'array': 2, 'ret': 100 * f['return'] * 0.4 if ctx['ret'] != 'void' and depth > 0 else 0,
                   'macro': 1 if getattr(self, 'macros', None) and ctx.get('macro') else 0,
                   'defer': 0.7 if depth == 0 and ctx.get('allow_defer') else 0,
                   'lit': 1.5, 'poly': 3 if self.poly_procs else 0, 'cast': 4}
        kind = pick(r, weights)
        if kind == 'assign':
            self.assign(ctx, out, pad)
        elif kind == 'decl':
            name = self.uid('v')
            t = pick(r, {'int': 60, 'float': 12, 'bool': 12, 'string': 16})
            if t == 'int':
                ty = pick(r, {'int': 50, 's32': 18, 'u32': 14, 'u8': 8, 'u16': 4, 's64': 6})
                if ty == 'int': out.append(f'{pad}{name} := {self.iexpr(scope)};')
                else: out.append(f'{pad}{name}: {ty} = cast,trunc({ty}) ({self.iexpr(scope)});')
                scope.append(Var(name, 'int', ty))
            elif t == 'float':
                out.append(f'{pad}{name} := {self.fexpr(scope)};'); scope.append(Var(name, 'float', 'float64'))
            elif t == 'bool':
                out.append(f'{pad}{name} := {self.bexpr(scope)};'); scope.append(Var(name, 'bool', 'bool'))
            else:
                out.append(f'{pad}{name} := {self.sexpr(scope)};'); scope.append(Var(name, 'string', 'string'))
        elif kind == 'call':
            self.call(ctx, out, pad)
        elif kind == 'if':
            cond = self.bexpr(scope)
            if self.chance(0.35) and ctx['ret'] == 'void':
                out.append(f'{pad}if {cond} {ctx["acc"]} += {self.iexpr(scope, 1)};')
                return
            if self.chance(0.25) and ctx['ret'] != 'void':
                out.append(f'{pad}if {cond} return {self.ret_expr(ctx)};')
                return
            out.append(f'{pad}if {cond} {{')
            mark = len(scope)
            self.stmts(ctx, r.randint(1, 4), out, indent + 1, depth + 1)
            del scope[mark:]
            if self.chance(self.flow['else'] / max(self.flow['if'], 0.01) * 1.2):
                out.append(f'{pad}}} else {{')
                self.stmts(ctx, r.randint(1, 3), out, indent + 1, depth + 1)
                del scope[mark:]
            out.append(f'{pad}}}')
        elif kind == 'for':
            self.loop(ctx, out, pad, indent, depth)
        elif kind == 'while':
            name = self.uid('w'); bound = r.randint(2, 5)
            out.append(f'{pad}{name} := 0;')
            out.append(f'{pad}while {name} < {bound} {{')
            ctx['mult'] *= bound
            mark = len(scope)
            self.stmts(ctx, r.randint(1, 3), out, indent + 1, depth + 1)
            del scope[mark:]
            ctx['mult'] //= bound
            out.append(f'{pad}    {name} += 1;')
            out.append(f'{pad}}}')
        elif kind == 'switch':
            enums = [v for v in scope if v.kind == 'enum']
            if enums and self.chance(0.6):
                v = r.choice(enums); members = self.enums[v.ty]
                out.append(f'{pad}if {v.name} == {{')
                for m in r.sample(members, min(len(members), r.randint(2, 4))):
                    out.append(f'{pad}    case .{m};')
                    mark = len(scope)
                    self.stmts(ctx, r.randint(1, 2), out, indent + 2, depth + 1)
                    del scope[mark:]
                out.append(f'{pad}    case;')
                mark = len(scope)
                self.stmts(ctx, 1, out, indent + 2, depth + 1)
                del scope[mark:]
                out.append(f'{pad}}}')
            else:
                out.append(f'{pad}if ({self.iexpr(scope, 1)}) & 3 == {{')
                for k in range(r.randint(2, 4)):
                    out.append(f'{pad}    case {k};')
                    mark = len(scope)
                    self.stmts(ctx, r.randint(1, 2), out, indent + 2, depth + 1)
                    del scope[mark:]
                out.append(f'{pad}    case;')
                mark = len(scope)
                self.stmts(ctx, 1, out, indent + 2, depth + 1)
                del scope[mark:]
                out.append(f'{pad}}}')
            # statements inside a case must not leak their variables
        elif kind == 'string':
            strs = [v for v in scope if v.kind == 'string' and v.mutable]
            tgt = r.choice(strs).name if strs else None
            if tgt: out.append(f'{pad}{tgt} = {self.sexpr(scope)};')
            else: self.assign(ctx, out, pad)
        elif kind == 'array':
            arrs = [v for v in scope if v.kind in ('arr', 'dyn')]
            if arrs:
                v = r.choice(arrs)
                if v.kind == 'arr': out.append(f'{pad}{v.name}[({self.iexpr(scope, 1)}) & {MASK}] = {self.iexpr(scope, 1)};')
                else: out.append(f'{pad}array_add(*{v.name}, {self.iexpr(scope, 1)});')
            else: self.assign(ctx, out, pad)
        elif kind == 'ret':
            out.append(f'{pad}if {self.bexpr(scope)} return {self.ret_expr(ctx)};')
        elif kind == 'macro':
            out.append(f'{pad}{r.choice(self.macros)}({self.iexpr(scope, 1)});')
        elif kind == 'defer':
            out.append(f'{pad}defer {ctx["acc"]} += 1;')
            ctx['allow_defer'] = False
        elif kind == 'lit':
            name = self.uid('tbl')
            vals = ', '.join(str(r.randint(0, 200)) for _ in range(r.randint(3, 8)))
            out.append(f'{pad}{name} := int.[{vals}];')
            out.append(f'{pad}for {name} {ctx["acc"]} += it;')
        elif kind == 'poly':
            name, _ = r.choice(self.poly_procs)
            ty = pick(r, {'int': 5, 's32': 2, 'float64': 3})
            if ty == 'float64':
                out.append(f'{pad}{ctx["acc"]} += (cast,trunc(int) fclamp({name}(cast(float64) (({self.iexpr(scope, 1)}) & 255), {self.r.randint(1, 4)}.5))) & 1023;')
            elif ty == 's32':
                out.append(f'{pad}{ctx["acc"]} += cast(int) {name}(cast,trunc(s32) ({self.iexpr(scope, 1)}), cast(s32) {self.r.randint(1, 9)});')
            else:
                out.append(f'{pad}{ctx["acc"]} += {name}({self.iexpr(scope, 1)}, {self.r.randint(1, 9)});')
        elif kind == 'cast':
            ty = r.choice(['u8', 's32', 'u16', 'u32'])
            out.append(f'{pad}{ctx["acc"]} += cast(int) (cast,trunc({ty}) ({self.iexpr(scope)}));')

    def ret_expr(self, ctx):
        s, t = ctx['scope'], ctx['ret']
        if t == 'bool': return self.bexpr(s)
        if t == 'string': return self.sexpr(s)
        if t == 'float64': return self.fexpr(s)
        if t == 'int': return self.iexpr(s)
        return f'cast,trunc({t}) ({self.iexpr(s)})'

    def assign(self, ctx, out, pad):
        r, scope = self.r, ctx['scope']
        ints = [v for v in scope if v.kind == 'int']
        targets = []
        for v in scope:
            if v.kind == 'int' and v.mutable: targets.append((v.name, v.ty))
            elif v.kind == 'struct' and v.mutable:
                targets += [(p, t) for p, t, _ in self.leaves(v, 'int')]
        if not targets or self.chance(0.12):
            out.append(f'{pad}{ctx["acc"]} {r.choice(["+=", "^=", "+="])} {self.iexpr(scope)};'); return
        name, ty = r.choice(targets)
        op = pick(r, {'=': 40, '+=': 40, '-=': 8, '^=': 6, '|=': 3, '&=': 3})
        e = self.iexpr(scope)
        if ty == 'int':
            out.append(f'{pad}{name} {op} {e};')
        else:
            out.append(f'{pad}{name} = cast,trunc({ty}) (cast(int) {name} {op[:-1] or "+"} ({e})) ;' if op != '=' else f'{pad}{name} = cast,trunc({ty}) ({e});')

    def loop(self, ctx, out, pad, indent, depth):
        r, scope = self.r, ctx['scope']
        arrs = [v for v in scope if v.kind == 'arr' or (v.kind == 'struct' and self.leaves(v, 'arr'))]
        it = self.uid('i')
        if arrs and self.chance(0.6):
            v = r.choice(arrs)
            arr = v.name if v.kind == 'arr' else r.choice(self.leaves(v, 'arr'))[0]
            out.append(f'{pad}for {it}: {arr} {{')
            n = 8
        else:
            n = r.randint(2, 6)
            out.append(f'{pad}for {it}: 0..{n - 1} {{')
        ctx['mult'] *= n
        mark = len(scope)
        scope.append(Var(it, 'int', 'int', 0))
        scope[-1].mutable = False
        self.stmts(ctx, r.randint(1, 4), out, indent + 1, depth + 1)
        del scope[mark:]
        ctx['mult'] //= n
        out.append(f'{pad}}}')

    def call(self, ctx, out, pad):
        scope = ctx['scope']
        cands = [p for p in self.procs if p.cost * ctx['mult'] + ctx['cost'] <= 4000 and all(k in ('int', 'float', 'bool', 'string') for k, _ in p.params)]
        if not cands:
            self.assign(ctx, out, pad); return
        p = self.r.choice(cands)
        ctx['cost'] += p.cost * ctx['mult']
        args = ', '.join(self.arg(k, ty, scope) for k, ty in p.params)
        out.append(f'{pad}{self.fold(p, f"{p.name}({args})", ctx["acc"])}')

    def fold(self, p, call, acc):
        if p.ret == 'void': return f'{call};'
        if p.ret == 'bool': return f'if {call} {acc} += 1;'
        # The callee may change the global `acc`, and the order of `acc += f()` is not defined: bind first.
        tmp = self.uid('r')
        if p.ret == 'string': value = f'{tmp}.count'
        elif p.ret == 'float64': value = f'cast,trunc(int) fclamp({tmp})'
        else: value = f'cast(int) {tmp}'
        return f'{tmp} := {call}; {acc} += {value};'

    def arg(self, kind, ty, scope):
        if kind == 'int':
            e = self.iexpr(scope, 1)
            return e if ty == 'int' else f'cast,trunc({ty}) ({e})'
        if kind == 'float': return self.fexpr(scope)
        if kind == 'bool': return self.bexpr(scope, 1)
        return self.sexpr(scope)

    # ---- procedures ---------------------------------------------------------------------------
    def param_list(self):
        hist = self.shape['proc_params']['hist']
        n = self.r.choices(range(len(hist)), weights=hist)[0]
        ps = []
        for _ in range(min(n, 5)):
            k = pick(self.r, {'int': 20, 's32': 10, 'u32': 10, 'u8': 4, 'float': 10, 'bool': 8, 'string': 15, 'ptr': 18, 'enum': 3})
            if k in ('ptr',) and not self.structs: k = 'int'
            if k == 'enum' and not self.enums: k = 'int'
            if k in INT_TYPES: ps.append(('int', k))
            elif k == 'float': ps.append(('float', 'float64'))
            elif k == 'bool': ps.append(('bool', 'bool'))
            elif k == 'string': ps.append(('string', 'string'))
            elif k == 'ptr': ps.append(('ptr', self.r.choice(self.structs).name))
            else: ps.append(('enum', self.r.choice(list(self.enums))))
        return ps

    def emit_proc(self):
        r = self.r
        verb, noun = r.choice(VERBS), r.choice(NOUNS)
        name = self.uid(f'{verb}_{noun}')
        params = self.param_list()
        ret = pick(r, {'void': 43, 'bool': 10, 'int': 20, 'string': 6, 'float64': 5, 'u32': 6, 's32': 6, 'u8': 4})
        hist = self.shape['proc_body_lines_hist']
        target = min(150, bucket(r, hist['edges'], hist['share']))
        scope, plist = [], []
        for i, (k, ty) in enumerate(params):
            pn = f'{"abcdefgh"[i]}{r.choice(["", "n", "v"])}'
            if k == 'ptr':
                scope.append(Var(pn, 'struct', ty)); plist.append(f'{pn}: *{ty}')
            elif k == 'enum':
                scope.append(Var(pn, 'enum', ty)); plist.append(f'{pn}: {ty}')
            else:
                scope.append(Var(pn, k, ty)); plist.append(f'{pn}: {ty}')
        simple = all(k in ('int', 'float', 'bool', 'string') for k, _ in params)
        out = []
        ctx = {'scope': scope, 'ret': ret, 'acc': 'acc', 'mult': 1, 'cost': 1, 'allow_defer': True}
        pre = []
        if ret != 'void' or not scope or self.chance(0.5):
            ctx['acc'] = 'local_acc'
            pre.append('    local_acc := 0;')
        if self.chance(0.1) and getattr(self, 'macros', None) and ctx['acc'] == 'local_acc':
            ctx['macro'] = True
        if self.chance(0.08):
            pre.append('    arr: [8] int;'); scope.append(Var('arr', 'arr', 'int', 8))
        if self.chance(0.05):
            pre.append('    dyn: [..] int;'); pre.append('    defer array_free(dyn);'); scope.append(Var('dyn', 'dyn', 'int'))
        for v in scope:
            v.mutable = v.kind in ('arr', 'dyn') or (v.kind == 'struct')
        if ret != 'void' or scope:
            ctx['mult'] = 1
        self.stmts(ctx, target, out, 1, 0)
        post = []
        if ctx['acc'] == 'local_acc':
            if ret == 'void': post.append('    acc += local_acc;')
        if ret == 'bool': post.append(f'    return {self.bexpr(scope)} || local_acc > 5;')
        elif ret == 'string': post.append(f'    return tprint("%", {self.iexpr(scope, 1)} + local_acc);')
        elif ret == 'float64': post.append(f'    return {self.fexpr(scope)} + cast(float64) (local_acc & 255);')
        elif ret == 'int': post.append(f'    return {self.iexpr(scope)} + local_acc;')
        elif ret != 'void': post.append(f'    return cast,trunc({ret}) ({self.iexpr(scope)} + local_acc);')
        sig = f'{name} :: ({", ".join(plist)})' + ('' if ret == 'void' else f' -> {ret}')
        self.lines += [f'{sig} {{'] + pre + out + post + ['}', '']
        self.procs.append(Proc(name, params, ret, max(1, ctx['cost'] + ctx['mult'] * 2)))
        self.procs[-1].params_names = plist

    # ---- drivers ------------------------------------------------------------------------------
    def emit_drivers(self, procs):
        """Group procs into drivers that call each one once; returns the driver names."""
        names = []
        for i in range(0, len(procs), 30):
            group = procs[i:i + 30]
            dn = self.uid('drive')
            body = []
            tmp = 0
            for p in group:
                args = []
                for k, ty in p.params:
                    if k == 'int':
                        v = self.r.randint(0, 99)
                        args.append(str(v) if ty == 'int' else f'cast,trunc({ty}) {v}')
                    elif k == 'float': args.append(f'{self.r.randint(0, 9)}.5')
                    elif k == 'bool': args.append(self.r.choice(['true', 'false']))
                    elif k == 'string': args.append(f'"abc{self.r.randint(0, 9)}"')
                    elif k == 'enum': args.append(f'{ty}.{self.r.choice(self.enums[ty])}')
                    else:
                        tmp += 1
                        body.append(f'    st{tmp}_: {ty};')
                        args.append(f'*st{tmp}_')
                call = f'{p.name}({", ".join(args)})'
                body.append('    ' + self.fold(p, call, 'acc'))
                if self.trace: body.append(f'    print("{p.name} %\\n", acc);')
            if self.trace:
                body = [x for b in body for x in (b, f'    print("%\\n", acc);') if not b.startswith('    st')] if False else body
            body.append('    reset_temporary_storage();')
            self.lines += [f'{dn} :: () {{'] + body + ['}', '']
            names.append(dn)
        return names

    def prelude(self):
        return ['#import "Basic";', '#import "Math";', '#import "String";', '#import "Hash_Table";', '',
                'acc: int;', '',
                'fclamp :: (f: float64) -> float64 {', '    if !(f == f) return 0;', '    if f > 1000000000.0 return 1000000000.0;',
                '    if f < -1000000000.0 return -1000000000.0;', '    return f;', '}', '']

    def emit_table_user(self):
        name = self.uid('count_table')
        self.lines += [f'{name} :: (n: int) -> int {{', '    t: Table(int, int);', '    defer deinit(*t);',
                       '    for 0..n - 1 table_set(*t, it, it * 3);',
                       '    total := 0;', '    for 0..n - 1 {', '        p := table_find_pointer(*t, it);', '        if p total += <<p;', '    }',
                       '    return total;', '}', '']
        return name

    def generate(self, target_lines: int, files: int):
        sh = self.shape
        # Item mix per kLOC, from the measured densities (structs and enums are declarations).
        self.lines = self.prelude()
        for _ in range(3): self.emit_enum()
        for _ in range(4): self.emit_struct()
        self.emit_macro(); self.emit_poly_proc(); self.emit_poly_proc(); self.emit_poly_struct()
        table_fns = [self.emit_table_user()]
        drive_extra: list[str] = []
        proc_budget_per_struct = max(1, int(sh['procs_per_kloc'] / sh['structs_per_kloc']))
        mark_procs = 0
        groups: list[list[Proc]] = []
        # Drivers add about one line per procedure (plus temporaries for pointer arguments).
        while len(self.lines) + int(len(self.procs) * 1.4) + 60 < target_lines:
            roll = self.r.random() * (sh['procs_per_kloc'] + sh['structs_per_kloc'] * 0.9 + sh['enums_per_kloc'])
            if roll < sh['enums_per_kloc']:
                self.emit_enum()
            elif roll < sh['enums_per_kloc'] + sh['structs_per_kloc'] * 0.9:
                if self.chance(sh['poly_struct_share'] * 2): self.emit_poly_struct()
                else: self.emit_struct()
            else:
                x = self.r.random()
                if x < sh['poly_proc_share'] * 0.5: self.emit_poly_proc()
                elif x < sh['poly_proc_share'] * 0.5 + 0.004: self.emit_macro()
                elif x < sh['poly_proc_share'] * 0.5 + 0.012:
                    self.emit_run_table()
                elif x < sh['poly_proc_share'] * 0.5 + 0.016: self.emit_insert()
                elif x < sh['poly_proc_share'] * 0.5 + 0.02: table_fns.append(self.emit_table_user())
                else: self.emit_proc()
        body_procs = [p for p in self.procs]
        drivers = self.emit_drivers(body_procs)
        # Exercise the polymorphic boxes, compile-time tables and inserted procedures too.
        misc = ['    for 0..7 acc += cast(int) (it * 3);']
        for name, push, total in self.poly_boxes:
            misc += [f'    {{', f'        b: {name}(int);', f'        for 1..5 {push}(*b, it * 3);', f'        acc += {total}(b);', '    }']
            misc += [f'    {{', f'        b: {name}(float64);', f'        for 1..3 {push}(*b, cast(float64) it);', f'        acc += cast,trunc(int) {total}(b);', '    }']
        for t in self.runs: misc.append(f'    for {t} acc += it & 255;')
        for p in getattr(self, 'insert_procs', []): misc.append(f'    acc += {p}(3);')
        for t in table_fns: misc.append(f'    acc += {t}(20);')
        self.lines += ['drive_misc :: () {'] + misc + ['}', '']
        drivers.append('drive_misc')
        main = ['main :: () {'] + [f'    {d}();' for d in drivers]
        main += ['    print("checksum %\\n", acc);', '}']
        self.lines += main
        return self.split(files)

    def split(self, files: int) -> dict[str, str]:
        if files <= 1:
            return {'main.jai': '\n'.join(self.lines) + '\n'}
        # Part files hold only declarations; globals and main stay in main.jai.
        head_end = next(i for i, l in enumerate(self.lines) if l.startswith('fclamp'))
        main_start = next(i for i, l in enumerate(self.lines) if l.startswith('main ::'))
        decls = self.lines[head_end:main_start]
        # split at blank lines between top-level items
        items, cur = [], []
        for l in decls:
            cur.append(l)
            if l == '' and (len(cur) > 1):
                items.append(cur); cur = []
        if cur: items.append(cur)
        per = (len(items) + files - 2) // (files - 1)
        out = {}
        loads = []
        for k in range(files - 1):
            chunk = items[k * per:(k + 1) * per]
            if not chunk: continue
            name = f'part_{k + 1}.jai'
            header = ['#import "Basic";', '#import "Math";', '#import "String";', '#import "Hash_Table";', '']
            out[name] = '\n'.join(header + [l for it in chunk for l in it]) + '\n'
            loads.append(f'#load "{name}";')
        out['main.jai'] = '\n'.join(self.lines[:head_end] + loads + [''] + self.lines[main_start:]) + '\n'
        return out


def generate(lines: int, seed: int = 1, files: int = 1, shape: dict | None = None, trace: bool = False) -> dict[str, str]:
    shape = shape or json.loads((ROOT / 'tools' / 'corpus-shape.json').read_text())
    return Gen(seed, shape, trace).generate(lines, files)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--lines', type=int, default=10000)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--files', type=int, default=1)
    ap.add_argument('--shape', type=Path, default=ROOT / 'tools' / 'corpus-shape.json')
    ap.add_argument('--trace', action='store_true', help='print the checksum after every driver call (to find miscompiles)')
    ap.add_argument('--out', type=Path, required=True, help='directory to write main.jai (and parts) into')
    a = ap.parse_args()
    a.out.mkdir(parents=True, exist_ok=True)
    for name, text in generate(a.lines, a.seed, a.files, json.loads(a.shape.read_text()), a.trace).items():
        (a.out / name).write_text(text)
    print(sum(p.read_text().count('\n') for p in a.out.glob('*.jai')), 'lines')


if __name__ == '__main__':
    main()
