"""Native integration and explicit unsupported-contract checks; no reference tools."""
from pathlib import Path
import argparse
import json
import os
import subprocess
import sys

root = Path(__file__).resolve().parents[2]
compiler = root / 'artifacts/target/debug/jaic'
parser = argparse.ArgumentParser()
parser.add_argument('--stdlib', type=Path, default=root/'artifacts/stdlib-verification')
args = parser.parse_args()
library = args.stdlib.resolve()
out = root / 'artifacts/numerics-corrective'
out.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, JAIC_STDLIB=str(library))
records = []

def check(name, command, marker=None, diagnostic=False):
    result = subprocess.run([str(x) for x in command], cwd=root, env=env,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    text = result.stdout.decode(errors='replace')
    (out / (name + '.log')).write_text(text)
    ok = result.returncode != 0 if diagnostic else result.returncode == 0
    if marker is not None:
        ok = ok and marker in text
    records.append(dict(name=name, passed=ok, exit=result.returncode,
                        kind='expected diagnostic' if diagnostic else 'supported behavior'))
    (out / 'results.json').write_text(json.dumps(records, indent=2) + '\n')
    print(('PASS ' if ok else 'FAIL ') + name, flush=True)
    if not ok:
        print(text)
    return ok

for source, path, marker in [
        ('native_clipboard', library/'GetRect/tests/native_clipboard.jai', 'native clipboard ok'),
        ('native_cursors', library/'GetRect/tests/native_cursors.jai', 'native cursor lifecycle ok'),
        ('native_ui_cursors', root/'tests/numerics/native_ui_cursors.jai', 'native UI cursor lifecycle ok')]:
    for level in ['O0', 'O2']:
        executable = out / (source + '-' + level)
        if check(source + '-' + level + '-build',
                 [compiler, path, '-I', 'tests/numerics/dependencies',
                  '-' + level, '-exe', executable]):
            check(source + '-' + level, [executable], marker)

for source, marker in [('font_metrics_missing', 'Font ABI required for em height'),
                       ('texture_abi_missing', 'concrete Texture ABI and ownership contract unavailable')]:
    for level in ['O0', 'O2']:
        executable = out / (source + '-' + level)
        if check(source + '-' + level + '-build',
                 [compiler, 'tests/numerics/' + source + '.jai',
                  '-I', 'tests/numerics/dependencies', '-' + level, '-exe', executable]):
            check(source + '-' + level, [executable], marker, diagnostic=True)

# This is a reproduced compiler/spec integration blocker, not implemented
# augmented-keyframe functionality. Check that the exact failure is still present.
check('augmented-restriction-blocker',
      [compiler, 'tests/numerics/augmented_generic_reproducer.jai', '-check'],
      'Augmented does not satisfy the restriction `Key`', diagnostic=True)

passed = sum(record['passed'] for record in records)
print(f'{passed}/{len(records)} corrective checks passed; see {out}')
sys.exit(passed != len(records))
