#!/usr/bin/env python3
"""Random program generator for differential testing (csmith-style).

Usage: tools/jaigen.py SEED [--out FILE]

Generates a valid, well-typed, deterministic, terminating Jai program from SEED and prints it. The program
folds its state into a 64-bit FNV-style checksum `H` as it goes, prints intermediate values (integers,
floats, enums, structs, strings through Basic's formatting) and ends with the checksum. Every backend must
print the same text; tools/jaic-diff.py runs them (`gen:SEED:COUNT`).

Only behaviour that docs/language/*.md defines is generated:
- integer + - * wrap at the type's width (numbers.md); shift and rotate amounts are masked below the width;
  division and remainder only by a divisor that is neither 0 nor -1 (truncating, numbers.md);
- integer casts wrap (casts-and-conversions.md); float -> integer casts only of values guarded into range;
- floats are IEEE: NaN payloads and signs are not observed (hashed and printed through a canonical form);
- procedures used inside expressions are pure, because operand and argument evaluation order is not
  documented; side effects on the checksum happen in statements only;
- loops have constant trip counts and recursion carries a decreasing depth, so every program terminates.
"""
import argparse, random, re, sys

INTS = {"s8": (8, True), "s16": (16, True), "s32": (32, True), "s64": (64, True),
        "u8": (8, False), "u16": (16, False), "u32": (32, False), "u64": (64, False)}
FLOATS = ["float32", "float64"]
SCALARS = list(INTS) + FLOATS + ["bool"]


def is_int(t):
    return isinstance(t, str) and t in INTS


def is_float(t):
    return isinstance(t, str) and t in FLOATS


def rng_of(t):
    bits, signed = INTS[t]
    return (-(1 << (bits - 1)), (1 << (bits - 1)) - 1) if signed else (0, (1 << bits) - 1)


def tyname(t):
    if isinstance(t, str):
        return t
    kind = t[0]
    if kind == "array":
        return f"[{t[2]}] {tyname(t[1])}"
    if kind in ("struct", "enum"):
        return t[1]
    if kind == "pair":
        return f"Pair({t[1]})"
    if kind == "view":
        return f"[] {tyname(t[1])}"
    if kind == "dyn":
        return f"[..] {tyname(t[1])}"
    raise ValueError(t)


class Var:
    def __init__(self, name, ty, mutable=True):
        self.name, self.ty, self.mutable = name, ty, mutable


class Proc:
    def __init__(self, name, params, ret, pure, poly=False):
        self.name, self.params, self.ret, self.pure, self.poly = name, params, ret, pure, poly


class Gen:
    def __init__(self, seed, size=1.0):
        self.r = random.Random(seed)
        self.seed = seed
        self.size = size
        self.counter = 0
        self.structs = {}   # name -> [(field, ty, default)]
        self.enums = {}     # name -> (base, [(member, value)], flags)
        self.procs = []
        self.globals = []
        self.out = []
        self.scopes = []
        self.depth_budget = 0
        self.in_pure = False
        self.loop_depth = 0

    # ---- helpers -------------------------------------------------------------------------------
    def fresh(self, prefix):
        self.counter += 1
        return f"{prefix}{self.counter}"

    def chance(self, p):
        return self.r.random() < p

    def pick(self, seq):
        return self.r.choice(list(seq))

    def emit(self, line, indent):
        self.out.append("    " * indent + line)

    def vars(self):
        """Variables in scope; an inner declaration hides an outer one of the same name (`it`)."""
        seen = {}
        for scope in self.scopes:
            for v in scope:
                seen[v.name] = v
        return list(seen.values())

    def int_literal(self, t):
        lo, hi = rng_of(t)
        k = self.r.random()
        if k < 0.4:
            v = self.r.randint(max(lo, -20), min(hi, 20))
        elif k < 0.6:
            v = self.pick([lo, hi, lo + 1, hi - 1, 0, 1])
        elif k < 0.75:
            v = self.edge_int(lo, hi)
        else:
            v = self.r.randint(lo, hi)
        # Typed, so that folding a constant subexpression wraps at the width instead of failing to fit.
        text = f"({v})" if v < 0 else (str(v) if self.chance(0.7) else hex(v))
        return f"cast({t}) {text}"

    def edge_int(self, lo, hi):
        """Bit patterns where conversions and rounding go wrong: powers of two and their
        neighbours, and integers just above a float32/float64 rounding tie."""
        bits = max(abs(lo), hi).bit_length()
        k = self.r.randint(1, bits)
        cands = [1 << (k - 1), (1 << (k - 1)) - 1, (1 << (k - 1)) + 1]
        for mant in (24, 53):
            if k > mant + 1:
                tie = (1 << (k - 1)) + (1 << (k - 1 - mant))
                cands += [tie, tie + 1, tie - 1, tie + (1 << (k - mant))]
        v = self.pick(cands)
        if lo < 0 and self.chance(0.5):
            v = -v
        return max(lo, min(hi, v))

    def float_literal(self, t):
        k = self.r.random()
        if k < 0.5:
            v = self.r.randint(-64, 64) / self.pick([1, 2, 4, 8, 16])
            s = repr(float(v))
        elif k < 0.8:
            s = f"{self.r.uniform(-1000, 1000):.6f}"
        else:
            s = self.pick(["0.1", "1e10", "3.0e-5", "123456.789", "0.333333", "1e-30", "6.02e23", "2.5", "0.5",
                           "-0.0", "1.5", "-2.5", "16777217.0", "9007199254740993.0", "1.0e-40", "3.4e38",
                           "2147483647.5", "4294967295.0", "0.49999999999999994", "1e300"])
        return f"cast({t}) ({s})" if s.startswith("-") else f"cast({t}) {s}"

    # ---- l-values and readable places ----------------------------------------------------------
    def places(self, ty, mutable_only=False):
        """Expressions naming a place of type `ty` reachable from the variables in scope. Array indices
        are markers until `realize` (picking one place must not generate an index for every other)."""
        found = []
        for v in self.vars():
            if mutable_only and not v.mutable:
                continue
            self.collect(v.name, v.ty, ty, found, 0)
        return found

    def pick_place(self, ty, mutable_only=False):
        found = self.places(ty, mutable_only)
        return self.realize(self.pick(found)) if found else None

    def realize(self, path):
        return re.sub("\x00([^\x00]*)\x00", lambda m: self.index_expr(m.group(1)), path)

    def collect(self, path, have, want, found, depth):
        if have == want:
            found.append(path)
        if depth > 2 or isinstance(have, str):
            return
        if have[0] == "struct":
            for f, fty, _ in self.structs[have[1]]:
                self.collect(f"{path}.{f}", fty, want, found, depth + 1)
        elif have[0] == "pair":
            for f in ("a", "b"):
                self.collect(f"{path}.{f}", have[1], want, found, depth + 1)
        elif have[0] in ("array", "view", "dyn") and depth < 2 and not self.no_index:
            elem = have[1]
            if self.reaches(elem, want):
                if have[0] == "array":
                    count = str(have[2])
                else:
                    count = f"{path}.count"
                if have[0] != "array" and not self.nonempty(path):
                    return
                self.collect(f"{path}[\x00{count}\x00]", elem, want, found, depth + 1)

    def nonempty(self, path):
        return path in self.nonempty_paths

    def reaches(self, have, want):
        if have == want:
            return True
        if isinstance(have, str):
            return False
        if have[0] == "struct":
            return any(self.reaches(f, want) for _, f, _ in self.structs[have[1]])
        if have[0] == "pair":
            return self.reaches(have[1], want)
        if have[0] in ("array", "view", "dyn"):
            return self.reaches(have[1], want)
        return False

    def index_expr(self, count):
        saved = self.no_index
        self.no_index = True
        e = self.expr(self.pick(INTS), 2)
        self.no_index = saved
        return f"wrap_index(cast,no_check(s64) ({e}), {count})"

    # ---- expressions ---------------------------------------------------------------------------
    def expr(self, ty, depth):
        if isinstance(ty, tuple):
            return self.aggregate_expr(ty, depth)
        if ty == "string":
            return self.string_expr()
        if depth >= 4 or self.chance(0.25 + 0.15 * depth):
            return self.leaf(ty)
        if is_int(ty):
            return self.int_expr(ty, depth + 1)
        if is_float(ty):
            return self.float_expr(ty, depth + 1)
        return self.bool_expr(depth + 1)

    def leaf(self, ty):
        if ty == "string":
            return self.string_expr()
        places = self.places(ty)
        if places and self.chance(0.75):
            return self.realize(self.pick(places))
        if is_int(ty):
            return self.int_literal(ty)
        if is_float(ty):
            return self.float_literal(ty)
        return self.pick(["true", "false"])

    def int_expr(self, t, d):
        bits, signed = INTS[t]
        k = self.r.randint(0, 15)
        a = lambda: self.expr(t, d)
        if k <= 3:
            op = self.pick(["+", "-", "*", "&", "|", "^", "+", "-", "*"])
            return f"({a()} {op} {a()})"
        if k == 4:
            op = self.pick(["<<", ">>", "<<<", ">>>"])
            return f"({a()} {op} ({a()} & {bits - 1}))"
        if k == 5:
            # Divisor forced into [1, 2^k - 1]: never 0 or -1.
            op = self.pick(["/", "%"])
            mask = self.pick([1, 3, 7, 15, 63, 127])
            return f"({a()} {op} (({a()} & {mask}) | 1))"
        if k == 6:
            op = self.pick(["/", "%"])
            fn = "safe_div" if op == "/" else "safe_rem"
            return f"{fn}_{t}({a()}, {a()})"
        if k == 7:
            if signed and self.chance(0.5):
                return f"(-{a()})"
            return self.pick([f"(~{a()})", f"(0 - {a()})"])
        if k == 8:
            src = self.pick([x for x in INTS if x != t])
            mod = self.pick(["", "", ",trunc", ",no_check"])
            return f"(cast{mod}({t}) {self.expr(src, d)})"
        if k == 9:
            src = self.pick(FLOATS)
            return self.float_to_int(t, self.expr(src, d), src)
        if k == 10:
            return f"(ifx {self.bool_expr(d)} then {a()} else {a()})"
        if k == 11:
            return self.call_expr(t, d) or self.leaf(t)
        if k == 12:
            return f"(ifx {self.bool_expr(d)} then cast({t}) 1 else cast({t}) 0)"
        if k == 13:
            e = self.enum_value_expr(d)
            if e:
                return f"(cast,no_check({t}) {e})"
            return self.leaf(t)
        if k == 14:
            s = self.string_expr()
            if self.chance(0.5):
                return f"(cast,no_check({t}) {s}.count)"
            return f"(cast({t}) string_byte({s}, {self.expr('s64', d)}))"
        poly = [p for p in self.procs if p.poly and p.pure]
        if poly:
            p = self.pick(poly)
            return f"{p.name}({a()}, {a()})"
        return self.leaf(t)

    def float_to_int(self, t, e, src):
        lo, hi = rng_of(t)
        if self.chance(0.5):
            # Guarded direct conversion to the target width.
            lo_g, hi_g = (lo, hi) if INTS[t][0] <= 32 else (-(1 << 52), 1 << 52)
            if not INTS[t][1]:
                lo_g = 0
            if src == "float32" and INTS[t][0] >= 32:
                hi_g = min(hi_g, 1 << 24)
                lo_g = max(lo_g, -(1 << 24))
            return f"f_to_{t}_{src}({e}, {lo_g}, {hi_g})"
        return f"(cast,no_check({t}) f_to_s64_{src}({e}, -1000000000, 1000000000))"

    def float_expr(self, t, d):
        k = self.r.randint(0, 9)
        a = lambda: self.expr(t, d)
        if k <= 3:
            op = self.pick(["+", "-", "*", "/", "+", "*"])
            return f"({a()} {op} {a()})"
        if k == 4:
            return f"(-{a()})"
        if k == 5:
            src = self.pick(INTS)
            return f"(cast({t}) {self.expr(src, d)})"
        if k == 6:
            other = "float64" if t == "float32" else "float32"
            return f"(cast({t}) {self.expr(other, d)})"
        if k == 7:
            return f"(ifx {self.bool_expr(d)} then {a()} else {a()})"
        if k == 8:
            return self.call_expr(t, d) or self.leaf(t)
        return f"clamp_f_{t}({a()})"

    def bool_expr(self, d):
        k = self.r.randint(0, 6)
        if k <= 2:
            t = self.pick(SCALARS[:-1])
            op = self.pick(["<", "<=", ">", ">=", "==", "!="])
            return f"({self.expr(t, d)} {op} {self.expr(t, d)})"
        if k == 3:
            op = self.pick(["&&", "||"])
            return f"({self.expr('bool', d)} {op} {self.expr('bool', d)})"
        if k == 4:
            return f"(!{self.expr('bool', d)})"
        if k == 5:
            e = self.enum_compare(d)
            if e:
                return e
        if k == 6:
            if self.chance(0.3):
                return f"({self.string_expr()} {self.pick(['==', '!='])} {self.string_expr()})"
            return f"(cast(bool) {self.expr(self.pick(INTS), d)})"
        return self.leaf("bool")

    def enum_value_expr(self, d):
        names = [n for n, (_, _, flags) in self.enums.items() if not flags]
        if not names:
            return None
        name = self.pick(names)
        places = self.places(("enum", name))
        if places and self.chance(0.7):
            return self.realize(self.pick(places))
        return f"{name}.{self.pick(self.enums[name][1])[0]}"

    def enum_compare(self, d):
        names = list(self.enums)
        if not names:
            return None
        name = self.pick(names)
        places = self.places(("enum", name))
        if not places:
            return None
        base, members, flags = self.enums[name]
        m = self.pick(members)[0]
        if flags:
            return f"(({self.realize(self.pick(places))} & .{m}) == .{m})"
        return f"({self.realize(self.pick(places))} {self.pick(['==', '!='])} .{m})"

    def string_expr(self):
        places = self.places("string")
        if places and self.chance(0.7):
            return self.realize(self.pick(places))
        return self.string_literal()

    def string_literal(self):
        words = ["alpha", "beta", "gamma", "", "x", "hello world", "tab\\there", "quote\\\"q", "nl\\nline", "ünï"]
        return f"\"{self.pick(words)}\""

    def call_expr(self, t, d):
        cands = [p for p in self.procs if p.pure and not p.poly and p.ret == t]
        if not cands or d >= 3:
            return None
        p = self.pick(cands)
        pointers = [name for name, q in self.fptrs if q is p]
        callee = self.pick(pointers) if pointers and self.chance(0.6) else p.name
        return f"{callee}({self.call_args(p, d + 1)})"

    def call_args(self, p, d):
        """Arguments for a call of `p`; a recursive procedure's depth is a small constant."""
        args = [self.expr(pt, d) for _, pt in p.params]
        if p.params and p.params[0][0] == "depth":
            args[0] = str(self.r.randint(0, 4))
        return ", ".join(args)

    def aggregate_expr(self, ty, d):
        places = self.places(ty)
        if ty[0] == "enum":
            name = ty[1]
            base, members, flags = self.enums[name]
            if places and self.chance(0.5):
                return self.realize(self.pick(places))
            if flags:
                ms = self.r.sample([m for m, _ in members], self.r.randint(1, len(members)))
                return f"({' | '.join(f'{name}.{m}' for m in ms)})"
            return f"{name}.{self.pick(members)[0]}"
        if places and self.chance(0.6):
            return self.realize(self.pick(places))
        if ty[0] == "struct":
            makers = [p for p in self.procs if p.pure and p.ret == ty]
            if makers and self.chance(0.4) and d < 3:
                p = self.pick(makers)
                return f"{p.name}({self.call_args(p, d + 1)})"
            fields = self.structs[ty[1]]
            chosen = [f for f in fields if self.chance(0.6)]
            inits = ", ".join(f"{f} = {self.expr(fty, d + 1)}" for f, fty, _ in chosen)
            return f"{ty[1]}.{{{inits}}}"
        if ty[0] == "pair":
            return f"Pair({ty[1]}).{{{self.expr(ty[1], d + 1)}, {self.expr(ty[1], d + 1)}}}"
        if ty[0] == "array":
            return f"{tyname(ty[1])}.[{', '.join(self.expr(ty[1], d + 1) for _ in range(ty[2]))}]"
        raise ValueError(ty)

    # ---- hashing --------------------------------------------------------------------------------
    def mix_of(self, path, ty):
        """Statements folding the value at `path` (of type `ty`) into H."""
        if is_int(ty):
            return [f"mix(cast,no_check(u64) {path});"]
        if is_float(ty):
            return [f"mix_float(cast(float64) {path});"]
        if ty == "bool":
            return [f"mix(ifx {path} then 0x9e37 else 0x79b9);"]
        if ty == "string":
            return [f"mix_string({path});"]
        if ty[0] == "enum":
            return [f"mix(cast,no_check(u64) {path});"]
        if ty[0] == "struct":
            out = []
            for f, fty, _ in self.structs[ty[1]]:
                out += self.mix_of(f"{path}.{f}", fty)
            return out
        if ty[0] == "pair":
            return self.mix_of(f"{path}.a", ty[1]) + self.mix_of(f"{path}.b", ty[1])
        if ty[0] in ("array", "view", "dyn"):
            inner = self.mix_of("it", ty[1])
            return [f"mix(cast(u64) {path}.count);", f"for {path} {{ {' '.join(inner)} }}"]
        raise ValueError(ty)

    # ---- statements ----------------------------------------------------------------------------
    def declare(self, ty, indent, name=None, init=None):
        name = name or self.fresh("v")
        if isinstance(ty, tuple) and ty[0] == "dyn":
            self.emit(f"{name}: {tyname(ty)};", indent)
        else:
            if init is None:
                init = self.expr(ty, 1)
            self.emit(f"{name}: {tyname(ty)} = {init};", indent)
        self.scopes[-1].append(Var(name, ty))
        return name

    def random_type(self, aggregate=True):
        k = self.r.random()
        if k < 0.55 or not aggregate:
            return self.pick(SCALARS + list(INTS) + ["string"]) if aggregate else self.pick(SCALARS + list(INTS))
        if k < 0.7 and self.structs:
            return ("struct", self.pick(self.structs))
        if k < 0.8 and self.enums:
            return ("enum", self.pick(self.enums))
        if k < 0.85:
            return ("array", self.pick(SCALARS), self.r.randint(1, 5))
        if k < 0.9 and self.structs:
            return ("array", ("struct", self.pick(self.structs)), self.r.randint(1, 3))
        return ("pair", self.pick(list(INTS)))

    def block(self, indent, n):
        self.scopes.append([])
        for _ in range(n):
            self.stmt(indent)
        self.scopes.pop()

    def stmt(self, indent):
        k = self.r.randint(0, 19)
        nest_ok = self.depth_budget > 0
        if k <= 3:
            return self.assign(indent)
        if k == 4:
            return self.declare(self.random_type(), indent)
        if k == 5 and not self.in_pure:
            v = self.pick_place_any()
            if v:
                for line in self.mix_of(*v):
                    self.emit(line, indent)
                return
        if k == 6 and nest_ok:
            self.depth_budget -= 1
            self.emit(f"if {self.bool_expr(1)} {{", indent)
            self.block(indent + 1, self.r.randint(1, 3))
            if self.chance(0.5):
                self.emit("} else {", indent)
                self.block(indent + 1, self.r.randint(1, 3))
            self.emit("}", indent)
            self.depth_budget += 1
            return
        if k == 7 and nest_ok:
            return self.for_range(indent)
        if k == 8 and nest_ok:
            return self.while_loop(indent)
        if k == 9 and nest_ok:
            return self.for_array(indent)
        if k == 10 and nest_ok:
            return self.switch(indent)
        if k == 11 and not self.in_pure:
            return self.print_stmt(indent)
        if k == 12 and nest_ok:
            self.depth_budget -= 1
            self.emit("{", indent)
            self.scopes.append([])
            if not self.in_pure:
                self.emit(f"defer mix({self.r.randint(1, 1 << 30)});", indent + 1)
            t = self.pick(list(INTS))
            target = self.pick_place(t, mutable_only=True)
            if target:
                self.emit(f"defer {target} ^= cast({t}) 0x5a;", indent + 1)
            for _ in range(self.r.randint(1, 3)):
                self.stmt(indent + 1)
            self.scopes.pop()
            self.emit("}", indent)
            self.depth_budget += 1
            return
        if k == 13 and nest_ok:
            self.depth_budget -= 1
            self.emit("#no_aoc {", indent)
            self.block(indent + 1, self.r.randint(1, 2))
            self.emit("}", indent)
            self.depth_budget += 1
            return
        if k == 14 and not self.in_pure:
            return self.call_stmt(indent)
        if k == 15:
            # Never grow an array while a loop walks it: the loop would see the new elements
            # (and `for *` would hold pointers into the old buffer).
            dyns = [v for v in self.vars() if isinstance(v.ty, tuple) and v.ty[0] == "dyn" and v.mutable
                    and v.name not in self.iterating]
            if dyns:
                v = self.pick(dyns)
                self.emit(f"array_add(*{v.name}, {self.expr(v.ty[1], 1)});", indent)
                return
        if k == 16:
            return self.string_stmt(indent)
        if k == 17:
            return self.pointer_stmt(indent)
        return self.assign(indent)

    def pick_place_any(self):
        cands = []
        for v in self.vars():
            cands.append((v.name, v.ty))
        return self.pick(cands) if cands else None

    def assign(self, indent):
        ty = self.pick(SCALARS + list(INTS))
        targets = self.places(ty, mutable_only=True)
        if not targets:
            return self.declare(ty, indent)
        t = self.realize(self.pick(targets))
        narrower = [n for n in INTS if is_int(ty) and self.widens(n, ty)]
        if narrower and self.chance(0.15):
            # Implicit widening: sign or zero extension of a narrower integer.
            self.emit(f"{t} = {self.expr(self.pick(narrower), 1)};", indent)
        elif is_int(ty) and self.chance(0.5):
            op = self.pick(["+=", "-=", "*=", "&=", "|=", "^="])
            self.emit(f"{t} {op} {self.expr(ty, 1)};", indent)
        elif is_float(ty) and self.chance(0.3):
            self.emit(f"{t} {self.pick(['+=', '-=', '*='])} {self.expr(ty, 1)};", indent)
        else:
            self.emit(f"{t} = {self.expr(ty, 1)};", indent)

    @staticmethod
    def widens(src, dst):
        """`src` converts to `dst` implicitly: the whole range fits (numbers.md)."""
        (sb, ss), (db, ds) = INTS[src], INTS[dst]
        if src == dst:
            return False
        return (ss == ds and sb < db) or (not ss and ds and sb < db)

    def loop_prologue(self, indent):
        # A defer in a loop body runs at the end of every iteration, also on `break` and `continue`.
        if not self.in_pure and self.chance(0.25):
            self.emit(f"defer mix({self.r.randint(1, 1 << 20)});", indent)

    def for_range(self, indent):
        self.depth_budget -= 1
        self.loop_depth += 1
        lo = self.r.randint(-3, 3)
        hi = lo + self.r.randint(-1, 5)
        name = self.fresh("i")
        rev = "< " if self.chance(0.2) else ""
        self.emit(f"for {rev}{name}: {lo}..{hi} {{", indent)
        self.scopes.append([Var(name, "s64", mutable=False)])
        self.loop_prologue(indent + 1)
        if self.chance(0.3):
            self.emit(f"if {name} == {self.r.randint(lo, hi + 1)} {self.pick(['break', 'continue'])};", indent + 1)
        for _ in range(self.r.randint(1, 3)):
            self.stmt(indent + 1)
        self.scopes.pop()
        self.emit("}", indent)
        self.loop_depth -= 1
        self.depth_budget += 1

    def while_loop(self, indent):
        self.depth_budget -= 1
        c = self.fresh("w")
        self.emit(f"{c} := 0;", indent)
        self.emit(f"while {c} < {self.r.randint(0, 6)} {{", indent)
        self.emit(f"{c} += 1;", indent + 1)
        self.scopes.append([Var(c, "s64", mutable=False)])
        self.loop_prologue(indent + 1)
        if self.chance(0.3):
            self.emit(f"if {self.bool_expr(1)} {self.pick(['break', 'continue'])};", indent + 1)
        for _ in range(self.r.randint(1, 3)):
            self.stmt(indent + 1)
        self.scopes.pop()
        self.emit("}", indent)
        self.depth_budget += 1

    def for_array(self, indent):
        arrays = [v for v in self.vars() if isinstance(v.ty, tuple) and v.ty[0] in ("array", "view", "dyn")]
        if not arrays or self.in_pure:
            return self.assign(indent)
        v = self.pick(arrays)
        self.iterating.append(v.name)
        self.depth_budget -= 1
        elem = v.ty[1]
        by_ptr = v.mutable and is_int(elem) and v.ty[0] != "view" and self.chance(0.4)
        rev = "< " if self.chance(0.25) and not by_ptr else ""
        if by_ptr:
            self.emit(f"for * {v.name} {{", indent)
            self.emit(f"it.* {self.pick(['+=', '^=', '*='])} {self.int_literal(elem)};", indent + 1)
            self.emit("mix(cast(u64) it_index);", indent + 1)
        else:
            self.emit(f"for {rev}{v.name} {{", indent)
            self.scopes.append([Var("it", elem, mutable=False), Var("it_index", "s64", mutable=False)])
            for line in self.mix_of("it", elem):
                self.emit(line, indent + 1)
            for _ in range(self.r.randint(0, 2)):
                self.stmt(indent + 1)
            self.scopes.pop()
        self.emit("}", indent)
        self.depth_budget += 1
        self.iterating.pop()

    def switch(self, indent):
        self.depth_budget -= 1
        enum_places = [(p, n) for n in self.enums if not self.enums[n][2] for p in self.places(("enum", n))]
        if enum_places and self.chance(0.4):
            p, n = self.pick(enum_places)
            self.emit(f"if {self.realize(p)} == {{", indent)
            members = self.r.sample(self.enums[n][1], self.r.randint(1, len(self.enums[n][1])))
            labels = [f".{m}" for m, _ in members]
        else:
            t = self.pick(list(INTS))
            self.emit(f"if {self.expr(t, 2)} == {{", indent)
            lo, hi = rng_of(t)
            vals = sorted({self.r.randint(max(lo, -4), min(hi, 8)) for _ in range(self.r.randint(1, 4))})
            labels = [f"({v})" if v < 0 else str(v) for v in vals]
        if self.chance(0.6):
            labels.append("")
        for lab in labels:
            self.emit(f"case {lab};" if lab else "case;", indent)
            self.block(indent + 1, self.r.randint(1, 2))
        self.emit("}", indent)
        self.depth_budget += 1

    def print_stmt(self, indent):
        v = self.pick_place_any()
        if not v:
            return
        path, ty = v
        label = re.sub("[^A-Za-z0-9_]", "", path)
        if isinstance(ty, str) and is_float(ty):
            self.emit(f"print_float(\"{label}\", cast(float64) {path});", indent)
        elif isinstance(ty, tuple) and self.reaches(ty, "float32") or isinstance(ty, tuple) and self.reaches(ty, "float64"):
            # Aggregates holding floats could hold a NaN; print only their checksum contribution.
            self.emit(f"print(\"{label} has % elements or fields\\n\", {self.count_of(ty, path)});", indent)
        elif isinstance(ty, tuple) and ty[0] == "dyn":
            self.emit(f"print(\"{label} = % (count %)\\n\", {path}, {path}.count);", indent)
        else:
            fmt = self.pick(["%", "[%]", "%;"])
            self.emit(f"print(\"{label} = {fmt}\\n\", {path});", indent)
        if is_int(ty) and self.chance(0.5):
            self.emit(f"print(\"hex %\\n\", formatInt({path}, base = {self.pick([2, 8, 16, 36])}));", indent)

    def count_of(self, ty, path):
        if ty[0] in ("array", "view", "dyn"):
            return f"{path}.count"
        if ty[0] == "struct":
            return str(len(self.structs[ty[1]]))
        return "2"

    def call_stmt(self, indent):
        effect = [p for p in self.procs if not p.pure]
        if not effect:
            return self.assign(indent)
        p = self.pick(effect)
        args = self.call_args(p, 1)
        if p.ret is None:
            self.emit(f"{p.name}({args});", indent)
        else:
            self.declare(p.ret, indent, init=f"{p.name}({args})")

    def string_stmt(self, indent):
        if self.in_pure:
            return self.assign(indent)
        k = self.r.randint(0, 2)
        name = self.fresh("str")
        if k == 0:
            vals = [self.pick_place_any() for _ in range(self.r.randint(1, 3))]
            vals = [v for v in vals if v and self.printable(v[1])]
            if not vals:
                return
            fmt = "|".join("%" for _ in vals)
            self.emit(f"{name} := tprint(\"{fmt}\", {', '.join(p for p, _ in vals)});", indent)
        elif k == 1:
            self.emit(f"{name} := {self.string_literal()};", indent)
        else:
            t = self.pick(list(INTS))
            self.emit(f"{name} := tprint(\"%-%\", {self.expr(t, 2)}, {self.string_expr()});", indent)
        self.scopes[-1].append(Var(name, "string", mutable=False))
        self.emit(f"mix_string({name});", indent)

    def printable(self, ty):
        return not (isinstance(ty, str) and is_float(ty)) and not (isinstance(ty, tuple) and (self.reaches(ty, "float32") or self.reaches(ty, "float64")))

    def pointer_stmt(self, indent):
        t = self.pick(list(INTS))
        targets = self.places(t, mutable_only=True)
        targets = [x for x in targets if "[" not in x]
        if not targets:
            return self.assign(indent)
        p = self.fresh("p")
        self.emit(f"{p} := *{self.pick(targets)};", indent)
        self.emit(f"{p}.* = {p}.* {self.pick(['+', '^', '*'])} {self.expr(t, 2)};", indent)
        if not self.in_pure:
            self.emit(f"mix(cast,no_check(u64) {p}.*);", indent)

    # ---- top level -----------------------------------------------------------------------------
    def gen_struct(self):
        name = self.fresh("S")
        fields = []
        for i in range(self.r.randint(1, 5)):
            ty = self.pick(SCALARS + list(INTS))
            default = self.leaf(ty) if self.chance(0.4) else None
            fields.append((f"f{i}", ty, default))
        if self.chance(0.4):
            fields.append(("arr", ("array", self.pick(list(INTS)), self.r.randint(1, 4)), None))
        if self.structs and self.chance(0.3):
            fields.append(("inner", ("struct", self.pick(self.structs)), None))
        self.out.append(f"{name} :: struct {{")
        for f, ty, default in fields:
            self.out.append(f"    {f}: {tyname(ty)}" + (f" = {default};" if default else ";"))
        self.out.append("}")
        self.out.append("")
        self.structs[name] = fields

    def gen_enum(self):
        name = self.fresh("E")
        flags = self.chance(0.3)
        base = self.pick(["u8", "u16", "s32", "u32", "s64"] if not flags else ["u8", "u16", "u32"])
        members = []
        value = -1
        lo, hi = rng_of(base)
        self.out.append(f"{name} :: {'enum_flags' if flags else 'enum'} {base} {{")
        for i in range(self.r.randint(1, 5)):
            m = f"{name}_{'ABCDEFG'[i]}"
            if not flags and self.chance(0.3):
                value = self.r.randint(max(lo, value + 1), min(hi, value + 50))
                self.out.append(f"    {m} :: {value};")
            else:
                value += 1
                self.out.append(f"    {m};")
            members.append((m, value))
        self.out.append("}")
        self.out.append("")
        self.enums[name] = (base, members, flags)

    def gen_proc(self, pure):
        name = self.fresh("pure_" if pure else "act_")
        params = []
        recursive = pure and self.chance(0.2)
        if recursive:
            params.append(("depth", "s64"))
        for i in range(self.r.randint(0, 4)):
            params.append((f"a{i}", self.random_type() if not pure or self.chance(0.7) else self.pick(SCALARS)))
        params = [(n, t) for n, t in params if not (isinstance(t, tuple) and t[0] == "dyn")]
        ret = self.pick(SCALARS + list(INTS)) if pure or self.chance(0.5) else None
        if pure and self.structs and self.chance(0.2):
            ret = ("struct", self.pick(self.structs))
        p = Proc(name, params, ret, pure)
        sig = ", ".join(f"{n}: {tyname(t)}" for n, t in params)
        inline = "inline " if self.chance(0.15) and not recursive else ""
        self.out.append(f"{name} :: {inline}({sig})" + (f" -> {tyname(ret)}" if ret else "") + " {")
        self.scopes = [[Var(g.name, g.ty, mutable=not pure) for g in self.globals] if not pure else [],
                       [Var(n, t, mutable=False) for n, t in params]]
        self.in_pure = pure
        self.depth_budget = 2
        self.scopes.append([])
        # Parameters are read-only; copy some into locals so statements can change them.
        for n, t in params[:2]:
            if n != "depth":
                self.declare(t, 1, init=n)
        for _ in range(self.r.randint(1, 6)):
            self.stmt(1)
        if recursive and ret is not None:
            rec_args = ["depth - 1"] + [n for n, _ in params[1:]]
            self.emit(f"if depth > 0 {{", 1)
            self.emit(f"inner := {name}({', '.join(rec_args)});", 2)
            self.emit(f"return inner;", 2)
            self.emit("}", 1)
        if ret is not None:
            # A recursive call inside its own body expression is not in `self.procs` yet, so it cannot recurse unboundedly.
            self.emit(f"return {self.expr(ret, 1)};", 1)
        self.out.append("}")
        self.out.append("")
        self.in_pure = False
        self.procs.append(p)

    def gen_poly(self):
        name = self.fresh("poly_")
        a, b = "a", "b"
        ops = []
        for _ in range(self.r.randint(1, 3)):
            ops.append(self.pick([f"({a} + {b})", f"({a} * {b})", f"({a} ^ {b})", f"({a} - {b})", f"(~{a} & {b})",
                                  f"({a} | cast(T) 3)", f"({a} >> 1)", f"({b} << 2)"]))
        body = ops[0]
        for o in ops[1:]:
            body = f"({body} {self.pick(['+', '^', '|'])} {o})"
        self.out.append(f"{name} :: (a: $T, b: T) -> T {{")
        if self.chance(0.5):
            self.out.append(f"    #if size_of(T) == 1 {{ return {body} ^ cast(T) 1; }}")
        self.out.append(f"    return {body};")
        self.out.append("}")
        self.out.append("")
        self.procs.append(Proc(name, [("a", None), ("b", None)], None, True, poly=True))

    def prelude(self):
        o = self.out
        o.append(f"// Generated by tools/jaigen.py, seed {self.seed}.")
        o.append('#import "Basic";')
        o.append("")
        o.append("H: u64 = 0xcbf29ce484222325;")
        o.append("mix :: (v: u64) { H = (H ^ v) * 0x100000001b3; }")
        o.append("mix_float :: (v: float64) {")
        o.append("    if v != v { mix(0x7ff8); return; }")
        o.append("    mix(cast,force(u64) v);")
        o.append("}")
        o.append("mix_string :: (s: string) { mix(cast(u64) s.count); for 0..s.count - 1 mix(cast(u64) s[it]); }")
        o.append("print_float :: (name: string, v: float64) {")
        o.append("    if v != v { print(\"% = nan\\n\", name); return; }")
        o.append("    print(\"% = %\\n\", name, v);")
        o.append("}")
        o.append("wrap_index :: (v: s64, n: s64) -> s64 { r := v % n; if r < 0 r += n; return r; }")
        o.append("string_byte :: (s: string, i: s64) -> u8 { if s.count == 0 return 7; return s[wrap_index(i, s.count)]; }")
        o.append("Pair :: struct (T: Type) { a: T; b: T; }")
        for t, (bits, signed) in INTS.items():
            if signed:
                o.append(f"safe_div_{t} :: (a: {t}, b: {t}) -> {t} {{ if b == 0 || b == -1 return a; return a / b; }}")
                o.append(f"safe_rem_{t} :: (a: {t}, b: {t}) -> {t} {{ if b == 0 || b == -1 return a; return a % b; }}")
            else:
                o.append(f"safe_div_{t} :: (a: {t}, b: {t}) -> {t} {{ if b == 0 return a; return a / b; }}")
                o.append(f"safe_rem_{t} :: (a: {t}, b: {t}) -> {t} {{ if b == 0 return a; return a % b; }}")
            for f in FLOATS:
                # NaN fails both comparisons and takes the fallback.
                o.append(f"f_to_{t}_{f} :: (v: {f}, lo: float64, hi: float64) -> {t} {{ if cast(float64) v >= lo && cast(float64) v <= hi return cast({t}) v; return {self.r.randint(0, 100)}; }}")
        for f in FLOATS:
            o.append(f"clamp_f_{f} :: (v: {f}) -> {f} {{ if v != v return 0.5; if v > 1e6 return 1e6; if v < -1e6 return -1e6; return v; }}")
        o.append("")

    def generate(self):
        self.prelude()
        self.nonempty_paths = set()
        self.no_index = False
        self.iterating = []
        self.fptrs = []  # (variable, Proc): procedure values, only in main
        self.scopes = [[]]
        for _ in range(self.r.randint(0, 3)):
            self.gen_enum()
        for _ in range(self.r.randint(0, 3)):
            self.gen_struct()
        for _ in range(self.r.randint(0, 2)):
            self.gen_poly()
        for _ in range(self.r.randint(1, 3)):
            ty = self.pick(SCALARS + list(INTS))
            g = Var(self.fresh("g"), ty)
            self.out.append(f"{g.name}: {ty} = {self.leaf(ty)};")
            self.globals.append(g)
        self.out.append("")
        nprocs = int(self.r.randint(2, 7) * self.size)
        for _ in range(nprocs):
            self.gen_proc(pure=self.chance(0.6))
        struct_consts = []
        for p in [p for p in self.procs if p.pure and isinstance(p.ret, tuple) and all(isinstance(t, str) and t != "string" for _, t in p.params)][:1]:
            self.scopes = [[]]
            args = [self.leaf(t) if n != "depth" else "1" for n, t in p.params]
            cname = self.fresh("CS")
            self.out.append(f"{cname} :: #run {p.name}({', '.join(args)});")
            struct_consts.append((cname, p, args))
        consts = []
        for p in [p for p in self.procs if p.pure and not p.poly and is_int(p.ret) and all(isinstance(t, str) and t != "string" for _, t in p.params)][:2]:
            self.scopes = [[]]
            self.depth_budget = 0
            args = [self.leaf(t) if n != "depth" else "2" for n, t in p.params]
            cname = self.fresh("CT")
            self.out.append(f"{cname} :: #run {p.name}({', '.join(args)});")
            consts.append((cname, p, args))
        self.out.append("")
        self.out.append("main :: () {")
        self.scopes = [[Var(g.name, g.ty) for g in self.globals], []]
        self.in_pure = False
        self.depth_budget = 3
        for _ in range(self.r.randint(2, 5)):
            self.declare(self.random_type(), 1)
        if self.chance(0.6):
            et = self.pick(list(INTS))
            d = self.declare(("dyn", et), 1)
            for _ in range(self.r.randint(1, 4)):
                self.emit(f"array_add(*{d}, {self.expr(et, 1)});", 1)
            self.nonempty_paths.add(d)
            arrays = [v for v in self.vars() if isinstance(v.ty, tuple) and v.ty[0] == "array"]
            if arrays and self.chance(0.6):
                a = self.pick(arrays)
                vname = self.fresh("view")
                self.emit(f"{vname}: [] {tyname(a.ty[1])} = {a.name};", 1)
                self.scopes[-1].append(Var(vname, ("view", a.ty[1]), mutable=False))
                self.nonempty_paths.add(vname)
        for p in self.r.sample([p for p in self.procs if p.pure and not p.poly and p.ret], min(2, len([p for p in self.procs if p.pure and not p.poly and p.ret]))):
            # Calls through a procedure value: an indirect call in compiled code.
            name = self.fresh("fp")
            self.emit(f"{name} := {p.name};", 1)
            self.fptrs.append((name, p))
        for cname, p, args in consts:
            # The same pure call at compile time (interpreter) and at run time (this backend).
            self.emit(f"if {cname} != {p.name}({', '.join(args)}) print(\"compile-time/run-time mismatch in {p.name}\\n\");", 1)
            self.emit(f"mix(cast,no_check(u64) {cname});", 1)
        for cname, p, args in struct_consts:
            # A struct computed by the compile-time interpreter against the same call at run time.
            ct, rt = self.fresh("ct"), self.fresh("rt")
            self.emit(f"{ct}: {tyname(p.ret)} = {cname};", 1)
            self.emit(f"{rt} := {p.name}({', '.join(args)});", 1)
            for f, fty, _ in self.structs[p.ret[1]]:
                if is_int(fty) or fty == "bool":
                    self.emit(f"if {ct}.{f} != {rt}.{f} print(\"compile-time/run-time mismatch in {p.name}.{f}\\n\");", 1)
            for line in self.mix_of(ct, p.ret) + self.mix_of(rt, p.ret):
                self.emit(line, 1)
        for _ in range(int(self.r.randint(8, 25) * self.size)):
            self.stmt(1)
        for v in list(self.vars()):
            for line in self.mix_of(v.name, v.ty):
                self.emit(line, 1)
        self.emit('print("checksum %\\n", formatInt(H, base = 16));', 1)
        self.out.append("}")
        return "\n".join(self.out) + "\n"


def generate(seed, size=1.0):
    return Gen(seed, size).generate()


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("seed", type=int)
    ap.add_argument("--size", type=float, default=1.0, help="scale the number of procedures and statements")
    ap.add_argument("--out")
    a = ap.parse_args()
    text = generate(a.seed, a.size)
    if a.out:
        with open(a.out, "w") as f:
            f.write(text)
    else:
        sys.stdout.write(text)


if __name__ == "__main__":
    main()
