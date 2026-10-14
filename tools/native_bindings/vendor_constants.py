#!/usr/bin/env python3
"""Recover omitted native macro operands from hash-pinned public C headers."""
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT/'stdlib/internal/bindings/vendor/constants.json'


def main():
    manifest = json.loads((ROOT/'tools/native_bindings/dependencies.json').read_text())
    constants = {}
    def header(package, filename):
        path = ROOT/'.work/vendor'/package/filename
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        if digest != manifest[package]['files'][filename]:
            raise SystemExit('Header hash mismatch: '+str(path))
        return data.decode(), {'package':package, 'header':filename, 'header_sha256':digest}
    text, provenance = header('x11','include/X11/Xlib.h')
    for name in ('XNClientWindow','XNFocusWindow','XNInputStyle'):
        value = re.search(r'^#define\s+'+name+r'\s+("[^"\n]+")',text,re.M)[1]
        constants['X11/'+name] = dict(provenance, code=name+' :: '+value+'.data')
    text, provenance = header('windows-sdk','generation/WinSDK/RecompiledIdlHeaders/shared/hidclass.h')
    name = 'GUID_DEVINTERFACE_HID'
    values = re.search(r'DEFINE_GUID\(\s*'+name+r',([^;]+)\);',text)[1].split(',')
    values = [hex(int(v.strip().rstrip('Ll'),0)) for v in values]
    if len(values) != 11:
        raise SystemExit('Unexpected native GUID initializer')
    initializer = 'GUID.{'+', '.join(values[:3])+', .['+', '.join(values[3:])+']}'
    constants['Windows/'+name] = dict(provenance, code=name+' :: '+initializer)
    OUTPUT.write_text(json.dumps(constants,indent=2)+'\n')
    print('Generated',len(constants),'native constant initializers')


if __name__ == '__main__':
    main()
