#!/usr/bin/env python3
"""Author SDK message calls from independently normalized Objective-C contracts.

No reference procedure bodies are consumed. Selector tables and public method
signatures provide names; SDK message.h defines typed dispatch. Known plain
aggregate records use the documented x64 structure-return entry when needed.
"""
from __future__ import annotations

import re
from pathlib import Path
from author_objective_c_helpers import helper_body

SDK = Path('/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk')
SCALAR = {'void', '()', 'bool', 'int', 'float', 'float32', 'float64', 'double',
          's8', 's16', 's32', 's64', 'u8', 'u16', 'u32', 'u64', 'BOOL', 'Boolean',
          'NSUInteger', 'NSInteger', 'CGFloat', 'NSTimeInterval', 'NSStringEncoding',
          'Selector', 'SEL', 'id', 'Class', 'NSWindowLevel', 'dispatch_queue_t'}
AGGREGATES = {'NSPoint': 16, 'NSSize': 16, 'CGSize': 16, 'NSRange': 16,
              'NSRect': 32, 'NSOperatingSystemVersion': 24, 'GCAcceleration': 24,
              'GCRotationRate': 24, 'GCEulerAngles': 24, 'GCQuaternion': 32}
INITIALIZERS = {'init_objective_c', 'init_objective_c_foundation', 'init_app_kit',
                'init_objective_c_gamecontroller', 'init_lightweight_rendering_view'}
METHOD = re.compile(r'^(\s*)(\w+) :: (\([^\n]*?) #foreign Native_Adapters "([^"]+)";', re.M)
SELECTOR_CORRECTIONS = {'addSubView_': 'addSubview_'}
SDK_VOID_RETURNS = {('NSApplication', 'setDelegate'), ('NSApplication', 'finishLaunching'),
                    ('NSApplication', 'setMainMenu'), ('NSWindow', 'update'),
                    ('NSWindow', 'display'), ('NSWindow', 'setTitle'),
                    ('NSOpenGLContext', 'update')}


def selector_spelling(field: str):
    corrected = SELECTOR_CORRECTIONS.get(field, field)
    prefix = len(corrected) - len(corrected.lstrip('_'))
    return corrected[:prefix] + corrected[prefix:].replace('_', ':')


def split_parameters(text: str) -> list[str]:
    result, start, depth = [], 0, 0
    for at, char in enumerate(text):
        if char in '([':
            depth += 1
        elif char in ')]':
            depth -= 1
        elif char == ',' and depth == 0:
            result.append(text[start:at].strip())
            start = at + 1
    if text[start:].strip():
        result.append(text[start:].strip())
    return result


def signature_parts(signature: str):
    depth = 0
    closing = None
    for at, char in enumerate(signature):
        if char == '(':
            depth += 1
        elif char == ')':
            depth -= 1
            if depth == 0:
                closing = at
                break
    if closing is None:
        return None
    suffix = signature[closing + 1:].strip()
    result = suffix[2:].strip() if suffix.startswith('->') else 'void'
    result = re.sub(r'\s+#\w+.*$', '', result).strip()
    params = []
    for parameter in split_parameters(signature[1:closing]):
        if parameter.startswith('$'):
            return None
        if ':=' in parameter:
            name, default = [value.strip() for value in parameter.split(':=', 1)]
            if default not in ('NO', 'YES'):
                return None
            kind = 'BOOL'
        elif ':' in parameter:
            name, kind = [value.strip() for value in parameter.split(':', 1)]
            kind = kind.split(' = ', 1)[0].strip()
        else:
            return None
        if kind in {'string', 'Type', 'Any'} or '$' in kind:
            return None
        params.append((name, kind))
    return params, result


def owner_at(source: str, offset: int):
    # Emitted public class records have top-level openings and closings.
    owner = None
    for line in source[:offset].splitlines():
        match = re.match(r'^(\w+) :: struct(?:\s*\([^)]*\))?\s*(?:#[^{}]*)?\{', line)
        if match:
            owner = match[1]
        elif line.startswith('}'):
            owner = None
    return owner


def scalar_aliases(source: str) -> set[str]:
    aliases = set(SCALAR)
    aliases.update(re.findall(r'^\s*(?:using\s+)?(\w+) :: enum(?:_flags)?\b', source, re.M))
    pending = re.findall(r'^\s*(\w+) :: ([^;{}\n]+);', source, re.M)
    for _ in range(5):
        for name, value in pending:
            if value.startswith('*') or value.strip() in aliases:
                aliases.add(name)
    return aliases


def selector_for(method: str, count: int, fields: set[str]):
    candidates = []
    for field in fields:
        if field.lstrip('_').count('_') != count:
            continue
        if (count == 0 and field == method) or (count > 0 and (field == method + '_' or field.startswith(method + '_'))):
            candidates.append(field)
            continue
        pieces = field.rstrip('_').split('_')
        combined = pieces[0] + ''.join(piece[0].upper() + piece[1:] for piece in pieces[1:] if piece)
        if combined == method:
            candidates.append(field)
    if len(candidates) == 1:
        return selector_spelling(candidates[0])
    return None


def emit_call(indent: str, signature: str, receiver: str, selector: str, args, result: str, compatibility_null=False):
    native_params = ['sdk_receiver: *void', 'sdk_selector: SEL'] + [name + ': ' + kind for name, kind in args]
    values = [receiver, 'sel_registerName(cast(*u8) "' + selector + '\\0")'] + [name for name, _ in args]
    body = [indent + signature + ' {']
    stret = result in AGGREGATES and AGGREGATES[result] > 16
    if stret:
        body += [indent + '    #if CPU == .X64 {', indent + '        result: ' + result + ';',
                 indent + '        invoke: (output: *' + result + ', ' + ', '.join(native_params) + ') -> void #c_call;',
                 indent + '        invoke = cast(type_of(invoke)) objc_msgSend_stret;',
                 indent + '        invoke(*result, ' + ', '.join(values) + ');', indent + '        return result;',
                 indent + '    } else {']
        inner = indent + '        '
    else:
        inner = indent + '    '
    body += [inner + 'invoke: (' + ', '.join(native_params) + ') -> ' + result + ' #c_call;',
             inner + 'invoke = cast(type_of(invoke)) objc_msgSend;',
             inner + ('return ' if result not in {'void', '()'} else '') + 'invoke(' + ', '.join(values) + ');']
    if compatibility_null:
        body.append(inner + 'return null;')
    if stret:
        body.append(indent + '    }')
    body.append(indent + '}')
    return '\n'.join(body)


def author(relative: str, source: str):
    if not relative.startswith('Objective_C/') or '/bindings/' in relative:
        return source, [], []
    fields = set(re.findall(r'^\s*(\w+)\s*:\s*Selector\s*;', source, re.M))
    aliases = scalar_aliases(source)
    implemented, omitted = [], []
    def replace(match):
        whitespace, name, signature, symbol = match.groups()
        indent = whitespace.split('\n')[-1]
        if name in INITIALIZERS:
            lines = [indent + name + ' :: ' + signature + ' {']
            for field in sorted(fields):
                selector = 'copy' if field == 'objc_copy' else selector_spelling(field)
                lines.append(indent + '    _sel.' + field + ' = cast(Selector) sel_registerName(cast(*u8) "' + selector + '\\0");')
            lines.append(indent + '}')
            implemented.append({'adapter_symbol': symbol, 'name': name, 'kind': 'independent-selector-table-initialization', 'selector_count': len(fields), 'native_runtime_verified': False})
            return whitespace[:-len(indent)] + '\n'.join(lines) if indent else whitespace + '\n'.join(lines)
        owner = owner_at(source, match.start())
        helper = helper_body(relative, owner, name, signature)
        if helper is not None:
            rendered = '\n'.join([indent + name + ' :: ' + signature + ' {'] +
                                 [indent + '    ' + line for line in helper.strip().splitlines()] + [indent + '}'])
            implemented.append({'adapter_symbol': symbol, 'name': name, 'owner': owner,
                                'kind': 'independent-sdk-or-reflection-helper', 'native_runtime_verified': False})
            return whitespace[:-len(indent)] + rendered if indent else whitespace + rendered
        parts = signature_parts(signature)
        reason = None
        if not owner:
            reason = 'non-method helper requires a separately authored implementation'
        elif owner.startswith('Lightweight'):
            reason = 'custom class SDK compatibility is implemented separately'
        elif not parts:
            reason = 'Jai string, untyped default, or generic ABI needs a bridge'
        else:
            params, result = parts
            correction = None
            if owner in {'GCExtendedGamepadSnapshot', 'GCMicroGamepadSnapshot'} and name.startswith('initWith') and result == owner:
                signature = signature.replace('-> ' + owner, '-> *' + owner)
                result = '*' + owner
                correction = 'Configured SDK instancetype returns an object pointer, not an object record'
            if not (result.startswith('*') or result in aliases or result in AGGREGATES):
                reason = 'unclassified return ABI'
            else:
                instance = bool(params and params[0][0] == 'self')
                args = params[1:] if instance else params
                selector = selector_for(name, len(args), fields)
                if not selector:
                    reason = 'selector cannot be inferred unambiguously from the normalized contract'
                else:
                    receiver = 'cast(*void) self' if instance else 'cast(*void) objc_getClass(cast(*u8) "' + owner + '\\0")'
                    void_return = (owner, name) in SDK_VOID_RETURNS and result == 'id'
                    if void_return:
                        correction = 'Configured SDK method returns void; compatibility Jai id result is null'
                    rendered = emit_call(indent, name + ' :: ' + signature, receiver, selector, args, 'void' if void_return else result, void_return)
                    if owner in {'NSArray', 'NSMutableArray'} and any(kind == 'Object_Type' for _, kind in args):
                        rendered = rendered.replace(' {\n', ' {\n' + indent + '    #assert type_info(Object_Type).type == .POINTER;\n', 1)
                    record = {'adapter_symbol': symbol, 'name': name, 'owner': owner, 'selector': selector, 'kind': 'independent-typed-sdk-message-dispatch', 'return_abi': 'x64-stret-arm64-direct' if result in AGGREGATES and AGGREGATES[result] > 16 else 'direct-typed', 'native_runtime_verified': False}
                    if correction:
                        record['sdk_signature_correction'] = correction
                    implemented.append(record)
                    return whitespace[:-len(indent)] + rendered if indent else whitespace + rendered
        omitted.append({'adapter_symbol': symbol, 'name': name, 'signature': signature, 'reason': reason})
        return match.group()
    result = METHOD.sub(replace, source)
    if '#foreign Native_Adapters' not in result:
        result = re.sub(r'\n#scope_file\nNative_Adapters :: #system_library "jai-stdlib-native-adapters";\n?', '\n', result)
    return result, implemented, omitted
