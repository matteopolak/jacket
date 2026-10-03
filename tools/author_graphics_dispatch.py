#!/usr/bin/env python3
"""Author COM forwarding bodies from independently normalized ABI declarations.

Only the authored output tree is read. The dispatch rule is the documented COM
interface pointer/vtable model; no implementation from reference is consumed.
"""
from pathlib import Path
import json
import re


PROCEDURE = re.compile(r'^(\w+) :: (\( .*? \)(?: -> .*?)?) #foreign Native_Adapters ;$', re.M)


def parameters(signature: str) -> list[tuple[str, str]]:
    end = signature.find(' )')
    inside = signature[2:end].strip()
    if not inside:
        return []
    result = []
    for parameter in inside.split(' , '):
        name, separator, kind = parameter.partition(' : ')
        if not separator:
            return []
        result.append((name.strip(), kind.split(' = ')[0].strip()))
    return result


def main() -> None:
    rows = []
    for family in ('d3d11', 'd3d12', 'dxgi', 'd3d_compiler', 'dxc_compiler'):
        for path in sorted((Path('stdlib') / family).glob('*.jai')):
            original = path.read_text()
            authored = []
            def replace(match: re.Match) -> str:
                name, signature = match.groups()
                params = parameters(signature)
                if not params:
                    return match.group()
                native_overloads = re.finditer(r'^' + re.escape(name) + r' :: (\( .*? \)(?: -> .*?)?) #foreign (?!Native_Adapters\b)\w+(?: .*?)? ;$', original, re.M)
                for native_overload in native_overloads:
                    native_params = parameters(native_overload[1])
                    if len(native_params) != len(params):
                        continue
                    args = []
                    for (parameter, kind), (_, native_kind) in zip(params, native_params):
                        if kind == native_kind:
                            args.append(parameter)
                        elif native_kind == '* ' + kind:
                            args.append('*' + parameter)
                        else:
                            break
                    else:
                        returns = ' -> ' in signature and not signature.endswith((' -> void', ' -> ( )'))
                        authored.append({'name': name, 'signature': signature,
                            'kind': 'independently-authored-sdk-overload-forwarding', 'native_runtime_verified': False})
                        return name + ' :: ' + signature + ' {\n    ' + ('return ' if returns else '') + name + '(' + ', '.join(args) + ');\n}'
                if not params[0][1].startswith('* '):
                    return match.group()
                receiver, receiver_type = params[0]
                interface = receiver_type[2:].strip()
                field = re.search(r'\b(vtable|\w+_vtable) : \* ' + re.escape(interface) + r'_VTable ;', original)
                if field is None:
                    return match.group()
                if name == 'vtable':
                    expression = receiver + '.' + field.group(1)
                elif name.startswith(interface + '_'):
                    method = name[len(interface) + 1:]
                    # Confirm the method is an ABI field, not a helper with a
                    # coincidentally similar name.
                    native = re.search(r'\b' + re.escape(method) + r' : (\( this : \* ' + re.escape(interface) + r'(?: , .*?)? \)(?: -> .*?)?) #cpp_method ;', original)
                    if native is None:
                        return match.group()
                    native_params = parameters(native.group(1))
                    if len(native_params) != len(params):
                        return match.group()
                    arguments = []
                    for (parameter, kind), (_, native_kind) in zip(params, native_params):
                        if kind == native_kind:
                            arguments.append(parameter)
                        elif native_kind == '* ' + kind:
                            arguments.append('*' + parameter)
                        else:
                            return match.group()
                    expression = receiver + '.' + field.group(1) + '.' + method + '(' + ', '.join(arguments) + ')'
                else:
                    return match.group()
                returns = ' -> ' in signature and not signature.endswith((' -> void', ' -> ( )'))
                body = ('return ' if returns or name == 'vtable' else '') + expression + ';'
                authored.append({'name': name, 'signature': signature,
                                 'kind': 'independently-authored-com-vtable-dispatch',
                                 'native_runtime_verified': False})
                return name + ' :: ' + signature + ' {\n    ' + body + '\n}'
            output = PROCEDURE.sub(replace, original)
            path.write_text(output)
            # Repeated authoring passes must keep earlier generated bodies in
            # the evidence rather than reporting only the newest replacements.
            retained = re.findall(r'^(\w+) :: (\( .*? \)(?: -> .*?)?) \{\n    (?:return )?\w+\.(?:vtable|\w+_vtable)(?:\.\w+\([^\n]*\))?;\n\}', output, re.M)
            all_authored = [{'name': name, 'signature': signature,
                             'kind': 'independently-authored-com-vtable-dispatch',
                             'native_runtime_verified': False} for name, signature in retained]
            sdk_forwarding = re.findall(r'^(\w+) :: (\( .*? \)(?: -> .*?)?) \{\n    (?:return )?\1\([^\n]*\);\n\}', output, re.M)
            all_authored.extend({'name': name, 'signature': signature,
                'kind': 'independently-authored-sdk-overload-forwarding', 'native_runtime_verified': False}
                for name, signature in sdk_forwarding)
            rows.append({'path': str(path), 'procedures': all_authored, 'count': len(all_authored), 'new_this_pass': len(authored)})
    Path('stdlib/.coverage/graphics-com-dispatch.json').write_text(json.dumps({'format': 1, 'rows': rows}, indent=2) + '\n')
    print(json.dumps({'authored_com_dispatch_procedures': sum(row['count'] for row in rows)}))


if __name__ == '__main__':
    main()
