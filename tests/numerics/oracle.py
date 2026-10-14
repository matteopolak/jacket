"""Independent Python math and IEEE struct oracles; emits durable Jai assertions."""
from pathlib import Path
import math, random, struct
from fractions import Fraction
root=Path(__file__).resolve().parents[2]
rng=random.Random(0x986831)
lines=['#import "Basic";', 'M :: #import "Math";', 'H :: #import "Float16";',
       'B :: union { f:float32; u:u32; }', 'main :: () {']
def check(expr, expected, relative=3e-14, absolute=1e-300):
    lines.append(f'    assert(M.abs(({expr})-cast(float64) {expected!r}) <= cast(float64) {max(absolute,abs(expected)*relative)!r}, "oracle {len(lines)}");')
for _ in range(120):
    x=rng.uniform(-700,700)
    check(f'M.exp(cast(float64) {x!r})',math.exp(x),1.5e-13)
    x=math.ldexp(rng.uniform(0.5,1),rng.randrange(-1021,1023))
    check(f'M.log(cast(float64) {x!r})',math.log(x),3e-14,1e-14)
    x=rng.uniform(-1000,1000)
    check(f'M.atan(cast(float64) {x!r})',math.atan(x))
    x=rng.uniform(-1,1)
    check(f'M.asin(cast(float64) {x!r})',math.asin(x))
    check(f'M.acos(cast(float64) {x!r})',math.acos(x))
    x,y=rng.uniform(-10,10),rng.uniform(-10,10)
    check(f'M.atan2(cast(float64) {y!r}, cast(float64) {x!r})',math.atan2(y,x))
    x,y=rng.uniform(0.1,10),rng.uniform(-8,8)
    check(f'M.pow(cast(float64) {x!r}, cast(float64) {y!r})',math.pow(x,y),1.5e-13)
for _ in range(1500):
    bits=rng.randrange(1<<32)
    f=struct.unpack('<f',struct.pack('<I',bits))[0]
    if math.isnan(f): continue
    try: half=struct.unpack('<H',struct.pack('<e',f))[0]
    except OverflowError: half=(bits>>16&0x8000)|0x7c00
    lines += [f'    {{ b:B; b.u=0x{bits:08x}; assert(cast(u16) H.float32_to_float16(b.f)==0x{half:04x}, "half oracle {bits:08x}"); }}']
for e in range(-25,16):
    for sign in [-1,1]:
        f=sign*math.ldexp(1,e)
        check(f'cast(float64) H.float16_to_float32(H.float32_to_float16(cast(float32) {f!r}))',struct.unpack('<e',struct.pack('<e',f))[0],0,0)
for dimension in [3,4]:
    for _ in range(24):
        # Strictly diagonally dominant integer matrices, inverted by exact rational
        # arithmetic independently of the worker's float32 algorithms.
        matrix=[[rng.randrange(-5,6) for _ in range(dimension)] for _ in range(dimension)]
        for i in range(dimension): matrix[i][i]=sum(abs(v) for j,v in enumerate(matrix[i]) if i!=j)+rng.randrange(1,8)
        a=[[Fraction(x) for x in row]+[Fraction(i==j) for j in range(dimension)] for i,row in enumerate(matrix)]
        for col in range(dimension):
            pivot=next(i for i in range(col,dimension) if a[i][col])
            a[col],a[pivot]=a[pivot],a[col]
            d=a[col][col]; a[col]=[x/d for x in a[col]]
            for i in range(dimension):
                if i!=col:
                    d=a[i][col]; a[i]=[x-d*y for x,y in zip(a[i],a[col])]
        values=','.join(str(x) for row in matrix for x in row)
        lines += [f'    {{ matrix:=M.Matrix{dimension}.{{{values}}}; ok,result:=M.inverse(matrix); assert(ok);']
        for i in range(dimension):
            for j in range(dimension):
                value=float(a[i][dimension+j])
                lines.append(f'      assert(M.abs(result.coef[{i}][{j}]-cast(float32) {value!r})<0.00001);')
        lines.append('    }')
for _ in range(32):
    axis=[rng.uniform(-1,1) for _ in range(3)]; norm=math.sqrt(sum(x*x for x in axis)); axis=[x/norm for x in axis]
    vector=[rng.uniform(-10,10) for _ in range(3)]; angle=rng.uniform(-math.pi,math.pi)
    dot=sum(a*b for a,b in zip(axis,vector)); cross=[axis[1]*vector[2]-axis[2]*vector[1],axis[2]*vector[0]-axis[0]*vector[2],axis[0]*vector[1]-axis[1]*vector[0]]
    expected=[v*math.cos(angle)+c*math.sin(angle)+a*dot*(1-math.cos(angle)) for v,c,a in zip(vector,cross,axis)]
    lines.append('    { q:M.Quaternion; M.set_from_axis_and_angle(*q,M.xyz('+','.join(repr(x) for x in axis)+'),'+repr(angle)+');')
    lines.append('      rotated:=M.rotate(M.xyz('+','.join(repr(x) for x in vector)+'),q);')
    for i,value in enumerate(expected): lines.append(f'      assert(M.abs(rotated.component[{i}]-cast(float32) {value!r})<0.00001);')
    lines.append('    }')
lines += ['    print("independent oracle ok\\n");','}']
p=root/'artifacts/numeric_oracle.jai'; p.parent.mkdir(exist_ok=True); p.write_text('\n'.join(lines)+'\n')
count=sum(line.count('assert(') for line in lines)
print(f'{p}: {count} independent assertions')
