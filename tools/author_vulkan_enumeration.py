#!/usr/bin/env python3
"""Author counted-array Vulkan overloads from the existing binding API.

The native enumeration signature and registry sType metadata determine the
independent two-call query/allocation/fill flow. Error statuses and partial
results are returned; this is not a native execution or GPU test.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET
from rewrite_native_api_contracts import normalize_contract


def parameters(signature: str) -> list[tuple[str, str]]:
    inside = signature[2:signature.find(' )')].strip()
    if not inside:
        return []
    result = []
    for item in inside.split(' , '):
        name, separator, kind = item.partition(' : ')
        if not separator:
            return []
        result.append((name.strip(), kind.strip()))
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--registry', required=True, type=Path, help='Reviewed canonical Khronos vk.xml input')
    arguments = parser.parse_args()
    registry = ET.parse(arguments.registry).getroot()
    structure_types = {}
    for item in registry.findall('types/type'):
        if item.get('category') == 'struct':
            for member in item.findall('member'):
                if member.findtext('name') == 'sType' and member.get('values'):
                    structure_types[item.get('name')] = member.get('values')
    results = []
    for path in [Path('stdlib/Vulkan/generated_linux.jai'), Path('stdlib/Vulkan/generated_windows.jai')]:
        source = path.read_text()
        native = {}
        for name, signature in re.findall(r'^(\w+) :: (\( .*? \)(?: -> .*?)?) #foreign libvulkan ;$', source, re.M):
            native.setdefault(name, []).append(signature)
        authored = []
        pattern = re.compile(r'^(\w+) :: (\( (?:.*? )?\) -> \[ \] (\w+)(?: , (VkResult))?) #foreign Native_Adapters ;$', re.M)
        def replace(match: re.Match) -> str:
            name, signature, element, result_kind = match.groups()
            inputs = parameters(signature)
            for actual in native.get(name, []):
                args = parameters(actual)
                if len(args) != len(inputs) + 2:
                    continue
                if [kind for _, kind in args[:-2]] != [kind for _, kind in inputs]:
                    continue
                if args[-2][1] != '* u32' or args[-1][1] != '* ' + element:
                    continue
                if bool(result_kind) != actual.endswith(' -> VkResult'):
                    continue
                break
            else:
                return match.group()
            call_inputs = ', '.join(parameter for parameter, _ in inputs)
            if call_inputs:
                call_inputs += ', '
            query = name + '(' + call_inputs + '*count, null)'
            fill = name + '(' + call_inputs + '*count, data.data)'
            lines = [name + ' :: ' + signature + ' {', '    data: [] ' + element + ';', '    count: u32 = 0;']
            if result_kind:
                lines += ['    status := ' + query + ';', '    if cast(s32) status < 0 || count == 0 return data, status;']
            else:
                lines += ['    ' + query + ';', '    if count == 0 return data;']
            lines += ['    data.data = cast(*' + element + ') Vulkan_Enumeration_Memory.alloc(cast(s64) count * size_of(' + element + '));']
            if result_kind:
                lines += ['    if data.data == null return data, VkResult.ERROR_OUT_OF_HOST_MEMORY;']
            else:
                lines += ['    if data.data == null return data;']
            lines += ['    data.count = cast(s64) count;']
            if element in structure_types:
                lines += ['    Vulkan_Enumeration_Memory.memset(data.data, 0, data.count * size_of(' + element + '));',
                          '    for index: 0..data.count - 1 data[index].sType = VkStructureType.' + structure_types[element] + ';']
            if result_kind:
                lines += ['    status = ' + fill + ';',
                          '    if cast(s32) status < 0 {',
                          '        Vulkan_Enumeration_Memory.free(data.data);',
                          '        data.data = null;', '        data.count = 0;',
                          '        return data, status;', '    }',
                          '    data.count = cast(s64) count;', '    return data, status;']
            else:
                lines += ['    ' + fill + ';', '    data.count = cast(s64) count;', '    return data;']
            lines += ['}']
            authored.append({'name': name, 'signature': signature, 'element_type': element,
                             'sType_initialized': structure_types.get(element),
                             'ownership': 'caller frees returned data.data with the same context allocator',
                             'native_execution_verified': False})
            return '\n'.join(lines)
        output = pattern.sub(replace, source)
        if authored and 'Vulkan_Enumeration_Memory :: #import "Basic";' not in output:
            output += '\n#scope_file\nVulkan_Enumeration_Memory :: #import "Basic";\n'
        path.write_text(output)
        _, inventory = normalize_contract(output)
        retained = []
        for helper in inventory['unimplemented_adapters']:
            result = re.search(r' -> \[ \] (\w+)(?: , VkResult)?$', helper['signature'])
            if result:
                retained.append({'name': helper['name'], 'signature': helper['signature'],
                    'element_type': result[1], 'sType_initialized': structure_types.get(result[1]),
                    'ownership': 'caller frees returned data.data with the same context allocator',
                    'native_execution_verified': False})
        results.append({'path': str(path), 'authored': retained, 'count': len(retained), 'new_this_pass': len(authored)})
    Path('stdlib/.coverage/vulkan-enumeration-wrappers.json').write_text(json.dumps({'format': 1,
        'registry_sha256': hashlib.sha256(arguments.registry.read_bytes()).hexdigest(), 'rows': results}, indent=2) + '\n')
    print(json.dumps({'enumeration_helpers': sum(row['count'] for row in results)}))


if __name__ == '__main__':
    main()
