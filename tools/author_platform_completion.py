"""Independently authored SDK wrappers over already-normalized contracts.

This pass never reads reference bodies. The maintained Jai packet files contain
full replacement declarations and private helpers, and are inserted only after
contract extraction. Requiring all expected unresolved signatures prevents
silently retaining a marker library if the contract changes.
"""
from pathlib import Path
import re

PACKETS = {
    'Objective_C/module.jai': 'objective-c-reflection.jai',
    'Android/EGL/module.jai': 'android-egl.jai',
    'Android/File.jai': 'android-assets.jai',
    'Android/module.jai': 'android-lifecycle.jai',
    'Windows_Resources.jai': 'windows-toolchain.jai',
}

def author(relative: str, source: str):
    filename = PACKETS.get(relative)
    if not filename:
        return source, []
    packet = (Path(__file__).parent / 'platform-completion' / filename).read_text()
    public, helpers = packet.split('// PRIVATE HELPERS\n', 1)
    # Bodies live in a separately authored source packet; declaration-only input
    # selects them by exact procedure name plus overload ordinal.
    declarations = re.findall(r'(?m)^([A-Za-z_]\w*)\s*::', public)
    replacements = []
    cursor = 0
    for name in declarations:
        start = re.search(r'(?m)^' + re.escape(name) + r'\s*::', public[cursor:])
        start_at = cursor + start.start()
        next_decl = re.search(r'(?m)^[A-Za-z_]\w*\s*::', public[cursor + start.end():])
        end_at = cursor + start.end() + next_decl.start() if next_decl else len(public)
        body = public[start_at:end_at].strip()
        pattern = re.compile(r'(?m)^([ \t]*)' + re.escape(name) + r'\s*::[^\n]*#foreign Native_Adapters "([^"]+)";')
        match = pattern.search(source)
        if not match:
            raise ValueError(f'{relative}: expected normalized missing wrapper {name}')
        indent = match[1]
        source = source[:match.start()] + '\n'.join(indent + line for line in body.splitlines()) + source[match.end():]
        replacements.append({'name': name, 'adapter_symbol': match[2],
                             'implementation': 'tools/platform-completion/' + filename,
                             'status': 'independent-jai-implementation',
                             'native_behavior_verified': False})
        cursor = end_at
    if '#foreign Native_Adapters' in source:
        raise ValueError(f'{relative}: unexpected remaining unresolved wrapper')
    source = re.sub(r'\n#scope_file\nNative_Adapters :: #system_library "jai-stdlib-native-adapters";\s*', '\n', source)
    if relative == 'Android/module.jai':
        # #program_export belongs immediately before the independently supplied
        # definition, and only the opt-in module-parameter branch contains it.
        source = source.replace('    android_main ::', '    #program_export\n    android_main ::', 1)
    source += '\n#scope_file\n' + helpers
    return source, replacements
