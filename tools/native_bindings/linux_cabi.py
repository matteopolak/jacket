"""Render a Linux target's C headers (a clang JSON AST) as Jai declarations.

Used by `vendor_linux_posix.py` to give each non-x86-64 Linux CPU its own POSIX layouts,
symbols and constants straight from that CPU's libc headers. Widths come from clang's own
desugared types, so `long`, `size_t` and `time_t` are whatever the target says.
"""
import re

SCALARS = {
    'char': 'u8', 'signed char': 's8', 'unsigned char': 'u8',
    'short': 's16', 'unsigned short': 'u16',
    'int': 's32', 'unsigned int': 'u32',
    'long long': 's64', 'unsigned long long': 'u64',
    'float': 'float32', 'double': 'float64', '_Bool': 'bool', 'void': 'void',
}
QUALIFIERS = re.compile(r'\b(const|volatile|restrict|__restrict|__restrict__|_Atomic)\b')
ATTRIBUTES = re.compile(r'\s*__attribute__\(\((?:[^()]|\([^()]*\))*\)\)')
TAGS = re.compile(r'\b(struct|union|enum)\s+')


class Unsupported(Exception):
    """A C construct with no faithful Jai rendering; the caller reports and skips it."""


class Context:
    """Types of one translation unit: what to name, what to expand, how wide `long` is."""

    def __init__(self, ast, long_bytes, keep, renames=None, reached=None):
        self.long_bytes = long_bytes
        # `#define st_mtime st_mtim.tv_sec`: the spec names the member by the macro.
        self.renames = renames or {}
        # `#define sa_handler __sigaction_handler.sa_handler`: the members of these fields are
        # used as if the record held them directly, so the field is a `using` one.
        self.reached = reached or set()
        self.keep = keep  # typedef names the public spec declares; they stay names
        self.typedefs, self.records, self.enums, self.functions = {}, {}, {}, {}
        self.constants = {}
        self.owned = {}
        self.by_id = {}
        for node in ast['inner']:
            self.by_id[node.get('id')] = node
            kind, name = node['kind'], node.get('name')
            if kind == 'TypedefDecl':
                self.typedefs[name] = node
                owned = (node.get('inner') or [{}])[0].get('ownedTagDecl', {}).get('id')
                self.owned[name] = owned
            elif kind == 'RecordDecl' and name and node.get('completeDefinition'):
                self.records[(node['tagUsed'], name)] = node
            elif kind == 'EnumDecl':
                if name:
                    self.enums[name] = node
                for member in node.get('inner', []):
                    if member['kind'] == 'EnumConstantDecl':
                        value = folded(member)
                        if value is not None:
                            self.constants[member['name']] = value
            elif kind == 'FunctionDecl' and name:
                self.functions.setdefault(name, node)

    def inline(self, name):
        """A record the spec does not export, written out where it is used."""
        for tag in ('struct', 'union'):
            node = self.records.get((tag, name))
            if node:
                return record_body(self, node, '')
        return name

    def exported(self, tag, name):
        """The name the spec gives the record `tag name`: `name` itself, or `name_t` when `name`
        is something else the spec exports (`sigaction` is also a function)."""
        for candidate in (name + '_t', name):
            if candidate in self.keep and (tag == 'enum' or self.record_key(candidate) == (tag, name)):
                return candidate
        return None

    def record_key(self, name):
        """The (tag, C name) of the record the Jai name `name` stands for: the struct or union
        of that name, the one a typedef of it defines, or `struct <name minus _t>`."""
        for tag in ('struct', 'union'):
            if (tag, name) in self.records:
                return tag, name
        node = self.by_id.get(self.owned.get(name))
        if node and node['kind'] == 'RecordDecl' and node.get('completeDefinition'):
            self.records[(node['tagUsed'], name)] = node
            return node['tagUsed'], name
        for tag in ('struct', 'union'):
            if name.endswith('_t') and (tag, name[:-2]) in self.records:
                return tag, name[:-2]
        return None

    def scalar(self, name):
        if name in SCALARS:
            return SCALARS[name]
        if name in ('long', 'unsigned long'):
            return ('s' if name == 'long' else 'u') + str(self.long_bytes * 8)
        raise Unsupported('scalar ' + name)


def folded(node):
    """The integer an enum constant or constant expression folds to; None if clang left it."""
    if node['kind'] in ('ConstantExpr', 'IntegerLiteral', 'CharacterLiteral') and 'value' in node:
        return int(node['value'])
    for child in node.get('inner', []):
        value = folded(child)
        if value is not None:
            return value
    return None


def tag_of(text):
    return re.match(r'(?:struct|union|enum)\s+\w+\s*$', text.strip())


def split_function(text):
    """`ret (a, b)` or `ret (*)(a, b)` into (ret, [a, b]); None when `text` is no function."""
    text = ATTRIBUTES.sub('', text).strip()
    if text.endswith(']') or text.endswith('*'):
        return None
    depth, opens = 0, []
    for i, char in enumerate(text):
        if char == '(':
            if depth == 0:
                opens.append(i)
            depth += 1
        elif char == ')':
            depth -= 1
    if not opens or not text.endswith(')'):
        return None
    start = opens[0]
    head = text[:start].strip()
    if text[start:].startswith('(*'):
        # `ret (*)(args)`: the parameter list is the second group.
        if len(opens) < 2:
            return None
        start = opens[1]
    elif len(opens) > 1:
        return None
    args = text[start + 1:-1]
    parts, depth, current = [], 0, ''
    for char in args:
        depth += char in '(['
        depth -= char in ')]'
        if char == ',' and depth == 0:
            parts.append(current.strip())
            current = ''
        else:
            current += char
    if current.strip():
        parts.append(current.strip())
    return head, ([] if parts == ['void'] else parts)


def render(ctx, text, desugared=None):
    """Jai spelling of the C type `text` (clang's qualType), expanding typedefs the spec does
    not name through `desugared`."""
    tagged = re.match(r'(?:(?:const|volatile)\s+)*(struct|union|enum)\s+\w+\s*$', text)
    tag = tagged and tagged[1]
    text = ATTRIBUTES.sub('', text)
    spelled = re.sub(r'\s+', ' ', QUALIFIERS.sub('', text)).strip()  # keeps `struct`/`union`
    text = TAGS.sub('', QUALIFIERS.sub('', text)).strip()
    text = re.sub(r'\s+', ' ', text)
    if tagged:
        return ctx.exported(tag, text) or (text if text in ctx.enums else ctx.inline(text))
    if desugared is not None:
        desugared = re.sub(r'\s+', ' ', TAGS.sub('', QUALIFIERS.sub('', desugared)).strip())
    function = split_function(text)
    if function:
        ret, args = function
        names = ', '.join('unknown%d: %s' % (i, render(ctx, a)) for i, a in enumerate(args))
        variadic = names.replace('unknown%d: ...' % (len(args) - 1), '..Any') if args and args[-1] == '...' else names
        return '#type (%s) -> %s #c_call' % (variadic, render(ctx, ret))
    array = re.fullmatch(r'(.*?)\s*\[(\d*)\]', text)
    if array:
        count = array[2]
        if not count:
            raise Unsupported('flexible array')
        inner = desugared and re.fullmatch(r'(.*?)\s*\[\d*\]', desugared)
        return '[%s] %s' % (count, render(ctx, array[1], inner[1] if inner else None))
    if text.endswith('*'):
        pointee = text[:-1].strip()
        inner = desugared[:-1].strip() if desugared and desugared.endswith('*') else None
        if pointee == 'char' or pointee == 'unsigned char':
            return '*u8'
        if pointee == 'void':
            return '*void'
        bare = TAGS.sub('', pointee).strip()
        if bare not in ctx.keep and bare not in ctx.enums and bare not in ctx.typedefs \
                and bare not in SCALARS and bare not in ('long', 'unsigned long'):
            return '*void'
        return '*' + render(ctx, spelled[:-1].strip() if tag_of(spelled[:-1]) else pointee, inner)
    if text == '...':
        return '..Any'
    if text in ctx.keep or text in ctx.enums:
        return text
    if any(text == n for (_, n) in ctx.records):
        return ctx.inline(text)
    if text in SCALARS or text in ('long', 'unsigned long'):
        return ctx.scalar(text)
    if text in ctx.typedefs:
        owned = ctx.by_id.get(ctx.owned.get(text))
        if owned and owned['kind'] == 'RecordDecl' and owned.get('completeDefinition'):
            return record_body(ctx, owned, '')
        target = ctx.typedefs[text]['type']
        expanded = target.get('desugaredQualType', target['qualType'])
        if TAGS.sub('', QUALIFIERS.sub('', expanded)).strip() == text:
            raise Unsupported('typedef ' + text)
        return render(ctx, expanded)
    if desugared and desugared != text:
        return render(ctx, desugared)
    if text in ctx.enums or any(text == n for (_, n) in ctx.records):
        return text
    raise Unsupported('type ' + text)


def function_declaration(ctx, name):
    """`(params) -> ret #foreign` for the C function `name`, and the symbol it links to."""
    node = ctx.functions[name]
    params = [p for p in node.get('inner', []) if p['kind'] == 'ParmVarDecl']
    spelled = []
    for i, param in enumerate(params):
        ty = param['type']
        spelled.append('%s: %s' % (param.get('name') or 'unknown%d' % i,
                                   render(ctx, ty['qualType'], ty.get('desugaredQualType'))))
    split = split_function(node['type']['qualType'])
    if split is None:
        raise Unsupported('function type ' + node['type']['qualType'])
    ret, args = split
    if args and args[-1] == '...':
        spelled.append('__args: ..Any')
    result = render(ctx, ret)
    if result.startswith('#type'):
        result = '(' + result[len('#type '):] + ')'
    return spelled, result, node.get('mangledName', name)


def record_declaration(ctx, tag, name):
    """`name :: struct { ... }` for a complete C struct or union, nested records included."""
    node = ctx.records[(tag, name)]
    return name + ' :: ' + record_body(ctx, node, '')


def shape(ctx, name):
    """The Jai declaration of the C record the spec calls `name`, or None when there is none."""
    key = ctx.record_key(name)
    if key is None:
        return None
    return name + ' :: ' + record_body(ctx, ctx.records[key], '')


def record_body(ctx, node, indent):
    union = node['tagUsed'] == 'union'
    for child in node.get('inner', []):
        if child['kind'] in ('PackedAttr', 'AlignedAttr', 'MaxFieldAlignmentAttr'):
            raise Unsupported('record attribute ' + child['kind'])
    lines, pending, first, last, padded = ['struct {'], [], None, None, None
    for child in node.get('inner', []):
        kind = child['kind']
        if kind == 'RecordDecl' and not child.get('name'):
            pending.append(child)
        elif kind == 'FieldDecl':
            if child.get('isBitfield'):
                width = folded(child)
                if child.get('name') or width is None or width % 8:
                    raise Unsupported('bit-field ' + str(child.get('name')))
                if width:
                    # Padding after a scalar member extends that member's slot, so positional
                    # literals still list only the real members (`timespec.{ sec, nsec }`).
                    if padded and last is None and padded[0] == len(lines) - 1:
                        padded[2] += width // 8
                        lines[-1] = '%s    #overlay (%s) __pad%d: [%d] u8;' % (indent, padded[1], padded[3], padded[2])
                        continue
                    scalar = last and re.fullmatch(r'[su](8|16|32|64)', last[1])
                    if scalar and not union:
                        size = int(scalar[1]) // 8 + width // 8
                        padded = [len(lines), last[0], size, len(lines)]
                        lines.append('%s    #overlay (%s) __pad%d: [%d] u8;' % (indent, last[0], len(lines), size))
                        last = None
                    else:
                        lines.append('%s    __pad%d: [%d] u8;' % (indent, len(lines), width // 8))
                continue
            ty = child['type']
            spelling = ty['qualType']
            if 'unnamed' in spelling or 'anonymous' in spelling:
                if not pending:
                    raise Unsupported('nested record without a body')
                text = record_body(ctx, pending.pop(0), indent + '    ')
                member = child.get('name') and ctx.renames.get(child['name'], child['name'])
                if member is None:
                    member = '__anonymous%d' % len(lines)
                    lines.append(indent + '    using ' + member + ': ' + text + ';')
                    last = None
                    continue
                if member in ctx.reached:
                    prefix = '#overlay (%s) ' % first if union and first is not None else ''
                    lines.append('%s    %susing %s: %s;' % (indent, prefix, member, text))
                    first = member if first is None else first
                    last = None
                    continue
            else:
                text = render(ctx, spelling, ty.get('desugaredQualType'))
                member = ctx.renames.get(child['name'], child['name'])
            if union and first is not None:
                lines.append('%s    #overlay (%s) %s: %s;' % (indent, first, member, text))
            else:
                lines.append('%s    %s: %s;' % (indent, member, text))
                first = member if first is None else first
            last = (member, text)
    lines.append(indent + '}')
    return '\n'.join(lines)


def typedef_declaration(ctx, name):
    """`name :: <type>` for a typedef, expanded to the target's own widths."""
    target = ctx.typedefs[name]['type']
    return name + ' :: ' + render(ctx, target['qualType'], target.get('desugaredQualType'))


def layouts(dump):
    """{(tag, name): (size, align)} from `clang -Xclang -fdump-record-layouts` output."""
    result, current = {}, None
    for line in dump.splitlines():
        header = re.match(r'\s*0 \| (struct|union) (\w+)\s*$', line)
        if header:
            current = (header[1], header[2])
            continue
        sizes = re.match(r'\s*\| \[sizeof=(\d+), (?:dsize=\d+, )?align=(\d+)', line)
        if sizes and current:
            result.setdefault(current, (int(sizes[1]), int(sizes[2])))
            current = None
    return result
