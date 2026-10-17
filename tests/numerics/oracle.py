"""Independent Python math and IEEE struct oracles; emits durable Jai assertions."""
from pathlib import Path
import math, random, struct
from fractions import Fraction
root=Path(__file__).resolve().parents[2]
import sys
sys.path.insert(0,str(root/'tools'))
from repo_output import output_dir
rng=random.Random(0x986831)
lines=['#import "Basic";', 'M :: #import "Math";', 'H :: #import "Float16";', 'C :: #import "Color_Interpolation";',
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
    lines.append('    { q:M.Quaternion; M.set_from_axis_and_angle(*q,M.xyz('+','.join('cast(float32) '+repr(x) for x in axis)+'),cast(float32) '+repr(angle)+');')
    lines.append('      rotated:=M.rotate(M.xyz('+','.join('cast(float32) '+repr(x) for x in vector)+'),q);')
    for i,value in enumerate(expected): lines.append(f'      assert(M.abs(rotated.component[{i}]-cast(float32) {value!r})<0.00001);')
    lines.append('    }')
def f32(value):
    return struct.unpack('<f',struct.pack('<f',value))[0]

# Independent Python libm cube roots and double matrix arithmetic. Inputs and
# externally stored intermediate vectors are rounded to actual binary32 values.
colors=[(0,0,0),(1,1,1),(1,0,0),(0,1,0),(0,0,1)]
colors += [tuple(f32(rng.uniform(-1,2)) for _ in range(3)) for _ in range(64)]
for r,g,b in colors:
    l=math.cbrt(0.4122214708*r+0.5363325363*g+0.0514459929*b)
    m=math.cbrt(0.2119034982*r+0.6806995451*g+0.1073969566*b)
    s=math.cbrt(0.0883024619*r+0.2817188376*g+0.6299787005*b)
    lab=[f32(0.2104542553*l+0.793617785*m-0.0040720468*s),
         f32(1.9779984951*l-2.428592205*m+0.4505937099*s),
         f32(0.0259040371*l+0.7827717662*m-0.808675766*s)]
    lines.append('    { lab:=C.linear_srgb_to_oklab(M.xyz('+','.join('cast(float32) '+repr(v) for v in [r,g,b])+'));')
    for i,value in enumerate(lab):
        lines.append(f'      assert(M.abs(cast(float64) lab.component[{i}]-cast(float64) {value!r})<0.000001);')
    x,y,z=lab
    l=(x+0.3963377774*y+0.2158037573*z)**3
    m=(x-0.1055613458*y-0.0638541728*z)**3
    s=(x-0.0894841775*y-1.291485548*z)**3
    rgb=[4.0767416621*l-3.3077115913*m+0.2309699292*s,
         -1.2684380046*l+2.6097574011*m-0.3413193965*s,
         -0.0041960863*l-0.7034186147*m+1.707614701*s]
    lines.append('      rgb:=C.oklab_to_linear_srgb(lab);')
    for i,value in enumerate(rgb):
        value=f32(min(1,max(0,value)))
        lines.append(f'      assert(M.abs(cast(float64) rgb.component[{i}]-cast(float64) {value!r})<0.000001);')
    lines.append('    }')
lines += ['    print("independent oracle ok\\n");','}']
p=output_dir('numerics','numeric_oracle.jai'); p.parent.mkdir(parents=True,exist_ok=True); p.write_text('\n'.join(lines)+'\n')
count=sum(line.count('assert(') for line in lines)
print(f'{p}: {count} independent assertions')
