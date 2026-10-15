#!/usr/bin/env python3
"""Generate native ABI facades exclusively from the reviewed public specs."""
import argparse
from collections import Counter, defaultdict
from dataclasses import replace
import hashlib
import json
from pathlib import Path
import re
import subprocess

from spec import parse_spec
from config import IMPORTS, LIBRARIES, PARAMETERS, PRIVATE, TARGETS, NATIVE_ABI_ADAPTERS, NATIVE_SHAPE_DELEGATES
from wrappers import simple_body, forwarding_body
from objc import materialize_methods
from bitfields import body as bitfield_body
from spec import Declaration, Shape
from rpmalloc_context import render_context
from conditions import possible

ROOT = Path(__file__).resolve().parents[2]
MANUAL_FACADE_SOURCES = {
    'rpmalloc': ('configuration.jai', 'wrappers.jai', 'allocator.jai', 'statistics.jai'),
}


def format_source(code, formatter):
    if not formatter:
        return code
    result = subprocess.run([str(formatter.resolve()), '--stdin', '--config',
                             str(ROOT/'jaifmt.toml')], input=code,
                            capture_output=True, text=True, check=True)
    return result.stdout
MODULES = {
    'Android': 'android', 'Android/AAudio': 'android-aaudio', 'Android/EGL': 'android-egl',
    'Curl': 'curl', 'd3d11': 'd3d11', 'd3d12': 'd3d12', 'd3d_compiler': 'd3d-compiler',
    'dxc_compiler': 'dxc-compiler', 'dxgi': 'dxgi', 'freetype': 'freetype', 'GL': 'gl',
    'ImGui': 'imgui', 'iOS': 'ios', 'Linux': 'linux', 'lz4': 'lz4', 'macOS': 'macos',
    'meshoptimizer': 'meshoptimizer', 'Metal': 'metal', 'MojoShader': 'mojoshader',
    'nvidia_aftermath': 'nvidia-aftermath', 'nvtt': 'nvtt', 'Objective_C': 'objective-c',
    'POSIX': 'posix', 'rpmalloc': 'rpmalloc', 'SDL': 'sdl', 'stb_image': 'stb-image',
    'stb_image_resize': 'stb-image-resize', 'stb_image_write': 'stb-image-write',
    'stb_vorbis': 'stb-vorbis', 'telemetry3': 'telemetry3', 'Vulkan': 'vulkan',
    'Windows': 'windows', 'X11': 'x11', 'xxHash': 'xxhash',
}
COM_MODULES = {'d3d11', 'd3d12', 'd3d_compiler', 'dxc_compiler', 'dxgi', 'Windows', 'nvtt'}


def split_arguments(text):
    parts, start, depth, quoted = [], 0, 0, False
    for i, c in enumerate(text):
        if c == '"' and (i == 0 or text[i - 1] != '\\'):
            quoted = not quoted
        if quoted:
            continue
        if c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
        elif c == ',' and depth == 0:
            parts.append(text[start:i].strip())
            start = i + 1
    if text[start:].strip():
        parts.append(text[start:].strip())
    return parts


def procedure_parts(code):
    match = re.match(r'(.+?)\s*::\s*(?:inline\s+)?\(', code)
    if not match:
        return None
    start, depth = match.end(), 1
    for i in range(start, len(code)):
        if code[i] == '(':
            depth += 1
        elif code[i] == ')':
            depth -= 1
            if not depth:
                params = split_arguments(code[start:i])
                if params and not any(':' in p for p in params):
                    return None
                names = [p.split(':', 1)[0].strip().removeprefix('using ') for p in params]
                return match[1].strip(), params, names, code[i + 1:]
    raise ValueError('Unclosed procedure signature: ' + code)


def mask_literals_and_comments(code):
    """Retain offsets/newlines while hiding non-syntactic braces, including nested comments."""
    masked = list(code)
    i = 0
    while i < len(code):
        start = i
        if code.startswith('//', i):
            i = code.find('\n', i)
            if i < 0:
                i = len(code)
        elif code.startswith('/*', i):
            depth = 1
            i += 2
            while i < len(code) and depth:
                if code.startswith('/*', i):
                    depth += 1
                    i += 2
                elif code.startswith('*/', i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            if depth:
                raise ValueError('Unclosed comment in reviewed aggregate shape')
        elif code[i] == '"':
            i += 1
            while i < len(code) and code[i] != '"':
                i += 2 if code[i] == '\\' else 1
            if i >= len(code):
                raise ValueError('Unclosed string in reviewed aggregate shape')
            i += 1
        else:
            i += 1
            continue
        for offset in range(start, i):
            if masked[offset] != '\n':
                masked[offset] = ' '
    return ''.join(masked)


def normalize_member_lines(code):
    masked = mask_literals_and_comments(code)
    boundaries = list(re.finditer(r'(?<=[{;])[ \t]*(?=\w+\s*::\s*(?:inline\s+)?\()', masked))
    for match in reversed(boundaries):
        code = code[:match.start()]+'\n'+code[match.end():]
    # Public member lists may retain an opening procedure marker while
    # intentionally omitting its body. Normalize that prototype before type
    # scope tracking; forwarding below must still supply a real implementation.
    lines = []
    for line in code.splitlines():
        signature = line.strip().rstrip(' {;')
        if line.rstrip().endswith('{') and procedure_parts(signature):
            line = line.rstrip()[:-1].rstrip()
        lines.append(line)
    return '\n'.join(lines)


def member_declarations_with_owners(code, root, condition):
    """Nested nominal records keep their scope; anonymous/conditional braces add no name."""
    masked = mask_literals_and_comments(code)
    headers = re.finditer(r'(?:^|(?<=[;{}]))\s*(?:using\s+)?(\w+)\s*::\s*'
                          r'(?:struct|union|enum(?:_flags)?)\b[^{};]*\{', masked, re.M)
    names = {match.end()-1:match[1] for match in headers}
    stack, rows, offset = [], {}, 0
    for index, line in enumerate(code.splitlines(keepends=True)):
        signature = line.strip().rstrip(' {};')
        if procedure_parts(signature):
            owner = '.'.join(name for name in stack if name) or root
            rows[index] = (Declaration(signature,condition),owner)
        for match in re.finditer(r'[{}]',masked[offset:offset+len(line)]):
            if match[0] == '{':
                stack.append(names.get(offset+match.start()))
            elif stack:
                stack.pop()
            else:
                raise ValueError('Unbalanced closing brace in reviewed aggregate shape')
        offset += len(line)
    if stack:
        raise ValueError('Unbalanced aggregate shape braces: '+root)
    return rows


def guard(code, condition):
    return '#if ' + condition + ' {\n' + code + '\n}\n' if condition else code + '\n'


def normalized_signature(code):
    code = re.sub(r'("(?:\\.|[^"\\])*")|#discard\b',
                  lambda match: match[1] or '', code)
    tokens = re.findall(r'"(?:\\.|[^"\\])*"|[^\s"]+',code.strip().rstrip(';'))
    return ''.join(tokens)


def authored_signatures(text):
    signatures = set()
    for match in re.finditer(r'^[ \t]*\w+\s*::\s*(?:inline\s+)?\(',text,re.M):
        fragment = text[match.start():]
        parts = procedure_parts(fragment)
        if not parts:
            continue
        delimiters = [m for m in re.finditer(r'"(?:\\.|[^"\\])*"|[{};]', parts[3])
                      if not m[0].startswith('"')]
        if not delimiters or delimiters[0][0] != '{':
            continue
        if re.search(r'#foreign\b', parts[3][:delimiters[0].start()]):
            continue
        # Parameter defaults may contain aggregate literals. Only a brace in
        # the tail after the balanced parameter list starts the authored body.
        end = len(fragment)-len(parts[3])+delimiters[0].start()
        signatures.add(normalized_signature(fragment[:end]))
    return signatures


def condition_for(module, declaration, spec=None):
    condition = declaration.condition
    if module == 'rpmalloc' and condition == 'else branch':
        condition = '!ENABLE_STATISTICS'
    alias = re.fullmatch(r'\w+\s*::\s*(\w+)\s*;?', declaration.code.strip())
    if module == 'rpmalloc' and alias and alias[1] not in ('struct', 'union', 'enum', 'enum_flags') and declaration.name not in ('ENABLE_OVERRIDE', 'RPMALLOC_CONFIGURABLE'):
        # The alias inventory belongs to the raw transliteration. The rewrite
        # supplies its own procedures and keeps its contextual Heap private.
        condition = 'USE_RAW' + (' && ('+condition+')' if condition else '')
    elif module == 'rpmalloc' and procedure_parts(declaration.code) and declaration.name != 'rpmalloc_allocator_proc':
        # The reviewed prose explicitly separates the transliteration's size_t,
        # _int, opaque typedef spelling and prefixed entry points from rewrite
        # s64/contextual-Heap procedures. Shared spelling/signatures remain
        # shared; their raw aliases still identify the canonical raw procedure.
        raw = (declaration.name.startswith('rpmalloc_') or
               bool(re.search(r'\b(size_t|_int|rpmalloc_heap_t)\b',declaration.code)) or
               declaration.code.startswith('dump_statistics :: (file:'))
        rewrite = (bool(re.search(r'\b(s64|Heap|Span)\b|\*\$T',declaration.code)) or
                   declaration.name in ('initialize','initialize_config') and '-> s32' in declaration.code or
                   declaration.name == 'dump_statistics' and '#expand' in declaration.code)
        if raw or rewrite:
            condition = ('USE_RAW' if raw else '!USE_RAW') + (' && ('+condition+')' if condition else '')
            if raw and re.search(r'\b(?:rpmalloc_heap_t)\b', declaration.code) and 'RPMALLOC_FIRST_CLASS_HEAPS' not in condition:
                condition += ' && RPMALLOC_FIRST_CLASS_HEAPS'
    if condition == 'else branch':
        if module == 'Objective_C':
            condition = 'OS != .MACOS' if declaration.name == 'BOOL' else 'CPU != .X64'
        elif module == 'telemetry3':
            condition = 'ENABLED && OS == .WINDOWS'
        elif module == 'rpmalloc':
            condition = '!ENABLE_STATISTICS'
        else:
            raise ValueError(f'{module}: ambiguous else condition')
    if condition == 'GLX extension':
        condition = 'OS == .LINUX'
    if module == 'GL' and declaration.name == 'DumpGLErrors':
        condition = '!DUMP_GL_ERRORS' if '#discard' in declaration.code else 'DUMP_GL_ERRORS'
    if module == 'Android' and declaration.name in ('android_app_read_cmd','android_app_pre_exec_cmd','android_app_post_exec_cmd'):
        condition = '!USE_GLUE_NATIVE_PORT' if '#foreign' in declaration.code else 'USE_GLUE_NATIVE_PORT'
    if spec is not None and module in ('d3d11', 'd3d12'):
        debug_types = {s.name for s in spec.shapes if s.condition == 'INCLUDE_DEBUG_BINDINGS'}
        if any(declaration.name == t or declaration.name == t+'_VTable' or declaration.name.startswith(t+'_') or re.search(r'\*'+re.escape(t)+r'\b',declaration.code) for t in debug_types):
            condition = 'INCLUDE_DEBUG_BINDINGS'
    return condition


def com_forward(module, declaration, declarations=()):
    parts = procedure_parts(declaration.code)
    if module not in COM_MODULES or not parts:
        return None
    name, params, args, tail = parts
    if name == 'vtable' and len(args) == 1:
        return declaration.code + ' { return ' + args[0] + '.vtable; }'
    if args and args[0] == 'this':
        target = re.match(r'this:\s*\*([\w]+)', params[0])
        aliases = {d.name:match[1] for d in declarations
                   if (match := re.fullmatch(r'\w+\s*::\s*(\w+)',d.code))}
        def canonical(type_name):
            visited = set()
            while type_name in aliases and type_name not in visited:
                visited.add(type_name)
                type_name = aliases[type_name]
            return type_name
        prefixes = ([target[1]]+[alias for alias in aliases if canonical(alias)==canonical(target[1])]) if target else []
        owner = next((p for p in sorted(prefixes,key=len,reverse=True) if name.startswith(p+'_')),None)
        if owner:
            method = name[len(owner) + 1:]
            if method == 'Destructor':
                method = 'Destructor'
            call = 'this.vtable.' + method + '(' + ', '.join(args) + ');'
            ret = 'return ' if '->' in tail and not re.search(r'->\s*(?:void|\(\))\b', tail) else ''
            return declaration.code + ' { ' + ret + call + ' }'
    return None


def declaration_group(declaration):
    """Choose our facade architecture without changing ABI member order.

    Storage stays in inventory order because initializers can have effects.
    Same-name overloads stay stable; their resolution may depend on order.
    """
    code = declaration.code
    if re.search(r'::\s*(?:struct|union|enum(?:_flags)?)\b', code):
        return 0, 'Types and layouts'
    if procedure_parts(code):
        return (4, 'Native entry points') if '#foreign' in code else (5, 'Convenience operations')
    if re.match(r'\w+\s*:(?!:)', code):
        return 3, 'Native storage'
    if re.search(r'::\s*(?:#type|\*struct)\b', code):
        return 1, 'Callback and opaque types'
    return 2, 'Aliases and constants'


def organize_declarations(output):
    sections = defaultdict(list)
    for index, (declaration, code) in enumerate(output):
        rank, title = declaration_group(declaration)
        key = (index,) if rank == 3 else (declaration.name.casefold(), index)
        sections[(rank, title)].append((key, code))
    return '\n'.join('// ' + title + '\n\n' + '\n'.join(
        code for _, code in sorted(entries))
        for (_, title), entries in sorted(sections.items()))


def supplement_nvtt_members(owner, body, condition):
    metadata = json.loads((ROOT/'stdlib/internal/bindings/vendor/nvtt-members.json').read_text())
    target = re.fullmatch(r'OS == \.(WINDOWS|LINUX|MACOS)', condition)
    if not target:
        return body, []
    def key(code):
        parts = procedure_parts(code)
        return (parts[0], tuple(p.split(':',1)[1].split('=',1)[0].strip() for p in parts[1])) if parts else None
    existing = {key(line.strip().rstrip(';')) for line in body.splitlines()}
    additions, records = [], []
    for code in metadata['targets'][target[1]].get(owner, []):
        if key(code) in existing:
            continue
        # Keep native const/non-const receiver overloads even when their Jai
        # parameter types coincide; both declarations have observable symbols.
        additions.append('    '+code+';')
        records.append({'signature':code, 'condition':condition, 'status':'declared',
                        'source':'pinned NVTT public C++ header', 'header_sha256':metadata['header_sha256']})
    closing = body.rfind('}')
    return body[:closing]+'\n'+'\n'.join(sorted(additions, key=lambda line:line.split('::',1)[0].strip().casefold()))+'\n'+body[closing:], records


def generate_facade(module, spec):
    shapes = defaultdict(list)
    for shape in spec.shapes:
        shapes[shape.name].append(shape)
    native_shape_provenance = {}
    if module in NATIVE_SHAPE_DELEGATES:
        path = ROOT/'stdlib/internal/bindings/vendor'/NATIVE_SHAPE_DELEGATES[module]
        native = json.loads(path.read_text())
        for cpu, target in native['targets'].items():
            condition = target['condition']
            for shape in target['shapes']:
                name = shape['name']
                # The public inventory controls names and availability. Native
                # headers never introduce extra exports or override a reviewed
                # target shape (including an intentional public ABI mismatch).
                declarations = [d for d in spec.declarations if d.name==name and
                                re.search(r'::\s*struct\b',d.code) and
                                possible('('+condition+') && ('+(d.condition or 'true')+')')]
                reviewed = ' || '.join('('+(s.condition or 'true')+')' for s in shapes[name]) or 'false'
                if not declarations or not possible('('+condition+') && !('+reviewed+')'):
                    continue
                candidate = Shape(name,shape['code'],condition)
                shapes[name].append(candidate)
                native_shape_provenance[(name,condition)] = {
                    'metadata':str(path.relative_to(ROOT)),
                    'metadata_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
                    'archive_sha256':native['archive_sha256'],'revision':native['revision'],
                    'cpu':cpu,'verification':'C header/layout assertions; not Android Jai/runtime execution',
                    'layout':{k:shape[k] for k in ('size','alignment','fields')}}
    ledger, output, emitted = [], [], set()
    constants_path = ROOT/'stdlib/internal/bindings/vendor/constants.json'
    native_constants = json.loads(constants_path.read_text()) if constants_path.exists() else {}
    manual_paths = [ROOT/'stdlib'/module/name for name in MANUAL_FACADE_SOURCES.get(module, ('wrappers.jai',))]
    signature_sources = {signature: path for path in manual_paths if path.exists()
                         for signature in authored_signatures(path.read_text())}
    manual_signatures = set(signature_sources)
    alias_targets = {d.name:re.fullmatch(r'\w+\s*::\s*(\w+)\s*;?',d.code.strip())[1]
                     for d in spec.declarations
                     if re.fullmatch(r'\w+\s*::\s*(\w+)\s*;?',d.code.strip())}
    for d in spec.declarations:
        record = {'name': d.name, 'signature': d.code, 'condition': d.condition, 'spec_line': d.line}
        condition = condition_for(module, d, spec)
        if condition != d.condition:
            record['effective_condition'] = condition
        adapter = NATIVE_ABI_ADAPTERS.get((module, d.name))
        if adapter and '#foreign' in d.code:
            wrapper_signature = re.sub(r'#foreign\s+\w+(?:\s+"[^"]*")?\s*;?$', '#c_call', d.code)
            if normalized_signature(wrapper_signature) in manual_signatures:
                record.update(status='implemented_manual', artifact=str(signature_sources[normalized_signature(wrapper_signature)].relative_to(ROOT)),
                              native_abi_adapter=adapter)
                ledger.append(record)
                continue
            record.update(status='unresolved', reason='Required native ABI adapter body does not match the complete public signature',
                          native_abi_adapter=adapter)
            ledger.append(record)
            continue
        authored = normalized_signature(d.code) in manual_signatures
        alias_target = alias_targets.get(d.name) if module == 'rpmalloc' else None
        alias_view = bool(alias_target and procedure_parts(d.code) and
                          normalized_signature(re.sub(r'^\w+',alias_target,d.code)) in manual_signatures)
        if authored or alias_view:
            signature = normalized_signature(d.code) if authored else normalized_signature(re.sub(r'^\w+',alias_target,d.code))
            record.update(status='implemented_manual', artifact=str(signature_sources[signature].relative_to(ROOT)))
            if alias_view and not authored:
                record['canonical_alias_target'] = alias_target
            if module == 'rpmalloc':
                record.update(status='partially_implemented',
                              implemented_condition='Native_Profile_Matches' + (' && ('+condition+')' if condition else ''),
                              reason='Real native boundaries for twenty-four raw/rewrite/layout/statistics/cache profiles; adaptive-cache/assertion/page-advice and numeric build options remain incomplete')
            if module == 'Objective_C' and d.name in {
                    'objc_type_encode', 'objc_func_encode', 'type_check_objc_method_type',
                    'objc_create_class', 'objc_add_instance_method', 'objc_add_class_method'}:
                record.update(status='partially_implemented',
                              reason='Real native C-compatible encoding/class/method behavior; Jai-specific alias, ownership, naming, validation and unsupported-reflection contracts still need public review')
            if module == 'Android' and d.name == 'android_logger':
                record.update(status='partially_implemented',
                              reason='Canonical committed Runtime discard is wired; host validation awaits its supplied output-lock dependency')
            ledger.append(record)
            continue
        key = (condition, d.code)
        if key in emitted:
            record.update(status='duplicate', reason='Exact repeated inventory signature')
            ledger.append(record)
            continue
        emitted.add(key)
        code = d.code.rstrip(';')
        native_constant = native_constants.get(module+'/'+d.name)
        if native_constant and (re.search(r'\w+\.$',code) or code.endswith(':: .data')):
            code = native_constant['code']
            record['native_initializer'] = native_constant
        if re.search(r'\*struct$', code):
            code += ' {}'
        if re.search(r'\w+\.$', code):
            record.update(status='unresolved', reason='Reviewed inventory omits aggregate initializer contents')
            ledger.append(record)
            continue
        aggregate = re.search(r':(?::)?\s*(?:struct|union|enum(?:_flags)?)(?:\s|$)', code)
        if aggregate:
            candidates = shapes[d.name]
            exact = [s for s in candidates if s.condition == condition]
            selected = exact or [s for s in candidates if not s.condition]
            if module == 'nvtt' and not selected and condition == 'OS == .MACOS':
                # The contract explicitly delegates this target's member API
                # to the pinned native header. Both Unix targets use its
                # Itanium object ABI; member linker names come from Clang.
                selected = [replace(s, condition=condition) for s in candidates if s.condition == 'OS == .LINUX']
            if not selected and not condition and candidates and all(s.condition for s in candidates):
                selected = candidates
            base_condition = ' && '.join('('+c+')' for c in (TARGETS.get(module,''),condition) if c) or 'true'
            if not selected:
                selected = [s for s in candidates if possible('('+base_condition+') && ('+(s.condition or 'true')+')')]
            selected = [s for s in selected if possible('('+base_condition+') && ('+(s.condition or 'true')+')')]
            if not selected:
                record.update(status='unresolved', reason='No unambiguous shape for this configuration')
                ledger.append(record)
                continue
            blocks = []
            coverage = ' || '.join('('+(s.condition or 'true')+')' for s in selected)
            missing_shape = '('+base_condition+') && !('+coverage+')'
            partial = possible(missing_shape)
            if partial:
                record['unresolved_shape_condition'] = missing_shape
            for s in selected:
                body = s.code
                if (s.name,s.condition) in native_shape_provenance:
                    record.setdefault('native_shape_delegates',[]).append(native_shape_provenance[(s.name,s.condition)])
                if module in ('ImGui', 'nvtt'):
                    if module == 'ImGui' and d.name == 'TextFilter':
                        metadata = json.loads((ROOT/'stdlib/internal/bindings/vendor/imgui-symbols.json').read_text())
                        symbol = metadata['symbols']['windows-x64' if s.condition=='OS == .WINDOWS' else 'linux-x64']
                        body = re.sub(r'(\bsplit\s*::[^\n]+?#foreign imgui)(?=\s*[;}])',r'\1 "'+symbol+'"',body)
                        record['native_symbol_metadata'] = metadata
                    # Public C++ shapes can put several prototypes on a line.
                    # Split only at statement boundaries before a procedure;
                    # aggregate member and overload order remain unchanged.
                    body = normalize_member_lines(body)
                    if module == 'nvtt':
                        body, native_members = supplement_nvtt_members(d.name, body, s.condition)
                        record['native_header_members'] = native_members
                    member_rows = member_declarations_with_owners(body, d.name, s.condition)
                    member_declarations = [entry[0] for entry in member_rows.values()]
                    lines = []
                    for index, line in enumerate(body.splitlines()):
                        if index in member_rows:
                            declaration, owner = member_rows[index]
                            signature = declaration.code
                            parts = procedure_parts(signature)
                            member = {'signature': signature, 'condition': s.condition, 'owner': owner, 'status': 'declared'}
                            if '#foreign' not in signature:
                                wrapper = forwarding_body(module,Declaration(signature,s.condition),member_declarations,parts,procedure_parts)
                                if wrapper is not None:
                                    line = '    '+signature+' { '+wrapper+' }'
                                    member['status'] = 'implemented'
                                else:
                                    record.setdefault('unresolved_members',[]).append(signature)
                                    member['status'] = 'unresolved'
                            record.setdefault('members', []).append(member)
                        lines.append(line)
                    body = '\n'.join(lines)
                if module == 'd3d11':
                    lines = []
                    for line in body.splitlines():
                        parts = procedure_parts(line.strip().rstrip(' {;'))
                        bits = bitfield_body(parts) if parts else None
                        if bits is not None:
                            line = line.rstrip(' {;') + ' { ' + bits + ' }'
                        lines.append(line)
                    body = '\n'.join(lines)
                if module in ('Objective_C', 'Metal'):
                    body = materialize_methods(d.name, body, procedure_parts)
                if s.member_list:
                    # The prose renderer omits the terminal semicolon of an
                    # inline anonymous member and uses postfix overlay notation.
                    body = re.sub(r'(\w+):\s*([^;{}]+?)\s+#overlay\s+(\w+);',
                                  r'#overlay (\3) \1: \2;', body)
                    body = re.sub(r'([^;{}\s])\s+}', r'\1; }', body)
                if module == 'SDL' and d.name == 'Hack':
                    body = body.replace('K :: inline (scancode: SDL_Scancode) -> s32',
                                        'K :: inline (scancode: SDL_Scancode) -> s32 { return cast(s32) scancode | (1 << 30); }')
                # Member-list headers omit type widths and directives; inventory
                # headers preserve those. Complete shape blocks retain overlays.
                if body.startswith(d.name + ' :: ') and '{' in body:
                    old_header = body[:body.index('{')].strip()
                    if '(' not in old_header and not re.search(r'#(?:align|no_padding)', old_header):
                        body = code + ' ' + body[body.index('{'):]
                blocks.append(guard(body, s.condition if s.condition != condition else ''))
            code = '\n'.join(blocks)
            record['status'] = 'partially_declared' if partial else 'declared'
        elif '#foreign' in code or '#elsewhere' in code:
            code += ';'
            record['status'] = 'declared'
        elif procedure_parts(code):
            forwarded = com_forward(module, d, spec.declarations)
            if forwarded:
                code = forwarded
                record['status'] = 'forwarded'
            else:
                parts = procedure_parts(code)
                body = simple_body(module, d.name, condition)
                if body is None:
                    body = forwarding_body(module, d, spec.declarations, parts, procedure_parts)
                if body is not None:
                    code += ' { ' + body + ' }'
                    record['status'] = 'implemented'
                else:
                    record.update(status='unresolved', reason='Requires independently authored wrapper body')
                    ledger.append(record)
                    continue
        else:
            code += ';'
            record['status'] = 'declared'
        output.append((d, guard(code, condition)))
        ledger.append(record)
    return organize_declarations(output), ledger


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--formatter', type=Path, help='Own source-built jaifmt; normalize generated sources before comparison/write')
    parser.add_argument('--module', action='append', choices=MODULES)
    args = parser.parse_args()
    all_ledger = {}
    stale = []
    for module, slug in MODULES.items():
        page = ROOT / 'docs/specs/stdlib/modules' / (slug + '.md')
        spec = parse_spec(page.read_text())
        code, entries = generate_facade(module, spec)
        digest = hashlib.sha256(page.read_bytes()).hexdigest()
        all_ledger[module] = {'spec': str(page.relative_to(ROOT)), 'spec_sha256': digest,
                              'counts': dict(Counter(e['status'] for e in entries)),
                              'member_counts':dict(Counter(m['status'] for e in entries for m in e.get('members',[]))),
                              'contextual_shapes':spec.contextual_blocks,
                              'declarations': entries}
        if module == 'rpmalloc':
            backend = ROOT/'stdlib/internal/bindings/rpmalloc-sdk'
            provenance = backend/'provenance.json'
            if provenance.exists():
                native = json.loads(provenance.read_text())
                all_ledger[module]['supplemental_backends'] = [{
                    'module':'internal/bindings/rpmalloc-sdk',
                    'artifact':str((backend/'module.jai').relative_to(ROOT)),
                    'artifact_sha256':hashlib.sha256((backend/'module.jai').read_bytes()).hexdigest(),
                    'status':'private_native_backend',
                    'public_support':'Unresolved; this backend does not establish Jai callback/config/heap compatibility',
                    'provenance':native,
                    'verification':'Source/header inventory only here; actual private SDK native evidence is indexed separately'}]
                checked = ROOT/'stdlib/internal/bindings/rpmalloc-checked/module.jai'
                all_ledger[module]['supplemental_backends'][0]['checked_boundary'] = {
                    'artifact':str(checked.relative_to(ROOT)),
                    'artifact_sha256':hashlib.sha256(checked.read_bytes()).hexdigest(),
                    'signature':'posix_memalign :: (output: **void, alignment: u64, size: u64) -> s32',
                    'scope':'Private SDK precondition validation and commit-on-success output; not public API completion'}
                callback_backend = ROOT/'stdlib/internal/bindings/rpmalloc-callbacks'
                if callback_backend.exists():
                    all_ledger[module]['supplemental_backends'].append({
                        'module':'internal/bindings/rpmalloc-callbacks',
                        'status':'partial_public_profile_adapter',
                        'artifacts':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()
                                     for p in sorted(callback_backend.iterdir()) if p.is_file()},
                        'scope':'Owned callback/name lifetime, bounded wide public offset registry, actual init failure; raw/rewrite and both statistics profiles',
                        'limitations':'Adaptive-cache/assertion/validation/page-advice/numeric profiles and exact native-error Jai location provenance unresolved',
                        'caller_context_artifacts':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()
                                                    for p in sorted((ROOT/'stdlib/internal/bindings/callback-context').iterdir()) if p.is_file()},
                        'tests':['tests/native-bindings/rpmalloc-public-callbacks.jai',
                                 'tests/native-bindings/rpmalloc-public-configurable.jai']})
                heap_backend = ROOT/'stdlib/internal/bindings/rpmalloc-heaps'
                if heap_backend.exists():
                    all_ledger[module]['supplemental_backends'].append({
                        'module':'internal/bindings/rpmalloc-heaps',
                        'status':'partial_public_canonical_heap_adapter',
                        'artifacts':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()
                                     for p in sorted(heap_backend.iterdir()) if p.is_file()},
                        'scope':'Separate canonical reviewed raw/rewrite records, actual atomic members, private typed native ownership, real SDK counters, span graph and unmap-driven retirement; both statistics and all thread/global-cache selections',
                        'limitations':'Raw heaps require explicit private build selection; only live/quiescent observations; other native options remain incomplete; acquired heap handle reservation OOM returns null before SDK mutation, but graph/default-thread association OOM remains fail-fast',
                        'tests':['tests/native-bindings/rpmalloc-public-heaps.jai',
                                 'tests/native-bindings/rpmalloc-public-heap-lifecycle.jai',
                                 'tests/native-bindings/rpmalloc-heap-marshalling.jai',
                                 'tests/native-bindings/rpmalloc-public-raw-heaps.jai',
                                 'tests/native-bindings/rpmalloc-public-raw-heap-lifecycle.jai',
                                 'tests/native-bindings/rpmalloc-raw-heap-marshalling.jai']})
        if module == 'Objective_C':
            appkit = ROOT/'stdlib/Objective_C/AppKit/module.jai'
            if appkit.exists():
                methods = []
                for signature in sorted(authored_signatures(appkit.read_text().split('#scope_file',1)[0])):
                    # Read only the public section; helper names do not need
                    # to be guessed or enumerated to preserve privacy.
                    methods.append({'signature':signature,
                                    'status':'implemented',
                                    'verification':'Source presence only; native tests are recorded separately.'})
                all_ledger[module]['supplemental_facades'] = [{
                    'module':'Objective_C/AppKit', 'artifact':str(appkit.relative_to(ROOT)),
                    'artifact_sha256':hashlib.sha256(appkit.read_bytes()).hexdigest(),
                    'public_types':['NSBackingStoreType','NSResponder','NSWindow','NSWindowCollectionBehavior','NSWindowStyleMask'],
                    'private_pointer_only_types':['NSView','NSScreen'],
                    'members':methods,
                    'aliases':[{'name':'NSWindow.frame','signature':'(self: *NSView) -> NSRect','target':'private NSView.frame','status':'implemented'}],
                    'note':'Contextual AppKit contract from reviewed prose, not additional root exports. The reviewed setTitle id result is unspecified and must be ignored; native dispatch uses void.'}]
        if args.module and module not in args.module:
            continue
        path = ROOT / 'stdlib' / module / 'declarations.jai'
        preamble = ['#scope_file']
        preamble += ['#import "' + dep + '";' for dep in IMPORTS.get(module, [])]
        preamble += [PRIVATE.get(module, '')]
        if module == 'Vulkan':
            for name in sorted(set(re.findall(r'\b(binding_vk\w+)\(', code))):
                public_name = name.removeprefix('binding_')
                for d in spec.declarations:
                    if d.name == 'PFN_' + public_name:
                        signature = d.code.split('#type', 1)[1].replace('#c_call', '#foreign libvulkan "' + public_name + '"')
                        preamble.append(guard(name+' :: '+signature+';', d.condition))
        if module == 'MojoShader':
            context = ROOT/'stdlib/internal/bindings/vendor/mojoshader-context.jai'
            if context.exists():
                preamble.append(context.read_text())
        if module == 'telemetry3':
            preamble += [guard(block, 'ENABLED && OS == .WINDOWS') for block in spec.contextual_blocks]
        library_names = sorted(set(re.findall(r'#(?:foreign|elsewhere)\s+(\w+)', code + '\n' + PRIVATE.get(module, ''))))
        for name in library_names:
            if name == 'gl_lib':
                preamble.append('#if OS == .WINDOWS { gl_lib :: #library,system "opengl32"; }\nelse #if OS == .MACOS { gl_lib :: #library,system "OpenGL"; }\nelse { gl_lib :: #library,system "GL"; }')
            else:
                preamble.append(name + ' :: #library,system "' + LIBRARIES.get(name, name) + '";')
        preamble.append('#scope_export')
        code = ('// Generated from ' + str(page.relative_to(ROOT)) + '.\n// Regenerate with tools/native_bindings/generate.py; do not hand-edit.\n\n' + '\n'.join(preamble) + '\n\n' + code).rstrip()+'\n'
        code = format_source(code, args.formatter)
        if args.check:
            if not path.exists() or path.read_text() != code:
                stale.append(str(path.relative_to(ROOT)))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(code)
        entry = ROOT / 'stdlib' / module / 'module.jai'
        entry_code = '\n'.join(spec.parameter_blocks or [PARAMETERS.get(module, '')]) + '\n'
        if module in TARGETS:
            entry_code += '#assert ' + TARGETS[module] + ', "' + module + ' requires its documented native platform";\n'
        contextual = ROOT / 'stdlib' / module / 'context.jai'
        sources = MANUAL_FACADE_SOURCES.get(module, ('wrappers.jai',))
        if 'configuration.jai' in sources:
            entry_code += '#load "configuration.jai";\n'
        if module == 'rpmalloc':
            contextual_code = format_source(render_context(page.read_text()), args.formatter)
            if args.check:
                if not contextual.exists() or contextual.read_text() != contextual_code:
                    stale.append(str(contextual.relative_to(ROOT)))
            else:
                contextual.write_text(contextual_code)
        if contextual.exists():
            entry_code += '#load "context.jai";\n'
        entry_code += '#load "declarations.jai";\n'
        for name in sources:
            if name!='configuration.jai' and (ROOT/'stdlib'/module/name).exists():
                entry_code += '#load "'+name+'";\n'
        entry_code = format_source(entry_code, args.formatter)
        if args.check:
            if not entry.exists() or entry.read_text() != entry_code:
                stale.append(str(entry.relative_to(ROOT)))
        else:
            entry.write_text(entry_code)
    path = ROOT / 'docs/native/bindings-coverage.json'
    serialized = json.dumps({'format': 1, 'note': 'Statuses describe source dispositions, not native verification. Manual bodies may remain unverified on unavailable targets. Unresolved entries are not implemented.', 'modules': all_ledger}, indent=2) + '\n'
    if args.check:
        if not path.exists() or path.read_text() != serialized:
            stale.append(str(path.relative_to(ROOT)))
        if stale:
            raise SystemExit('Stale generated files:\n' + '\n'.join(stale))
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(serialized)
    for module, value in all_ledger.items():
        print(module + ': ' + ', '.join(f'{count} {status}' for status, count in value['counts'].items()))


if __name__ == '__main__':
    main()
