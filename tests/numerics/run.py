"""Run clean-room numerics tests against an explicit locally source-built compiler."""
from pathlib import Path
import argparse,json,os,subprocess,sys,time
root=Path(__file__).resolve().parents[2]
parser=argparse.ArgumentParser()
parser.add_argument('--compiler',type=Path,required=True)
parser.add_argument('--stdlib',type=Path,required=True)
parser.add_argument('--quick',action='store_true')
parser.add_argument('--canonical-dependencies',action='store_true',help='Use actual Input/Window_Type/Hash_Table instead of the explicitly marked UI fixtures')
args=parser.parse_args()
compiler=args.compiler.resolve(); env=dict(os.environ,JAIC_STDLIB=str(args.stdlib.resolve()))
out=root/'artifacts/numerics-verification'; out.mkdir(parents=True,exist_ok=True)
records=[]
imports=[] if args.canonical_dependencies else ['-I','tests/numerics/dependencies']
def run(label,cmd,sentinel=None,expect_fail=False):
    started=time.monotonic()
    result=subprocess.run([str(x) for x in cmd],cwd=root,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    log=result.stdout
    (out/(label+'.log')).write_text(log)
    passed=(result.returncode!=0 and 'overflow assertion was missed' not in log and 'half float overflow' in log) if expect_fail else result.returncode==0 and (sentinel is None or sentinel in log) and 'assertion failed' not in log
    records.append(dict(label=label,passed=passed,exit=result.returncode,seconds=round(time.monotonic()-started,3)))
    print(('PASS ' if passed else 'FAIL ')+label,flush=True)
    if not passed: print(log)
    (out/'results.json').write_text(json.dumps(records,indent=2)+'\n')
    return passed
run('inventory',[sys.executable,'tools/numerics_surface.py'])
run('generate-oracle',[sys.executable,'tests/numerics/oracle.py'])
tests={
    'core':('tests/numerics/core.jai','numerics core ok'),
    'extended':('tests/numerics/extended.jai','extended numerics ok'),
    'edges':('tests/numerics/edges.jai','numeric edges ok'),
    'algebra':('tests/numerics/algebra.jai','algebra overloads ok'),
    'color-policies':('tests/numerics/color_policies.jai','color policies ok'),
    'ui':('tests/numerics/ui.jai','ui contracts ok'),
    'ui-smoke':('tests/numerics/ui_smoke.jai','ui smoke ok'),
    'ui-interaction':('tests/numerics/ui_interaction.jai','ui interaction ok'),
    'render-callbacks':('tests/numerics/render_callbacks.jai','render callbacks ok'),
    'oracle':('artifacts/numeric_oracle.jai','independent oracle ok'),
    'public-contract':('tests/stdlib-contract/math-float16/main.jai','ok'),
}
for name,(source,sentinel) in tests.items():
    common=[compiler,source]+imports
    run(name+'-interp',common+['-interp'],sentinel)
    if args.quick: continue
    for level in ['O0','O2']:
        exe=out/(name+'-'+level)
        if run(name+'-'+level+'-build',common+['-'+level,'-exe',exe]): run(name+'-'+level,[exe],sentinel)
    wasm=out/(name+'.wasm')
    if run(name+'-wasm-build',common+['-os','wasm','-O2','-exe',wasm]): run(name+'-wasm',['node','--no-warnings','tools/wasi_run.mjs',wasm],sentinel)
for target in ['linux','windows','macos','wasm']:
    run(target+'-unused-check',[compiler,'tests/numerics/ui_smoke.jai']+imports+['-check','-no_dce','-os',target])
for target in ['linux','windows','macos']:
    run(target+'-x64-unused-check',[compiler,'tests/numerics/ui_smoke.jai']+imports+['-check','-no_dce','-os',target,'-cpu','x64'])
run('overflow-assert',[compiler,'tests/numerics/half_overflow.jai','-interp'],expect_fail=True)
if not args.quick:
    for level in ['O0','O2']:
        exe=out/('overflow-'+level)
        if run('overflow-'+level+'-build',[compiler,'tests/numerics/half_overflow.jai','-'+level,'-exe',exe]):
            run('overflow-'+level,[exe],expect_fail=True)
    wasm=out/'overflow.wasm'
    if run('overflow-wasm-build',[compiler,'tests/numerics/half_overflow.jai','-os','wasm','-O2','-exe',wasm]):
        run('overflow-wasm',['node','--no-warnings','tools/wasi_run.mjs',wasm],expect_fail=True)
failures=[r for r in records if not r['passed']]
print(f'{len(records)-len(failures)}/{len(records)} checks passed; evidence: {out}')
raise SystemExit(bool(failures))
