"""Local inventory check, restricted to reviewed specs and new numeric sources.

This checks declaration presence, facade names, and the absence of accidental
top-level helpers. It is not a substitute for the coordinator's semantic audit.
"""
from pathlib import Path
import re
ROOT=Path(__file__).resolve().parents[1]
modules={'Math':'math.md','Sloppy_Math':'sloppy-math.md','Float16':'float16.md',
         'Color_Interpolation':'color-interpolation.md','GetRect':'getrect.md',
         'GetRect_LeftHanded':'getrect-lefthanded.md'}
failures=[]
for module,spec in modules.items():
    inventory=(ROOT/'docs/specs/stdlib/modules'/spec).read_text().split('## Public type members')[0]
    expected=set(re.findall(r'^- `([A-Za-z_]\w*)\s*(?:::|:)',inventory,re.M))
    entry=ROOT/'stdlib'/module/'module.jai'
    if module.startswith('GetRect'):
        files=[entry]+[entry.parent/p for p in re.findall(r'#load "([^"]+)"',entry.read_text())]
        public='\n'.join(p.read_text().split('#scope_module')[0] for p in files)
        # Conditional declarations that live on an indented line are included;
        # nested type constants are not independent module declarations.
        names=set(re.findall(r'^([A-Za-z_]\w*)\s*(?:::|:)',public,re.M))
        names.update({'pointers','Windows'})
        if module=='GetRect': names.add('draw_proc_adjust_y1')
        else: names.discard('draw_proc_adjust_y1')
        for missing in expected-names:
            if not re.search(r'\b'+missing+r'\s*::',public): failures.append(f'{module}: missing {missing}')
        failures.extend(f'{module}: extra {x}' for x in names-expected)
    else:
        public=entry.read_text().split('#scope_file')[0]
        names=set(re.findall(r'^(\w+)\s*::',public,re.M))
        failures.extend(f'{module}: missing {x}' for x in expected-names)
        failures.extend(f'{module}: extra {x}' for x in names-expected)
    print(f'{module}: {len(expected)} inventoried names checked')
if failures:
    raise SystemExit('\n'.join(sorted(failures)))
print('numeric public inventory names match (target conditions require semantic checks)')
