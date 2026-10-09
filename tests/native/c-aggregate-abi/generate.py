#!/usr/bin/env python3
"""Generates shapes.c and shapes_N.jai for the c-aggregate-abi fixture, and variadics.c and
variadics.jai for the sibling c-variadic-aggregates fixture (see docs/native/c-abi.md).

Every shape gets: ret_ / check_ / call_ / apply_check_ (a C struct crossing the boundary as a
result and as an argument), and pass_ / passtail_ / passfar_ / passd_ (C calls a Jai `#c_call`
callback with the struct as an argument: first among few arguments, after five integers, after
seven integers, and after eight doubles). A shape marked big is only returned.

Run from this directory: python3 generate.py, then format the output with jaifmt (the pre-commit hook
requires it): jaifmt shapes_*.jai ../c-variadic-aggregates/variadics.jai
"""
import os

SCALARS = {
    "i8": ("signed char", "s8", False),
    "u8": ("unsigned char", "u8", False),
    "i16": ("short", "s16", False),
    "i32": ("int", "s32", False),
    "i64": ("long long", "s64", False),
    "f32": ("float", "float32", True),
    "f64": ("double", "float64", True),
}


def S(*members, pack=None):
    return ("struct", list(members), pack)


def U(*members):
    return ("union", list(members), None)


def A(elem, n):
    return ("arr", elem, n)


# (name, type, big)
SHAPES = [
    # Eightbyte classes, as plain structs.
    ("i64x2", S("i64", "i64"), False),
    ("i32i64", S("i32", "i64"), False),
    ("i8i64", S("i8", "i64"), False),
    ("i16i8i32", S("i16", "i8", "i32"), False),
    ("i64x3", S("i64", "i64", "i64"), False),
    ("i64x5", S("i64", "i64", "i64", "i64", "i64"), False),
    ("f32x2", S("f32", "f32"), False),
    ("f32x3", S("f32", "f32", "f32"), False),
    ("f32x4", S("f32", "f32", "f32", "f32"), False),
    ("f32x5", S("f32", "f32", "f32", "f32", "f32"), False),
    ("f64x2", S("f64", "f64"), False),
    ("f64x3", S("f64", "f64", "f64"), False),
    ("f64x4", S("f64", "f64", "f64", "f64"), False),
    ("i64f64", S("i64", "f64"), False),
    ("f64i64", S("f64", "i64"), False),
    ("i32f32", S("i32", "f32"), False),
    ("f32i32f64", S("f32", "i32", "f64"), False),
    ("f32x2i64", S("f32", "f32", "i64"), False),
    ("f64f32x2", S("f64", "f32", "f32"), False),
    ("u8x3", S("u8", "u8", "u8"), False),
    # Arrays, unions and nested structs.
    ("ai8x9", S(A("i8", 9)), False),
    ("ai32x3", S(A("i32", 3)), False),
    ("af32x2", S(A("f32", 2)), False),
    ("af64x3", S(A("f64", 3)), False),
    ("asmix", S(A(S("i32", "f32"), 2)), False),
    ("ui32f32", U("i32", "f32"), False),
    ("ui64f64", U("i64", "f64"), False),
    ("uf32x2i64", U(S("f32", "f32"), "i64"), False),
    ("nmix", S(S("i32", "f32"), "f64"), False),
    ("nf32", S(S("f32", "f32"), S("f32", "f32")), False),
    ("nsmall", S(S("i8"), S("i16"), S("i32")), False),
    ("nbig", S(S("i64", "i64"), S("f64", "f64")), False),
    # Packed (#no_padding, or a member `#align 1`): C's __attribute__((packed)). System V puts
    # a struct with an unaligned member in memory, however small.
    ("pk_i8i32", S("i8", "i32", pack="nopad"), False),
    ("pk_i8i32a", S("i8", "i32", pack="align1"), False),
    ("pk_i8i64", S("i8", "i64", pack="nopad"), False),
    ("pk_i8f64", S("i8", "f64", pack="align1"), False),
    ("pk_i8f32", S("i8", "f32", pack="nopad"), False),
    ("pk_f32x2", S("f32", "f32", pack="nopad"), False),
    ("pk_f32x2a", S("f32", "f32", pack="align1"), False),
    ("pk_i8i16i8i32", S("i8", "i16", "i8", "i32", pack="nopad"), False),
    ("pk_i16i32", S("i16", "i32", pack="align1"), False),
    ("pk_i8x3", S("i8", "i8", "i8", pack="nopad"), False),
    ("pk_i8i64x2", S("i8", "i64", "i64", pack="nopad"), False),
    ("pk_f64x4", S("f64", "f64", "f64", "f64", pack="nopad"), False),
    ("pk_i32f64", S("i32", "f64", pack="align1"), False),
    ("pk_f64i32", S("f64", "i32", pack="nopad"), False),
    ("pk_i32x2", S("i32", "i32", pack="nopad"), False),
    ("pk_f32f64", S("f32", "f64", pack="nopad"), False),
    ("pk_i8f32x2", S("i8", "f32", "f32", pack="nopad"), False),
    ("pk_ai32x2", S("i8", A("i32", 2), pack="nopad"), False),
    ("pk_nested", S("i8", S("i8", "i32", pack="nopad"), pack="nopad"), False),
    # A kilobyte comes back through a hidden pointer (the interpreter used to stop at 512 bytes).
    ("big1k", S(A("i64", 128)), True),
]

def build(ty, name, c_decls, jai_decls):
    """Declares `ty` (named `name` if it is an aggregate); returns its C type, its Jai type and
    its leaves as (access suffix, scalar) pairs."""
    if isinstance(ty, str):
        c, j, _ = SCALARS[ty]
        return c, j, [("", ty)]
    kind, members, pack = ty
    c_fields, j_fields, leaves = [], [], []
    align = " #align 1" if pack == "align1" else ""
    for i, m in enumerate(members):
        if isinstance(m, tuple) and m[0] == "arr":
            c, j, inner = build(m[1], f"{name}_f{i}", c_decls, jai_decls)
            c_fields.append(f"{c} f{i}[{m[2]}];")
            j_fields.append(f"f{i}: [{m[2]}] {j}{align};")
            found = [(f".f{i}[{k}]{s}", t) for k in range(m[2]) for s, t in inner]
        else:
            c, j, inner = build(m, f"{name}_f{i}", c_decls, jai_decls)
            c_fields.append(f"{c} f{i};")
            j_fields.append(f"f{i}: {j}{align};")
            found = [(f".f{i}{s}", t) for s, t in inner]
        # A union is written and checked through its first member only.
        if kind == "struct" or i == 0:
            leaves += found
    attr = " __attribute__((packed))" if pack else ""
    keyword = "union" if kind == "union" else "struct"
    c_decls.append(f"typedef {keyword}{attr} {{ {' '.join(c_fields)} }} {name};")
    tail = " #no_padding" if pack == "nopad" else ""
    body = "".join(f"    {f}\n" for f in j_fields)
    jai_decls.append(f"{name} :: {keyword} {{\n{body}}}{tail}\n")
    return name, name, leaves


def value(k, scalar, c):
    _, j, is_float = SCALARS[scalar]
    expr = f"seed + {k + 1}" if not is_float else f"seed + {k + 1}.5"
    return f"(({SCALARS[scalar][0]}) ({expr}))" if c else f"cast({j}) ({expr})"


def emit(shape):
    name, ty, big = shape
    decls = ([], [])
    _, _, leaves = build(ty, name, *decls)
    T = name
    c_sets = " ".join(f"s{a} = {value(k, t, True)};" for k, (a, t) in enumerate(leaves))
    c_checks = " && ".join(f"s{a} == {value(k, t, True)}" for k, (a, t) in enumerate(leaves))
    j_sets = "\n".join(f"    r{a} = {value(k, t, False)};" for k, (a, t) in enumerate(leaves))
    j_checks = " && ".join(f"r{a} == {value(k, t, False)}" for k, (a, t) in enumerate(leaves))
    longs = lambda n: ", ".join(["long long"] * n)
    c = []
    c += decls[0]
    c.append(f"{T} ret_{name}(int seed) {{\n    {T} s;\n    memset(&s, 0, sizeof s);\n    {c_sets}\n    return s;\n}}")
    c.append(f"int check_{name}({T} s, int seed) {{\n    return {c_checks};\n}}")
    c.append(f"{T} call_{name}({T} (*cb)(int), int seed) {{\n    return cb(seed);\n}}")
    c.append(f"int apply_check_{name}({T} (*cb)(int), int seed) {{\n    return check_{name}(cb(seed), seed);\n}}")
    if not big:
        c.append(f"int pass_{name}(int (*cb)({T}, int), int seed) {{\n    return cb(ret_{name}(seed), seed);\n}}")
        c.append(
            f"int passtail_{name}(int (*cb)({longs(5)}, {T}, int), int seed) {{\n"
            f"    return cb(1, 2, 3, 4, 5, ret_{name}(seed), seed);\n}}"
        )
        c.append(
            f"int passfar_{name}(int (*cb)({longs(7)}, {T}, double, int), int seed) {{\n"
            f"    return cb(1, 2, 3, 4, 5, 6, 7, ret_{name}(seed), 0.5, seed);\n}}"
        )
        c.append(
            f"int passd_{name}(int (*cb)(double, double, double, double, double, double, double, double, {T}, int), int seed) {{\n"
            f"    return cb(1, 2, 3, 4, 5, 6, 7, 8, ret_{name}(seed), seed);\n}}"
        )
    j = []
    j += decls[1]
    j.append(f"ret_{name} :: (seed: s32) -> {T} #foreign shapes;")
    j.append(f"call_{name} :: (cb: (seed: s32) -> {T} #c_call, seed: s32) -> {T} #foreign shapes;")
    j.append(f"apply_check_{name} :: (cb: (seed: s32) -> {T} #c_call, seed: s32) -> s32 #foreign shapes;")
    if not big:
        j.append(f"check_{name} :: (r: {T}, seed: s32) -> s32 #foreign shapes;")
        j.append(f"pass_{name} :: (cb: (r: {T}, seed: s32) -> s32 #c_call, seed: s32) -> s32 #foreign shapes;")
        longs_j = ", ".join(f"a{i}: s64" for i in range(1, 6))
        j.append(f"passtail_{name} :: (cb: ({longs_j}, r: {T}, seed: s32) -> s32 #c_call, seed: s32) -> s32 #foreign shapes;")
        longs_j = ", ".join(f"a{i}: s64" for i in range(1, 8))
        j.append(f"passfar_{name} :: (cb: ({longs_j}, r: {T}, d: float64, seed: s32) -> s32 #c_call, seed: s32) -> s32 #foreign shapes;")
        ds = ", ".join(f"d{i}: float64" for i in range(1, 9))
        j.append(f"passd_{name} :: (cb: ({ds}, r: {T}, seed: s32) -> s32 #c_call, seed: s32) -> s32 #foreign shapes;")
    j.append(f"\nmake_{name} :: (seed: s32) -> {T} #c_call {{\n    r: {T};\n{j_sets}\n    return r;\n}}")
    j.append(f"\nis_{name} :: (r: {T}, seed: s32) -> bool #c_call {{\n    return {j_checks};\n}}")
    if not big:
        j.append(f"\ncb_pass_{name} :: (r: {T}, seed: s32) -> s32 #c_call {{\n    return ifx is_{name}(r, seed) then 1 else 0;\n}}")
        args = ", ".join(f"a{i}: s64" for i in range(1, 6))
        ok = " && ".join(f"a{i} == {i}" for i in range(1, 6))
        j.append(
            f"\ncb_tail_{name} :: ({args}, r: {T}, seed: s32) -> s32 #c_call {{\n"
            f"    return ifx {ok} && is_{name}(r, seed) then 1 else 0;\n}}"
        )
        args = ", ".join(f"a{i}: s64" for i in range(1, 8))
        ok = " && ".join(f"a{i} == {i}" for i in range(1, 8))
        j.append(
            f"\ncb_far_{name} :: ({args}, r: {T}, d: float64, seed: s32) -> s32 #c_call {{\n"
            f"    return ifx {ok} && d == 0.5 && is_{name}(r, seed) then 1 else 0;\n}}"
        )
        ds = ", ".join(f"d{i}: float64" for i in range(1, 9))
        ok = " && ".join(f"d{i} == {i}" for i in range(1, 9))
        j.append(
            f"\ncb_dbl_{name} :: ({ds}, r: {T}, seed: s32) -> s32 #c_call {{\n"
            f"    return ifx {ok} && is_{name}(r, seed) then 1 else 0;\n}}"
        )
    checks = [
        f'    if !is_{name}(ret_{name}(7), 7) fail("{name}: foreign return");',
        f'    if !apply_check_{name}(make_{name}, 7) fail("{name}: callback return read by C");',
        f'    if !is_{name}(call_{name}(make_{name}, 9), 9) fail("{name}: callback return round trip");',
    ]
    if not big:
        checks += [
            f'    if !check_{name}(make_{name}(7), 7) fail("{name}: argument made in Jai");',
            f'    if !pass_{name}(cb_pass_{name}, 7) fail("{name}: callback argument");',
            f'    if !passtail_{name}(cb_tail_{name}, 7) fail("{name}: callback argument after five integers");',
            f'    if !passfar_{name}(cb_far_{name}, 7) fail("{name}: callback argument after seven integers");',
            f'    if !passd_{name}(cb_dbl_{name}, 7) fail("{name}: callback argument after eight doubles");',
        ]
    return "\n".join(c), "\n".join(j), "\n".join(checks)


# Shapes passed through `...` (C `va_arg`): System V and AAPCS64 treat them like fixed arguments,
# Apple's arm64 puts them all on the stack, Microsoft x64 passes anything over 8 bytes by
# reference, and Windows on arm64 ignores homogeneous float aggregates (everything goes in x0-x7).
VA_SHAPES = [
    ("i64x2", S("i64", "i64")),
    ("i8i32", S("i8", "i32")),
    ("i8x3", S("i8", "i8", "i8")),
    ("i32x5", S("i32", "i32", "i32", "i32", "i32")),
    ("i64x3", S("i64", "i64", "i64")),
    ("f32x2", S("f32", "f32")),
    ("f32x3", S("f32", "f32", "f32")),
    ("f32x4", S("f32", "f32", "f32", "f32")),
    ("f64x2", S("f64", "f64")),
    ("f64x4", S("f64", "f64", "f64", "f64")),
    ("i64f64", S("i64", "f64")),
    ("f64i64", S("f64", "i64")),
    ("f32i32", S("f32", "i32")),
    ("pk_i8i32", S("i8", "i32", pack="nopad")),
    ("pk_i8f64", S("i8", "f64", pack="align1")),
]


def emit_variadic(shape):
    name, ty = shape
    decls = ([], [])
    _, _, leaves = build(ty, name, *decls)
    T = name
    seeds = "seed"
    c_checks = " && ".join(f"s{a} == {value(k, t, True)}" for k, (a, t) in enumerate(leaves))
    j_sets = "\n".join(f"    r{a} = {value(k, t, False)};" for k, (a, t) in enumerate(leaves))
    longs = ", ".join(f"long long a{i}" for i in range(1, 8))
    c = list(decls[0])
    reader = (
        "    va_list ap;\n    va_start(ap, n);\n    int ok = 1;\n"
        "    for (int i = 0; i < n; i++) {\n"
        f"        int seed = 7 + i * 3;\n        {T} s = va_arg(ap, {T});\n"
        f"        ok = ok && {c_checks};\n    }}\n    va_end(ap);\n    return ok;\n"
    )
    c.append(f"int va_{name}(int n, ...) {{\n{reader}}}")
    c.append(f"int vaafter_{name}({longs}, int n, ...) {{\n{reader}}}")
    j = list(decls[1])
    j.append(f"va_{name} :: (n: s32, args: ..Any) -> s32 #foreign variadics;")
    fixed = ", ".join(f"a{i}: s64" for i in range(1, 8))
    j.append(f"vaafter_{name} :: ({fixed}, n: s32, args: ..Any) -> s32 #foreign variadics;")
    j.append(f"\nmake_{name} :: (seed: s32) -> {T} {{\n    r: {T};\n{j_sets}\n    return r;\n}}")
    checks = [
        f'    if !va_{name}(1, make_{name}(7)) fail("{name}: one variadic aggregate");',
        f'    if !va_{name}(3, make_{name}(7), make_{name}(10), make_{name}(13)) fail("{name}: three variadic aggregates");',
        f'    if !vaafter_{name}(1, 2, 3, 4, 5, 6, 7, 2, make_{name}(7), make_{name}(10)) fail("{name}: variadic aggregates after the registers");',
    ]
    return "\n".join(c), "\n".join(j), "\n".join(checks)


def main_variadic(here):
    cs = ["// Generated by generate.py: C functions reading aggregates through `va_arg`.\n"
          "#include <stdarg.h>\n"]
    js = ["// Generated by generate.py: aggregates passed to C through `...` (see docs/native/c-abi.md).\n"
          "#import \"Basic\";\n\nvariadics :: #library \"libvariadics\";\n\nfailures := 0;\n\n"
          "fail :: (what: string) {\n    failures += 1;\n    print(\"FAIL %\\n\", what);\n}\n"]
    checks = []
    for shape in VA_SHAPES:
        c, j, k = emit_variadic(shape)
        cs.append(c + "\n")
        js.append(j + "\n")
        checks.append(k)
    js.append("main :: () {\n" + "\n".join(checks) + "\n    if failures == 0 print(\"ok\\n\");\n}\n")
    os.makedirs(os.path.join(here, "..", "c-variadic-aggregates"), exist_ok=True)
    out = os.path.join(here, "..", "c-variadic-aggregates")
    with open(os.path.join(out, "variadics.c"), "w") as f:
        f.write("\n".join(cs))
    with open(os.path.join(out, "variadics.jai"), "w") as f:
        f.write("\n".join(js))


# The interpreter hands C at most 64 thunks per return shape, so the Jai side is split into
# programs of CHUNK shapes each; all of them call into the one C library.
CHUNK = 10


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    cs = ["// Generated by generate.py: C aggregates of every class, by value in both directions and as\n"
          "// arguments of Jai callbacks (see docs/native/c-abi.md).\n#include <string.h>\n"]
    programs = []
    for start in range(0, len(SHAPES), CHUNK):
        js = ["// Generated by generate.py: foreign calls with C aggregates, and `#c_call` Jai procedures that\n"
              "// C calls with them (interpreter and native).\n#import \"Basic\";\n\n"
              "shapes :: #library \"libshapes\";\n\nfailures := 0;\n\n"
              "fail :: (what: string) {\n    failures += 1;\n    print(\"FAIL %\\n\", what);\n}\n"]
        checks = []
        for shape in SHAPES[start : start + CHUNK]:
            c, j, k = emit(shape)
            cs.append(c + "\n")
            js.append(j + "\n")
            checks.append(k)
        js.append("main :: () {\n" + "\n".join(checks) + "\n    if failures == 0 print(\"ok\\n\");\n}\n")
        programs.append("\n".join(js))
    with open(os.path.join(here, "shapes.c"), "w") as f:
        f.write("\n".join(cs))
    main_variadic(here)
    for k, text in enumerate(programs):
        with open(os.path.join(here, f"shapes_{k}.jai"), "w") as f:
            f.write(text)


if __name__ == "__main__":
    main()
