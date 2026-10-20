"""Tiny self-contained programs, one per cell of a table of feature pairs.

Three tables, each an axis of `tools/differential/compare_compilers.py`'s `grid` set:

- `context`: value kinds whose type comes from context (`.{...}`, `.[...]`, bare literals, `.NAME`,
  `null`, `---`, a procedure literal) against typing contexts (a declaration, an argument, an operand of an
  operator overload, `return`, an `ifx` arm, ...). Each cell is generated twice when meaningful: a valid
  use and a near miss (wrong field count, a type mismatch).
- `poly`: overloaded and polymorphic procedure declarations against argument expressions.
- `using`: `using` forms against the ways a member is named and used.

Every program prints a deterministic line, so a disagreement in the value is caught too. To add a row or a
column, add an entry to the table; to add an axis, write a generator yielding `Program`s and list it in
`AXES`. Placeholders in the templates: `@TY@` the kind's type, `@RT@` the same type spelled through an
alias (for positions where a procedure type does not parse bare), `@V@` the context-typed value, `@O@` a
value of the type that needs no context, `@ALT@` another type that candidate overloads take, `@OP@` the
operator.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Iterator

from compare_core import Program

PRELUDE = '#import "Basic";\n'


@dataclass(frozen=True)
class Kind:
    name: str
    ty: str                  # the type the value is converted to
    ok: str                  # a valid value expression whose type comes from context
    other: str               # a typed value (no context needed)
    alt: str                 # a different type, for overload candidates
    decls: str = ''          # top-level declarations the kind needs
    bad: str | None = None   # a near miss; None when there is none
    show: str = 'print("%\\n", @R@);'   # prints a variable `@R@` of the kind's type
    caps: frozenset[str] = frozenset()
    ops: str = ''            # operator overloads on `ty` (needs cap `ops`)
    alt_ops: str = ''        # the same on a second type, for overload resolution of operators


@dataclass(frozen=True)
class Context:
    name: str
    body: str                          # statements in `main`; leaves the result in `res`
    decls: str = ''                    # top-level declarations
    needs: frozenset[str] = frozenset()    # kind needs one of these capabilities (empty: any kind)
    plain_result: bool = False         # `res` is not of the kind's type: print it as is
    uninit_ok: bool = False            # `---` is allowed here
    ops: tuple[str, ...] = ('',)       # expand `@OP@` over these operators
    uses_alt_ops: bool = False


def caps(*names: str) -> frozenset[str]:
    return frozenset(names)


STRUCT_OPS = '''operator + :: (x: V, y: V) -> V { return .{x.a + y.a, x.b + y.b}; }
operator * :: (x: V, y: V) -> V { return .{x.a * y.a, x.b * y.b}; }
operator == :: (x: V, y: V) -> bool { return x.a == y.a && x.b == y.b; }
operator * :: (x: V, f: float32) -> V { return .{x.a * cast(s32) f, x.b * cast(s32) f}; }
operator * :: (f: float32, x: V) -> V { return .{x.a * cast(s32) f, x.b * cast(s32) f}; }
'''
STRUCT_ALT_OPS = '''operator + :: (x: W, y: W) -> W { return .{x.a + y.a, x.b + y.b, x.c + y.c}; }
operator == :: (x: W, y: W) -> bool { return x.a == y.a; }
'''
STRUCT_DECLS = '''V :: struct { a: s32; b: s32; }
W :: struct { a: s32; b: s32; c: s32 = 9; }
Ty :: V;
'''

KINDS: tuple[Kind, ...] = (
    Kind('struct_positional', 'V', '.{3, 4}', 'V.{1, 1}', 'W', STRUCT_DECLS, bad='.{1, 2, 3}',
         caps=caps('ops'), ops=STRUCT_OPS, alt_ops=STRUCT_ALT_OPS),
    Kind('struct_named', 'V', '.{a = 3, b = 4}', 'V.{1, 1}', 'W', STRUCT_DECLS, bad='.{c = 1}',
         caps=caps('ops'), ops=STRUCT_OPS, alt_ops=STRUCT_ALT_OPS),
    Kind('struct_empty', 'V', '.{}', 'V.{1, 1}', 'W', STRUCT_DECLS, caps=caps('ops'), ops=STRUCT_OPS,
         alt_ops=STRUCT_ALT_OPS),
    Kind('array_literal', '[3] s32', '.[1, 2, 3]', 'og', '[3] s64',
         'og: [3] s32 = .[7, 8, 9];\nTy :: [3] s32;\n', bad='.[1, 2]'),
    Kind('int_literal', 's64', '7', 'oi', 's32', 'oi: s64 = 1;\nTy :: s64;\n', bad='7.5',
         caps=caps('num', 'eq', 'case', 'int')),
    Kind('int_literal_u8', 'u8', '200', 'oi', 's32', 'oi: u8 = 1;\nTy :: u8;\n', bad='300',
         caps=caps('num', 'eq', 'case', 'int')),
    Kind('int_literal_as_float', 'float64', '3', 'of', 'float32', 'of: float64 = 0.5;\nTy :: float64;\n', bad='true',
         caps=caps('num', 'eq')),
    Kind('float_literal', 'float64', '2.5', 'of', 'float32', 'of: float64 = 0.5;\nTy :: float64;\n', bad='"s"',
         caps=caps('num', 'eq')),
    Kind('enum_shorthand', 'E', '.B', 'E.A', 'E2', 'E :: enum { A; B; C; }\nE2 :: enum { A; B; C; D; }\nTy :: E;\n',
         bad='.Z', caps=caps('eq', 'case')),
    Kind('null_literal', '*s32', 'null', 'op', '*u8', 'og: s32 = 5;\nop: *s32 = *og;\nTy :: *s32;\n', bad='5',
         show='print("%\\n", @R@ == null);', caps=caps('eq')),
    Kind('uninitialized', 's32', '---', 'ou', 's64', 'ou: s32 = 4;\nTy :: s32;\n', show='print("ok\\n");'),
    Kind('procedure_literal', '(s32) -> s32', '(n: s32) -> s32 { return n + 1; }', 'inc', '(s32, s32) -> s32',
         'inc :: (n: s32) -> s32 { return n + 100; }\nTy :: (s32) -> s32;\n',
         bad='(n: s32, m: s32) -> s32 { return n; }', show='print("%\\n", @R@(4));'),
)

# `body` leaves its result in `res`. `ops` expands @OP@ into one context per operator.
CONTEXTS: tuple[Context, ...] = (
    Context('typed_declaration', 'res: @TY@ = @V@;', uninit_ok=True),
    Context('assignment', 'res: @TY@;\n    res = @V@;'),
    Context('argument', 'res := take(@V@);', 'take :: (p: @TY@) -> Ty { return p; }'),
    Context('argument_overloaded', 'res := take(@V@);',
            'take :: (p: @TY@) -> s32 { return 1; }\ntake :: (p: @ALT@) -> s32 { return 2; }', plain_result=True),
    Context('operator_left', 'res := @V@ @OP@ @O@;', needs=caps('ops'), plain_result=True, ops=('+', '*', '==')),
    Context('operator_right', 'res := @O@ @OP@ @V@;', needs=caps('ops'), plain_result=True, ops=('+', '*', '==')),
    Context('operator_overloaded_left', 'res := @V@ @OP@ @O@;', needs=caps('ops'), plain_result=True,
            ops=('+', '=='), uses_alt_ops=True),
    Context('operator_overloaded_right', 'res := @O@ @OP@ @V@;', needs=caps('ops'), plain_result=True,
            ops=('+', '=='), uses_alt_ops=True),
    Context('operator_scalar_left', 'res := @V@ @OP@ 2.0;', needs=caps('ops'), ops=('*',)),
    Context('operator_scalar_right', 'res := 2.0 @OP@ @V@;', needs=caps('ops'), ops=('*',)),
    Context('compound_assignment', 'res := @O@;\n    res @OP@ @V@;', needs=caps('ops', 'num'), ops=('+=', '*=')),
    Context('return', 'res := make();', 'make :: () -> Ty { return @V@; }'),
    Context('ifx_declared_then', 'flag := true;\n    res: @TY@ = ifx flag then @V@ else @O@;'),
    Context('ifx_declared_else', 'flag := false;\n    res: @TY@ = ifx flag then @O@ else @V@;'),
    Context('ifx_inferred_then', 'flag := true;\n    res := ifx flag then @V@ else @O@;'),
    Context('ifx_inferred_else', 'flag := false;\n    res := ifx flag then @O@ else @V@;'),
    Context('array_element_first', 'arr: [2] Ty = .[@V@, @O@];\n    res := arr[0];'),
    Context('array_element_last', 'arr: [2] Ty = .[@O@, @V@];\n    res := arr[1];'),
    Context('field_default', 'h: Holder;\n    res := h.f;', 'Holder :: struct { f: @TY@ = @V@; }', uninit_ok=True),
    Context('default_argument', 'res := take();', 'take :: (p: @TY@ = @V@) -> Ty { return p; }'),
    Context('cast_argument', 'res := cast(@TY@) @V@;'),
    Context('for_range_upper', 'n: s64;\n    for 0..@V@ { n += it; }\n    res := n;', needs=caps('int'),
            plain_result=True),
    Context('for_range_lower', 'n: s64;\n    for @V@..5 { n += it; }\n    res := n;', needs=caps('int'),
            plain_result=True),
    Context('comparison_left', 'res := @V@ == @O@;', needs=caps('eq'), plain_result=True),
    Context('comparison_right', 'res := @O@ == @V@;', needs=caps('eq'), plain_result=True),
    Context('case_value', 'x := @O@;\n    res: s32 = 0;\n    if x == {\n        case @V@; res = 1;\n        case; res = 2;\n    }',
            needs=caps('case'), plain_result=True),
)

OPERATOR_NAMES = {'+': 'plus', '*': 'times', '==': 'equal', '+=': 'plus_assign', '*=': 'times_assign', '': ''}


def fill(template: str, **values: str) -> str:
    for key, value in values.items():
        template = template.replace(f'@{key}@', value)
    return template


def context_cells() -> Iterator[Program]:
    for kind in KINDS:
        for context in CONTEXTS:
            if context.needs and not (kind.caps & context.needs):
                continue
            if kind.name == 'uninitialized' and not context.uninit_ok:
                continue
            for operator in context.ops:
                variants = [('valid', kind.ok)]
                if kind.bad is not None:
                    variants.append(('near_miss', kind.bad))
                for variant, value in variants:
                    yield context_program(kind, context, operator, variant, value)


def context_program(kind: Kind, context: Context, operator: str, variant: str, value: str) -> Program:
    values = {'TY': kind.ty, 'RT': 'Ty', 'V': value, 'O': kind.other, 'ALT': kind.alt, 'OP': operator}
    decls = kind.decls
    if 'ops' in kind.caps:
        decls += kind.ops + (kind.alt_ops if context.uses_alt_ops else '')
    decls += fill(context.decls, **values) + '\n' if context.decls else ''
    shown = 'print("%\\n", res);' if context.plain_result else fill(kind.show, R='res')
    body = fill(context.body, **values)
    source = f'{PRELUDE}{decls}main :: () {{\n    {body}\n    {shown}\n}}\n'
    operator_name = OPERATOR_NAMES[operator]
    context_name = f'{context.name}_{operator_name}' if operator_name else context.name
    return Program(f'grid_{kind.name}__{context_name}__{variant}', source,
                   ('axis:context', f'kind:{kind.name}', f'context:{context.name}', f'variant:{variant}',
                    *((f'operator:{operator_name}',) if operator_name else ())))


# ---- overload resolution against polymorphic parameters ----

@dataclass(frozen=True)
class Form:
    name: str
    decls: str
    call: str = 'f(@ARG@)'
    skip: tuple[str, ...] = ()    # arguments left out: the program is nondeterministic or hangs


FORMS: tuple[Form, ...] = (
    Form('polymorphic_vs_integer', 'f :: (x: $T) -> s32 { return 1; }\nf :: (x: s64) -> s32 { return 2; }'),
    Form('polymorphic_vs_float', 'f :: (x: $T) -> s32 { return 1; }\nf :: (x: float64) -> s32 { return 2; }'),
    Form('polymorphic_vs_struct', 'f :: (x: $T) -> s32 { return 1; }\nf :: (x: V) -> s32 { return 2; }'),
    Form('integer_vs_float', 'f :: (x: s64) -> s32 { return 1; }\nf :: (x: float64) -> s32 { return 2; }'),
    Form('struct_vs_wider_struct', 'f :: (x: V) -> s32 { return 1; }\nf :: (x: W) -> s32 { return 2; }'),
    Form('polymorphic_only', 'f :: (x: $T) -> s64 { return size_of(T); }'),
    Form('polymorphic_return', 'f :: (x: $T) -> T { return x; }'),
    Form('polymorphic_slice', 'f :: (x: [] $T) -> s32 { return cast(s32) x.count; }'),
    Form('polymorphic_pointer', 'f :: (x: *$T) -> s32 { return 1; }\nf :: (x: $T) -> s32 { return 2; }'),
    Form('polymorphic_pair', 'f :: (x: $T, y: T) -> s32 { return 1; }', 'f(@ARG@, @ARG@)'),
    Form('polymorphic_default', 'f :: (x: $T = 5) -> s32 { return 1; }'),
    Form('two_polymorphic', 'f :: (x: $T) -> s32 { return 1; }\nf :: (x: $U) -> s32 { return 2; }'),
    Form('constant_parameter', 'f :: ($N: s32) -> s32 { return N * 2; }', skip=('string',)),
)

ARGS: tuple[tuple[str, str], ...] = (
    ('integer', '5'),
    ('float', '5.5'),
    ('string', '"text"'),
    ('typed_struct', 'V.{1, 2}'),
    ('anonymous_struct', '.{1, 2}'),
    ('anonymous_array', '.[1, 2, 3]'),
    ('typed_array', 's32.[1, 2, 3]'),
    ('null', 'null'),
    ('narrow_cast', 'cast(u8) 3'),
    ('bool', 'true'),
)


def poly_cells() -> Iterator[Program]:
    prelude = f'{PRELUDE}{STRUCT_DECLS}'
    for form in FORMS:
        for arg_name, arg in ARGS:
            if arg_name in form.skip:
                continue
            call = fill(form.call, ARG=arg)
            source = f'{prelude}{form.decls}\nmain :: () {{\n    res := {call};\n    print("%\\n", res);\n}}\n'
            yield Program(f'grid_poly_{form.name}__{arg_name}', source,
                          ('axis:poly', f'form:{form.name}', f'argument:{arg_name}'))


# ---- `using` against name lookup ----

@dataclass(frozen=True)
class Setup:
    name: str
    decls: str
    body: str                                  # makes the member reachable
    names: tuple[tuple[str, str], ...]         # (label, expression naming the member `x`)
    proc: str = ''                             # when set, the statements run inside this procedure body wrapper


INNER = 'Inner :: struct { x: s32; y: s32; }\n'

SETUPS: tuple[Setup, ...] = (
    Setup('local_using', INNER + 'Outer :: struct { using i: Inner; z: s32; }',
          'using o: Outer;\n    o.x = 1;', (('bare', 'x'), ('qualified', 'o.x'))),
    Setup('parameter_using', INNER + 'Outer :: struct { using i: Inner; z: s32; }',
          'o: Outer;\n    o.x = 1;\n    run(o);', (('bare', 'x'), ('qualified', 'o.x')), proc='o: Outer'),
    Setup('field_using_qualified', INNER + 'Outer :: struct { using i: Inner; z: s32; }',
          'o: Outer;\n    o.x = 1;', (('through_field', 'o.i.x'), ('promoted', 'o.x'))),
    Setup('nested_using', INNER + 'Mid :: struct { using i: Inner; }\nOuter :: struct { using m: Mid; }',
          'o: Outer;\n    o.x = 1;', (('promoted', 'o.x'), ('through_fields', 'o.m.i.x'))),
    Setup('two_structs_same_member', 'A :: struct { x: s32; }\nB :: struct { x: s32; }\n'
          'Outer :: struct { using a: A; using b: B; }', 'o: Outer;', (('promoted', 'o.x'), ('qualified', 'o.a.x'))),
    Setup('shadowing_local', INNER + 'Outer :: struct { using i: Inner; }',
          'using o: Outer;\n    o.x = 1;\n    x := 100;', (('bare', 'x'), ('qualified', 'o.x'))),
    Setup('shadowing_global', INNER + 'Outer :: struct { using i: Inner; }\nx := 50;',
          'using o: Outer;\n    o.x = 1;', (('bare', 'x'), ('qualified', 'o.x'))),
    Setup('pointer_using', INNER + 'Outer :: struct { using i: Inner; }',
          'o: Outer;\n    o.x = 1;\n    using p := *o;', (('bare', 'x'), ('qualified', 'p.x'))),
    Setup('enum_using', 'E :: enum { x; y; }', 'using E;', (('bare', 'x'), ('qualified', 'E.x'))),
)

ACCESSES: tuple[tuple[str, str], ...] = (
    ('read', 'print("%\\n", @X@);'),
    ('write', '@X@ = 9;\n    print("%\\n", @X@);'),
    ('address', 'ptr := *@X@;\n    print("%\\n", ptr.*);'),
    ('compound', '@X@ += 5;\n    print("%\\n", @X@);'),
)


def using_cells() -> Iterator[Program]:
    for setup in SETUPS:
        for label, expression in setup.names:
            for access_name, access in ACCESSES:
                if setup.name == 'enum_using' and access_name != 'read':
                    continue
                statement = fill(access, X=expression)
                if setup.proc:
                    wrapper = f'run :: (using {setup.proc}) {{\n    {statement}\n}}\n'
                    body = setup.body
                    source = f'{PRELUDE}{setup.decls}\n{wrapper}main :: () {{\n    {body}\n}}\n'
                else:
                    source = f'{PRELUDE}{setup.decls}\nmain :: () {{\n    {setup.body}\n    {statement}\n}}\n'
                yield Program(f'grid_using_{setup.name}__{label}__{access_name}', source,
                              ('axis:using', f'setup:{setup.name}', f'naming:{label}', f'access:{access_name}'))


AXES = {'context': context_cells, 'poly': poly_cells, 'using': using_cells}


def grid(axes: list[str] | None = None) -> list[Program]:
    """Every cell of the named axes (default all), in a stable order."""
    chosen = axes or list(AXES)
    unknown = [a for a in chosen if a not in AXES]
    if unknown:
        raise ValueError(f'unknown grid axis {unknown[0]!r}; axes are {", ".join(AXES)}')
    return [program for axis in chosen for program in AXES[axis]()]
