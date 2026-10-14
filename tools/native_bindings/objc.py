"""Typed Objective-C forwarding derived from public member signatures.

Underscore-to-colon selector conversion is specified by Objective_C. Plain
member names use the selector's subsequent argument labels, as public Apple
Foundation/Metal declarations spell them. ABI calls always use typed pointers.
"""
import re


def message_body(owner, name, params, args, tail):
    tail = tail.rstrip(' {;').strip()
    ret = tail.removeprefix('->').strip() if tail.startswith('->') else 'void'
    ret = ret or 'void'
    if owner == 'NSString' and name == 'initWithString':
        return 'return initWithBytes(self, str.data, cast(NSUInteger) str.count, NSUTF8StringEncoding);'
    if owner == 'NSString' and name == 'getTempString':
        return 'object := cast(*NSString) objc_alloc(objc_getClass("NSString")); object = initWithBytes(object, str.data, cast(NSUInteger) str.count, NSUTF8StringEncoding); return cast(*NSString) autorelease(object);'
    instance = bool(args and args[0] == 'self')
    call_args = args[1:] if instance else args
    if '_' in name:
        selector = name.replace('_', ':')
        if call_args and not selector.endswith(':'):
            selector += ':'
    elif call_args:
        selector = ':'.join([name] + call_args[1:]) + ':'
    else:
        selector = name
    target = 'cast(*void) self' if instance else 'binding_objc_class("' + owner + '")'
    types = ['a' + str(i) + ': type_of(' + arg + ')' for i, arg in enumerate(call_args)]
    signature = 'Send :: #type (receiver: *void, op: *void' + (', ' + ', '.join(types) if types else '') + ') -> ' + ret + ' #c_call; '
    signature += 'send: Send; '
    if ret not in ('void', '()'):
        signature += '#if CPU == .X64 && size_of(' + ret + ') > 16 { send = cast(Send) binding_objc_send_stret; } else { send = cast(Send) binding_objc_send; } '
    else:
        signature += 'send = cast(Send) binding_objc_send; '
    call = 'send(' + ', '.join([target, 'binding_objc_selector("' + selector + '")'] + call_args) + ');'
    return signature + ('' if ret in ('void', '()') else 'return ') + call


def materialize_methods(owner, code, parse):
    lines = []
    for line in code.splitlines():
        if '#foreign' not in line and re.match(r'\s*\w+\s*::\s*(?:inline\s+)?\(', line):
            signature = line.strip().rstrip(' {;')
            parts = parse(signature)
            if parts:
                name, params, args, tail = parts
                line = '    ' + signature + ' { ' + message_body(owner, name, params, args, tail) + ' }'
        lines.append(line)
    return '\n'.join(lines)
