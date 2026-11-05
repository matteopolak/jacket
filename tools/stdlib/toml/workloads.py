"""Independently authored, deterministic TOML workloads; no runtime dependency or private input.

    python3 tools/stdlib/toml/workloads.py --output /tmp/toml-workloads
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

from toolkit.repo import ROOT


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repository-config', type=Path, default=ROOT / 'jacket.toml')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    workloads = {
        'config': "title = 'tool'\n[format]\nindent_width = 2\nmax_width = 100\nbrace_style = 'preserve'\nignore = ['vendor/**', '**/generated_*.jai']\n[doc.format]\nmax_width = 80\n",
    }
    workloads['repository-config'] = args.repository_config.read_bytes().decode('utf-8')
    for size in (1024, 8192):
        text = workloads['config']
        index = 0
        while len(text.encode()) < size:
            text += f"[[assets]]\npath = 'assets/object-{index}.bin'\nlabel = 'item-{index}'\nflags = [true, false]\n"
            index += 1
        workloads[f'config-{size}'] = text
    for size in (4096, 100 << 10, 1 << 20, 10 << 20):
        lines = []
        length = 0
        index = 0
        while length < size:
            line = f'key_{index:07d} = {index}\n'
            lines.append(line)
            length += len(line)
            index += 1
        workloads[f'wide-{size}'] = ''.join(lines)
    lines = []
    length = 0
    index = 0
    while length < 100 << 10:
        line = f'[[records]]\nid = {index}\nat = 2024-02-29T12:34:56.123+01:30\nlabel = "record-{index}"\nvalues = [1, 2.5, true, "λ"]\n'
        lines.append(line)
        length += len(line.encode())
        index += 1
    workloads['records-102400'] = ''.join(lines)
    manifest = {}
    for name, text in workloads.items():
        data = text.encode()
        (args.output / f'{name}.toml').write_bytes(data)
        manifest[name + '.toml'] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
    invalid = args.output / 'invalid'
    invalid.mkdir(exist_ok=True)
    for name in ('wide-4096', 'wide-102400', 'wide-1048576'):
        text = workloads[name]
        for case, bad in [('early', 'bad = @\n' + text), ('late', text + 'bad = @\n'), ('duplicate', text + 'key_0000000 = 1\n')]:
            data = bad.encode()
            filename = name + '-' + case + '.toml'
            (invalid / filename).write_bytes(data)
            manifest['invalid/' + filename] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'expected': 'reject'}
    (args.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps(manifest, indent=2))


if __name__ == '__main__':
    main()
