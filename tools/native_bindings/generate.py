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
from config import IMPORTS, LIBRARIES, PARAMETERS, PRIVATE, TARGETS
from wrappers import simple_body, forwarding_body
from objc import materialize_methods
from bitfields import body as bitfield_body
from spec import Declaration
from conditions import possible

ROOT = Path(__file__).resolve().parents[2]


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


def guard(code, condition):
    return '#if ' + condition + ' {\n' + code + '\n}\n' if condition else code + '\n'


def normalized_signature(code):
    tokens = re.findall(r'"(?:\\.|[^"\\])*"|[^\s"]+',code.strip().rstrip(';'))
    return ''.join(tokens)


def authored_signatures(text):
    signatures = set()
    for match in re.finditer(r'^[ \t]*\w+\s*::\s*(?:inline\s+)?\(',text,re.M):
        fragment = text[match.start():]
        parts = procedure_parts(fragment)
        if not parts or '{' not in parts[3]:
            continue
        # Parameter defaults may contain aggregate literals. Only a brace in
        # the tail after the balanced parameter list starts the authored body.
        end = len(fragment)-len(parts[3])+parts[3].index('{')
        signatures.add(normalized_signature(fragment[:end]))
    return signatures


def condition_for(module, declaration, spec=None):
    condition = declaration.condition
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
        condition = 'USE_GLUE_NATIVE_PORT' if '#foreign' in declaration.code else '!USE_GLUE_NATIVE_PORT'
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
    ledger, output, emitted = [], [], set()
    constants_path = ROOT/'stdlib/internal/bindings/vendor/constants.json'
    native_constants = json.loads(constants_path.read_text()) if constants_path.exists() else {}
    manual_path = ROOT / 'stdlib' / module / 'wrappers.jai'
    manual_signatures = authored_signatures(manual_path.read_text()) if manual_path.exists() else set()
    for d in spec.declarations:
        record = {'name': d.name, 'signature': d.code, 'condition': d.condition, 'spec_line': d.line}
        condition = condition_for(module, d, spec)
        if condition != d.condition:
            record['effective_condition'] = condition
        if normalized_signature(d.code) in manual_signatures:
            record.update(status='implemented_manual', artifact=str(manual_path.relative_to(ROOT)))
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
                if module in ('ImGui', 'nvtt'):
                    if module == 'ImGui' and d.name == 'TextFilter':
                        metadata = json.loads((ROOT/'stdlib/internal/bindings/vendor/imgui-symbols.json').read_text())
                        symbol = metadata['symbols']['windows-x64' if s.condition=='OS == .WINDOWS' else 'linux-x64']
                        body = re.sub(r'(\bsplit\s*::[^\n]+?#foreign imgui)(?=\s*[;}])',r'\1 "'+symbol+'"',body)
                        record['native_symbol_metadata'] = metadata
                    # Public C++ shapes can put several prototypes on a line.
                    # Split only at statement boundaries before a procedure;
                    # aggregate member and overload order remain unchanged.
                    body = re.sub(r';[ \t]*(?=\w+\s*::\s*(?:inline\s+)?\()', ';\n', body)
                    if module == 'nvtt':
                        body, native_members = supplement_nvtt_members(d.name, body, s.condition)
                        record['native_header_members'] = native_members
                    member_declarations = [Declaration(line.strip().rstrip(' {;'),s.condition)
                                           for line in body.splitlines() if procedure_parts(line.strip().rstrip(' {;'))]
                    lines = []
                    for line in body.splitlines():
                        signature = line.strip().rstrip(' {;')
                        parts = procedure_parts(signature)
                        if parts:
                            member = {'signature': signature, 'condition': s.condition, 'status': 'declared'}
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
        entry_code += '#load "declarations.jai";\n'
        wrappers = ROOT / 'stdlib' / module / 'wrappers.jai'
        if wrappers.exists():
            entry_code += '#load "wrappers.jai";\n'
        entry_code = format_source(entry_code, args.formatter)
        if args.check:
            if not entry.exists() or entry.read_text() != entry_code:
                stale.append(str(entry.relative_to(ROOT)))
        else:
            entry.write_text(entry_code)
    path = ROOT / 'docs/native/bindings-coverage.json'
    serialized = json.dumps({'format': 1, 'note': 'Declared is not ABI or runtime verified. Unresolved entries are not implemented.', 'modules': all_ledger}, indent=2) + '\n'
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
