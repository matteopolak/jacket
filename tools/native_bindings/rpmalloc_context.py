"""Normalize complete reviewed contextual record prose, never native Jai input.

Unknown untyped fields/conditions and new record families fail generation.
Field order remains the reviewed order; declaration order follows dependencies.
"""
import re

ORDER = ('Heap_Size_Class', 'Span_Cache', 'Span_Large_Cache', 'Span_Use',
         'Size_Class_Use', 'Span', 'Heap')
RAW_NAMES = dict(zip(ORDER, ('heap_size_class_t', 'span_cache_t',
                 'span_large_cache_t', 'span_use_t', 'size_class_use_t',
                 'span_t', 'heap_t')))
CONDITIONS = {'ENABLE_THREAD_CACHE', 'ENABLE_STATISTICS',
              'ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS',
              'RPMALLOC_FIRST_CLASS_HEAPS'}


def widths(text):
    series = r'((?:`\w+`[,\s]*(?:and\s+)?)+)'
    def expand(match, exception=False):
        names = re.findall(r'`(\w+)`', match[1])
        return ', '.join('`'+name+': '+(match[4] if exception and name==match[3] else match[2])+'`' for name in names)
    text = re.sub(series+r'\s*\(each `(\w+)` except `(\w+): (\w+)`\)',
                  lambda m: expand(m, True), text)
    return re.sub(series+r'\s*\(`(\w+)` each\)', expand, text)


def fields(text, initial=''):
    rows = []
    pending = []
    condition = initial
    for token in re.findall(r'`([^`]+)`', widths(text)):
        if token in CONDITIONS:
            if pending:
                raise ValueError('Untyped fields before condition: '+repr(pending))
            condition = token
            continue
        if ':' not in token:
            if not re.fullmatch(r'[a-z]\w*', token):
                raise ValueError('Unrecognized contextual token: '+token)
            pending.append(token)
            continue
        names, type_name = token.split(':', 1)
        names = pending + [name.strip() for name in names.split(',')]
        pending = []
        if not all(re.fullmatch(r'\w+', name) for name in names):
            raise ValueError('Invalid contextual member list: '+token)
        rows.extend((name, type_name.strip(), condition) for name in names)
    if pending:
        raise ValueError('Untyped contextual fields: '+repr(pending))
    if len({name for name, _, _ in rows}) != len(rows):
        raise ValueError('Repeated contextual fields')
    return rows


def records_in(section):
    headers = list(re.finditer(r'`(\w+)`(?: \(only under `([^`]+)`\))? has ', section))
    records = {}
    for i, header in enumerate(headers):
        end = headers[i+1].start() if i+1 < len(headers) else len(section)
        records[header[1]] = fields(section[header.end():end], header[2] or '')
    return records


def rewrite_records(spec):
    section = spec.split('### Jai rewrite heap (`USE_RAW := false`)', 1)[1].split('### Raw heap', 1)[0]
    records = records_in(section)
    if set(records) != set(ORDER):
        raise ValueError('Changed contextual record family: '+repr(set(records)))
    return records


def raw_records(spec, rewrite):
    section = spec.split('### Raw heap (`USE_RAW := true`)', 1)[1].split('## Contracts', 1)[0]
    paragraph = section.strip().split('\n\n')[0]
    raw = {'heap_t': fields(paragraph.split('It has ', 1)[1])}
    helpers = section.split('\n\n', 2)[2].split('`span_t` has ', 1)[0]
    raw.update(records_in(helpers))
    if set(raw) != {'heap_t', 'heap_size_class_t', 'span_cache_t', 'span_large_cache_t'}:
        raise ValueError('Changed explicit raw contextual records')
    if '`span_t` has the same member order and names as `Span`' not in section:
        raise ValueError('Raw span no longer delegates member order')
    if '`span_use_t` and `size_class_use_t` have the same field names' not in section:
        raise ValueError('Raw counter records no longer delegate fields')
    span_clause = section.split('`span_t` has ',1)[1].split(', with ',1)[1].split(' uses the bit values',1)[0]
    span_overrides = {name:ty for name,ty,_ in fields(span_clause)}
    if set(span_overrides) - {name for name,_,_ in rewrite['Span']}:
        raise ValueError('Raw span adds undelegated fields')
    raw['span_t'] = [(name,span_overrides.get(name, re.sub(r'\b(Heap|Span)\b', lambda m: RAW_NAMES[m[0]], ty)),condition)
                     for name,ty,condition in rewrite['Span']]
    counter_types = re.search(r'with `(\w+)` for counters except (.*?), which are `(\w+)`', section)
    if not counter_types:
        raise ValueError('Missing raw counter type contract')
    exceptions = set(re.findall(r'`(\w+)`',counter_types[2]))
    if exceptions - {field for name in ('Span_Use','Size_Class_Use') for field,_,_ in rewrite[name]}:
        raise ValueError('Unknown raw counter type exception')
    for name in ('Span_Use','Size_Class_Use'):
        raw[RAW_NAMES[name]] = [(field, counter_types[3] if field in exceptions else counter_types[1], condition)
                                for field,_,condition in rewrite[name]]
    return raw


def render_record(name, rows):
    lines = [name+' :: struct {']
    previous = ''
    outer = 'ENABLE_STATISTICS' if name in ('Size_Class_Use','size_class_use_t') else ''
    for field, ty, condition in rows:
        if condition == outer:
            condition = ''
        if condition != previous:
            if previous: lines.append('    }')
            if condition: lines.append('    #if '+condition+' {')
        lines.append(('        ' if condition else '    ')+field+': '+ty+';')
        previous = condition
    if previous: lines.append('    }')
    lines.append('}')
    code = '\n'.join(lines)
    if outer:
        code = '#if ENABLE_STATISTICS {\n'+code+'\n}'
    return code


def render_context(spec):
    rewrite = rewrite_records(spec)
    raw = raw_records(spec, rewrite)
    lines = ['// Generated from complete reviewed rpmalloc contextual record contracts.',
             '// Regenerate with tools/native_bindings/generate.py; do not hand-edit.',
             '// Private canonical types; never cast native SDK heap/span storage.',
             '#scope_module', '#if !USE_RAW {']
    lines.extend(render_record(name, rewrite[name]) for name in ORDER)
    lines += ['}', '#if USE_RAW {',
              'size_t :: u64;\n_int :: s32;\nint32_t :: s32;',
              'atomic32_t :: s32;\natomic64_t :: s64;\natomicptr_t :: *void;']
    # Reviewed source has no FIRST_CLASS_HEAPS parameter/default. Independent
    # packaging policy follows the official C default; this is not a Jai claim.
    lines += ['// Independent raw packaging policy: no undeclared public option.',
              'RPMALLOC_FIRST_CLASS_HEAPS :: false;']
    lines.extend(render_record(RAW_NAMES[name], raw[RAW_NAMES[name]]) for name in ORDER)
    lines += ['}']
    return '\n\n'.join(lines)+'\n'
