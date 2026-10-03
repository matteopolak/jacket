#!/usr/bin/env python3
"""Author Objective-C selector dispatch from the authored Metal API contract.

Typed dispatch follows SDK objc/message.h. Reviewed LP64 plain aggregate
returns use an explicit X64 stack-result branch where required, and typed
ARM64 dispatch. No supplied procedure implementation is read or retained.
"""
from pathlib import Path
import json
import re


def main() -> None:
    path = Path('stdlib/Metal/Metal.jai')
    source = path.read_text()
    selectors = set(re.findall(r'^    (\w+) : \* void = --- ;$', source, re.M))
    scalar_aliases = {'NSUInteger', 'NSInteger', 'BOOL', 'uint32_t', 'uint64_t', 'int32_t',
                      'float', 'float32', 'float64', 'double', 'CFTimeInterval', 'id', 'CGColorSpaceRef',
                      'IOSurfaceRef', 'dispatch_queue_t', 's32', 's64', 'u32', 'u64'}
    scalar_aliases.update(re.findall(r'^(?:using )?(\w+) :: enum(?:_flags)?\b', source, re.M))
    reviewed_aggregate_bytes = {'CGSize': 16, 'MTLSizeAndAlign': 16,
                                'MTLSize': 24, 'MTLClearColor': 32}
    implemented, omitted = [], []
    pattern = re.compile(r'^(    )(\w+) :: (\( self : \* \w+(?: , .*?)? \)(?: -> .*?)?) #foreign Native_Adapters ;$', re.M)
    def replace(match: re.Match) -> str:
        indent, method, signature = match.groups()
        arguments, _, result = signature.partition(' -> ')
        result = result.strip() or '( )'
        if not (result.startswith('* ') or result in scalar_aliases or result in reviewed_aggregate_bytes or result in {'( )', 'void'}):
            omitted.append({'name': method, 'signature': signature, 'reason': 'aggregate or unclassified return ABI requires target-specific reviewed dispatch'})
            return match.group()
        pieces = arguments[2:-2].split(' , ')
        params = [piece.split(' : ', 1) for piece in pieces]
        if any(len(pair) != 2 for pair in params):
            return match.group()
        count = len(params) - 1
        selector_field = method if count == 0 else method + '_'
        if selector_field not in selectors or selector_field.count('_') != count:
            omitted.append({'name': method, 'signature': signature, 'reason': 'selector spelling cannot be derived unambiguously from API declaration'})
            return match.group()
        selector = selector_field.replace('_', ':')
        receiver_type = params[0][1]
        native_arguments = ['receiver: ' + receiver_type, 'selector: SEL'] + [name + ': ' + kind for name, kind in params[1:]]
        native_type = '(' + ', '.join(native_arguments) + ') -> ' + result + ' #c_call'
        call = 'invoke(self, sel_registerName(cast(*u8) "' + selector + '\\0")'
        if count:
            call += ', ' + ', '.join(name for name, _ in params[1:])
        call += ')'
        body = [indent + method + ' :: ' + signature + ' {',
                indent + '    invoke: ' + native_type + ';',
                indent + '    invoke = cast(type_of(invoke)) objc_msgSend;',
                indent + '    ' + ('return ' if result not in {'( )', 'void'} else '') + call + ';',
                indent + '}']
        if reviewed_aggregate_bytes.get(result, 0) > 16:
            stret_type = '(output: *' + result + ', ' + ', '.join(native_arguments) + ') -> void #c_call'
            stret_call = call.replace('invoke(self,', 'send_stret(*value, self,', 1)
            body = [indent + method + ' :: ' + signature + ' {',
                indent + '    #if CPU == .X64 {',
                indent + '        value: ' + result + ';',
                indent + '        send_stret: ' + stret_type + ';',
                indent + '        send_stret = cast(type_of(send_stret)) objc_msgSend_stret;',
                indent + '        ' + stret_call + ';',
                indent + '        return value;',
                indent + '    } else {'] + [indent + '    ' + line[len(indent):] for line in body[1:-1]] + [
                indent + '    }', indent + '}']
        implemented.append({'name': method, 'signature': signature, 'selector': selector,
                            'kind': 'independently-authored-typed-objc-message-dispatch', 'native_runtime_verified': False})
        return '\n'.join(body)
    authored = pattern.sub(replace, source)
    class_methods = {
        'texture2DDescriptorWithPixelFormat_width_height_mipmapped': 'MTLTextureDescriptor',
        'textureCubeDescriptorWithPixelFormat_size_mipmapped': 'MTLTextureDescriptor',
        'textureBufferDescriptorWithPixelFormat_width_resourceOptions_usage': 'MTLTextureDescriptor',
        'argumentDescriptor': 'MTLArgumentDescriptor',
        'renderPassDescriptor': 'MTLRenderPassDescriptor',
        'stageInputOutputDescriptor': 'MTLStageInputOutputDescriptor',
        'vertexDescriptor': 'MTLVertexDescriptor',
        'sharedCaptureManager': 'MTLCaptureManager',
    }
    class_pattern = re.compile(r'^    (\w+) :: (\( (?:.*? )?\) -> \* \w+) #foreign Native_Adapters ;$', re.M)
    def class_replace(match: re.Match) -> str:
        method, signature = match.groups()
        if method not in class_methods:
            return match[0]
        arguments, _, result = signature.partition(' -> ')
        params = [] if arguments == '( )' else [piece.split(' : ', 1) for piece in arguments[2:-2].split(' , ')]
        selector = method.replace('_', ':') + (':' if params else '')
        native_params = ['receiver: Class', 'selector: SEL'] + [name + ': ' + kind for name, kind in params]
        call_args = ['objc_getClass(cast(*u8) "' + class_methods[method] + '\\0")',
                     'sel_registerName(cast(*u8) "' + selector + '\\0")'] + [name for name, _ in params]
        return '\n'.join(['    ' + method + ' :: ' + signature + ' {',
            '        invoke: (' + ', '.join(native_params) + ') -> ' + result + ' #c_call;',
            '        invoke = cast(type_of(invoke)) objc_msgSend;',
            '        return invoke(' + ', '.join(call_args) + ');', '    }'])
    authored = class_pattern.sub(class_replace, authored)
    initialization = ['init_metal :: () {']
    for field in sorted(selectors):
        initialization.append('    __selectors.' + field + ' = cast(*void) sel_registerName(cast(*u8) "' + field.replace('_', ':') + '\\0");')
    initialization.append('}')
    authored = re.sub(r'^init_metal :: .*? #foreign Native_Adapters ;$', lambda _: '\n'.join(initialization), authored, flags=re.M)
    path.write_text(authored)
    # Reconstruct every authored message body on repeated runs. Counts describe
    # the resulting source rather than only this invocation's substitutions.
    implemented = []
    body_pattern = re.compile(r'^    (\w+) :: (.*?) \{\n(.*?)^    \}', re.M | re.S)
    for match in body_pattern.finditer(authored):
        if 'invoke = cast(type_of(invoke)) objc_msgSend;' not in match[3]:
            continue
        selector = re.search(r'sel_registerName\(cast\(\*u8\) "(.*?)\\0"\)', match[3])
        if selector:
            implemented.append({'name': match[1], 'signature': match[2],
                'selector': selector[1], 'kind': 'independently-authored-typed-objc-message-dispatch',
                'native_runtime_verified': False})
    Path('stdlib/.coverage/metal-source-dispatch.json').write_text(json.dumps({'format': 1,
        'typed_message_dispatch_procedures': implemented, 'count': len(implemented),
        'selector_initialization_count': len(selectors), 'unimplemented_dispatch': omitted,
        'sdk_contract': '/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/objc/message.h',
        'native_execution_performed': False, 'aggregate_return_dispatch_verified': False,
        'aggregate_return_dispatch_authored': reviewed_aggregate_bytes,
        'aggregate_dispatch_policy': 'Reviewed LP64 plain records: X64 >16 byte results use output-first objc_msgSend_stret; ARM64 uses typed objc_msgSend.'}, indent=2) + '\n')
    print(json.dumps({'message_dispatch_helpers': len(implemented), 'selector_initializers': len(selectors), 'unimplemented': len(omitted)}))


if __name__ == '__main__':
    main()
