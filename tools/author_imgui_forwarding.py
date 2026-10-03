#!/usr/bin/env python3
"""Author API-preserving value/reference and byte-string ImGui overloads.

Only authored binding declarations are read. Native C++ symbol/layout version
requirements remain external and unverified; these bodies do not establish
compatibility with a different Dear ImGui version.
"""
from collections import defaultdict
import json
from pathlib import Path
import re
from rewrite_native_api_contracts import tokenize, balanced_end, normalize_contract


def signature_parts(signature: str):
    tokens = tokenize(signature)
    stop = balanced_end(tokens, 0, '(', ')')
    params, segment, depth = [], [], 0
    for token in tokens[1:stop-1]:
        value = token.value
        if value in {'(', '[', '{', '.{'}:
            depth += 1
        elif value in {')', ']', '}'}:
            depth -= 1
        if value == ',' and depth == 0:
            params.append(segment)
            segment = []
        else:
            segment.append(value)
    if segment:
        params.append(segment)
    result = []
    for parameter in params:
        if ':' in parameter:
            colon = parameter.index(':')
            name = parameter[0]
            kind = parameter[colon+1:]
            if '=' in kind:
                kind = kind[:kind.index('=')]
        elif ':=' in parameter:
            name = parameter[0]
            default = parameter[parameter.index(':=')+1:]
            if default in [['false'], ['true']]:
                kind = ['bool']
            elif len(default) == 1 and default[0].startswith('"'):
                kind = ['string']
            else:
                return None
        else:
            return None
        result.append((name, ' '.join(kind)))
    returns = ' '.join(t.value for t in tokens[stop:] if t.value not in {'#cpp_method', '#c_call'})
    if returns in {'-> void', '-> ( )'}:
        returns = ''
    return result, returns


def main() -> None:
    reports = []
    for path in (Path('stdlib/ImGui/unix.jai'), Path('stdlib/ImGui/windows.jai')):
        source = path.read_text()
        prototype = re.compile(r'(?P<indent>^[ \t]*)(?P<notes>(?:@\w+ )*)(?P<name>\w+) :: (?P<signature>\(.*?\)(?: ->.*?)?) #foreign (?P<library>imgui|Native_Adapters)(?: "[^"\n]*")? ;', re.M | re.S)
        natives = defaultdict(list)
        for match in prototype.finditer(source):
            if match.group('library') == 'imgui':
                parsed = signature_parts(match.group('signature'))
                if parsed:
                    natives[match.group('name')].append(parsed)
        authored = []
        def replace(match: re.Match) -> str:
            if match.group('library') != 'Native_Adapters':
                return match.group()
            parsed = signature_parts(match.group('signature'))
            if not parsed:
                return match.group()
            params, returns = parsed
            candidates = [(match.group('name'), item) for item in natives[match.group('name')]]
            candidates += [(match.group('name') + '_CFormat', item) for item in natives[match.group('name') + '_CFormat']]
            for native_name, (native_params, native_returns) in candidates:
                if len(params) != len(native_params) or returns != native_returns:
                    continue
                arguments, transformations = [], []
                for (name, kind), (_, native_kind) in zip(params, native_params):
                    if kind == native_kind:
                        arguments.append(('..' if kind.startswith('..') else '') + name)
                    elif native_kind == '* ' + kind:
                        arguments.append('*' + name)
                        transformations.append('value-address')
                    elif kind == 'string' and native_kind == '* u8':
                        arguments.append('ImGui_String_Memory.temp_c_string(' + name + ')')
                        transformations.append('temporary-byte-string-terminator')
                    else:
                        break
                else:
                    if not transformations:
                        continue
                    break
            else:
                return match.group()
            indent = match.group('indent')
            resultful = returns not in {'', '-> void', '-> ( )'}
            call = native_name + '(' + ', '.join(arguments) + ')'
            body = indent + match.group('notes') + match.group('name') + ' :: ' + match.group('signature') + ' {\n' + indent + '    ' + ('return ' if resultful else '') + call + ';\n' + indent + '}'
            authored.append({'name': match.group('name'), 'signature': match.group('signature'),
                             'transformations': transformations, 'native_runtime_verified': False})
            return body
        output = prototype.sub(replace, source)
        if authored:
            output += '\n#scope_file\nImGui_String_Memory :: #import "Basic";\n'
        path.write_text(output)
        _, body_inventory = normalize_contract(output)
        retained = [{'name': item['name'], 'signature': item['signature'],
                     'kind': 'independently-authored-native-overload-forwarding',
                     'native_runtime_verified': False}
                    for item in body_inventory['unimplemented_adapters']]
        reports.append({'path': str(path), 'authored': retained, 'count': len(retained), 'new_this_pass': len(authored)})
    Path('stdlib/.coverage/imgui-forwarding-wrappers.json').write_text(json.dumps({'format': 1, 'rows': reports}, indent=2) + '\n')
    print(json.dumps({'forwarding_helpers': sum(row['count'] for row in reports)}))


if __name__ == '__main__':
    main()
