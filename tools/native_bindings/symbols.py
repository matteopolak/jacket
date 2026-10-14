"""Check native symbol availability without executing vendor functions."""
import ctypes
import ctypes.util
from pathlib import Path
import platform
import re

from config import LIBRARIES


def check_symbols(module, spec, enabled, native_root):
    os_name = 'macos' if platform.system() == 'Darwin' else 'windows' if platform.system() == 'Windows' else 'linux'
    cpu = 'arm64' if platform.machine().lower() in ('arm64', 'aarch64') else 'x64'
    libraries = {}
    result = []
    for d in spec.declarations:
        if not enabled(d.condition, os_name, cpu):
            continue
        match = re.search(r'#(?:foreign|elsewhere)\s+(\w+)(?:\s+"([^"]+)")?', d.code)
        if not match:
            continue
        alias, explicit = match.groups()
        name = LIBRARIES.get(alias, alias)
        symbol = explicit or d.name
        record = {'module': module, 'declaration': d.name, 'library': name, 'symbol': symbol}
        if name not in libraries:
            suffix = '.dylib' if os_name == 'macos' else '.dll' if os_name == 'windows' else '.so'
            path = native_root / ('lib' + name.removeprefix('lib') + suffix)
            try:
                libraries[name] = ctypes.CDLL(str(path) if path.exists() else ctypes.util.find_library(name.removeprefix('lib')) or name)
            except OSError as e:
                libraries[name] = str(e)
        library = libraries[name]
        if isinstance(library, str):
            record.update(status='unavailable', reason=library)
        else:
            try:
                getattr(library, symbol)
                record['status'] = 'passed'
            except AttributeError as e:
                record.update(status='failed', reason=str(e))
        result.append(record)
    return result
