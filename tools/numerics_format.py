"""Scoped formatter-engine verification; full jaifmt CLI status is recorded separately."""
from pathlib import Path
import argparse
from numerics_codegen import ROOT,normalize

parser=argparse.ArgumentParser()
parser.add_argument('--check',action='store_true')
args=parser.parse_args()
roots=[ROOT/'stdlib'/name for name in ['Math','Sloppy_Math','Float16','Color_Interpolation','GetRect','GetRect_LeftHanded','internal/numerics']]
roots += [ROOT/'tests/numerics']
files=sorted({p for root in roots for p in root.rglob('*.jai')})
files += [ROOT/'tools/numerics-format-driver.jai', ROOT/'tools/numerics-formatter-files.jai', ROOT/'tools/numerics-formatter-basic.jai']
changed=[]
for path in files:
    source=path.read_text(); formatted=normalize(source)
    if formatted==source: continue
    changed.append(str(path.relative_to(ROOT)))
    if not args.check: path.write_text(formatted)
print(f'Repository Jai_Format engine: {len(files)} owned files, {len(changed)} '+('would change' if args.check else 'formatted'))
if args.check and changed:
    print('\n'.join(changed))
    raise SystemExit(1)
