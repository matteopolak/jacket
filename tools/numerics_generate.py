"""Generate repetitive independently designed numeric component operations."""
from pathlib import Path
import re
from numerics_codegen import write_generated

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / 'stdlib/internal/numerics'
BASE.mkdir(parents=True, exist_ok=True)

def write(name, text):
    write_generated(BASE / name,text)

types = []
for n in [2, 3, 4]:
    fields = 'xyzw'[:n]
    overlays = [f'#overlay(x) component: [{n}]float;']
    for i in range(n-1):
        overlays.append(f'#overlay({fields[i]}) {fields[i:i+2]}: Vector2;') if n > 2 else None
    if n == 4:
        overlays += ['#overlay(x) xyz: Vector3;', '#overlay(y) yzw: Vector3;']
    types.append(f'Vector{n} :: struct {{ '+', '.join(fields)+': float; '+' '.join(overlays)+' }')
types += ['Quaternion :: struct { x,y,z: float; w := 1.0; #overlay(x) xyz: Vector3; #overlay(x) component: [4]float; }',
          'Complex :: #type,isa Vector2;',
          'Plane3 :: struct { a,b,c,d: float; #overlay(a) #as vector4: Vector4; #overlay(a) normal: Vector3; }',
          'AnyVector2 :: struct { x,y: float; }']
for name,r,c in [('Matrix2',2,2),('Matrix3',3,3),('Matrix4x3',3,4),('Matrix4',4,4)]:
    fields = ', '.join(f'_{i+1}{j+1}' for i in range(r) for j in range(c))
    extra = '#overlay(_11) mat4x3: Matrix4x3;' if name == 'Matrix4' else ''
    types.append(f'{name} :: struct {{ {fields}: float; #overlay(_11) v: [{r}]Vector{c}; #overlay(_11) coef: [{r}][{c}]float; #overlay(_11) floats: [{r*c}]float; {extra} Row_Type :: Vector{c}; Column_Type :: Vector{r}; IsMatrixFromMathModule :: true; }}')
    diagonal = ', '.join(f'_{i+1}{i+1}=1' for i in range(min(r,c)))
    types.append(f'{name}_Identity :: {name}.{{{diagonal}}};')
    if name != 'Matrix4':
        types.append(f'Any{name} :: struct {{ {fields}: float; }}')
types += ['AnyMatrix :: struct { IsMatrixFromMathModule :: true; }']
write('math_types.jai', '\n'.join(types)+'\n')

ops = []
for n in [2,3,4]:
    T=f'Vector{n}'; fs='xyzw'[:n]
    def aggregate(expr): return '.{'+', '.join(expr(f) for f in fs)+'}'
    ops += [f'dot_product :: (a: {T}, b: {T}) -> float {{ return '+ ' + '.join(f'a.{f}*b.{f}' for f in fs)+'; }',
            f'length_squared :: inline (a: {T}) -> float {{ return dot_product(a,a); }}',
            f'length :: (a: {T}) -> float {{ return cast(float) length_wide(a); }}',
            f'abs :: inline (using v: {T}) -> {T} {{ return '+aggregate(lambda f:f'abs(v.{f})')+'; }',
            f'saturate :: (a: {T}) -> {T} {{ return '+aggregate(lambda f:f'saturate(a.{f})')+'; }',
            f'lerp :: (a: {T}, b: {T}, t: float) -> {T} {{ return '+aggregate(lambda f:f'lerp(a.{f},b.{f},t)')+'; }']
    for op in ['min','max']:
        ops.append(f'{op} :: inline (v1: {T}, v2: {T}) -> {T} {{ return '+aggregate(lambda f:f'{op}(v1.{f},v2.{f})')+'; }')
    for op in ['+','-']:
        ops.append(f'operator {op} :: (a: {T}, b: {T}) -> {T} {{ return '+aggregate(lambda f:f'a.{f}{op}b.{f}')+'; }')
    for op in ['*','/']:
        ops.append(f'operator {op} :: (a: {T}, b: float) -> {T} {{ return '+aggregate(lambda f:f'a.{f}{op}b')+'; }')
    ops.append(f'operator - :: (a: {T}) -> {T} {{ return '+aggregate(lambda f:f'-a.{f}')+'; }')
    ops.append(f'operator * :: (b: float, a: {T}) -> {T} {{ return a*b; }}')
    ops.append(f'make_vector{n} :: (x: float) -> {T} {{ return '+'.{'+', '.join('x' for _ in fs)+'}; }')
    fn=fs
    ops.append(f'{fn} :: ('+', '.join(f'{f}: float' for f in fs)+f') -> {T} {{ return .{{'+', '.join(fs)+'}; }')
    ops.append(f'{fn} :: (x: float) -> {T} {{ return make_vector{n}(x); }}')
    if n < 4:
        ops += [f'multiply :: (v: {T}, f: float) -> {T} {{ return v*f; }}',
                f'distance_squared :: inline (a: {T}, b: {T}) -> float {{ return length_squared(a-b); }}',
                f'unit_vector :: (v: {T}) -> {T} {{ return unit_vector_wide(v); }}']
    ops.append(f'distance :: (a: {T}, b: {T}) -> float {{ return length(a-b); }}')
ops += ['dot :: dot_product;', 'rgb :: xyz;', 'rgba :: xyzw;',
        'MyVectors :: .[Vector2,Vector3,Vector4,Quaternion];',
        'normalize :: (v: *$T/MyVectors, epsilon := 0.0, fallback := T.{}) -> float { wide := length_wide(v.*); n := cast(float) wide; if n <= epsilon v.* = fallback; else { for *x: v.component x.* = cast(float) (cast(float64) x.* / wide); } return n; }',
        'normalize :: (v: $T/MyVectors, epsilon := 0.0, fallback := T.{}) -> T { r := v; normalize(*r, epsilon, fallback); return r; }',
        'make_vector3 :: (c: [3]float) -> Vector3 { return .{c[0],c[1],c[2]}; }',
        'make_vector3 :: (using v: Vector2, z: float) -> Vector3 { return .{x,y,z}; }',
        'xyz :: (c: [3]float) -> Vector3 { return make_vector3(c); }',
        'xyz :: (using v: Vector2, z: float) -> Vector3 { return make_vector3(v,z); }',
        'make_vector4 :: (using v: Vector2, z: float, w: float) -> Vector4 { return .{x,y,z,w}; }',
        'make_vector4 :: (using v: Vector3, w: float) -> Vector4 { return .{x,y,z,w}; }',
        'xyzw :: (using v: Vector2, z: float, w: float) -> Vector4 { return make_vector4(v,z,w); }',
        'xyzw :: (using v: Vector3, w: float) -> Vector4 { return make_vector4(v,w); }',
        'cross :: cross_product;',
        'cross_product :: (a: Vector3, b: Vector3) -> Vector3 { return .{a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x}; }',
        'rotate :: (v: Vector2, theta: float) -> Vector2 { c := cos(theta); s := sin(theta); return .{c*v.x-s*v.y,s*v.x+c*v.y}; }',
        'rotate :: (v: *Vector2, theta: float) { v.* = rotate(v.*,theta); }']
wide_norm_helpers = '''
#scope_module
// Preserve range and significand precision until the final public binary32
// result. Normalization must not divide by that rounded/overflowed result.
length_wide :: (v: $T/MyVectors) -> float64 {
    largest: float64 = 0;
    for x: v.component {
        if isnan(x) return FLOAT64_NAN;
        largest = max(largest, abs(cast(float64) x));
    }
    if largest == 0 || isinf(largest) return largest;
    sum: float64 = 0;
    for x: v.component {
        scaled := cast(float64) x / largest;
        sum += scaled * scaled;
    }
    return largest * sqrt(sum);
}

unit_vector_wide :: (v: $T/MyVectors) -> T {
    wide := length_wide(v);
    if wide == 0 return v;
    r := v;
    for *x: r.component x.* = cast(float) (cast(float64) x.* / wide);
    return r;
}
'''
write('math_vectors.jai','\n'.join(ops)+wide_norm_helpers+'\n#scope_file\n#import "Basic";\n')

def facade(module,spec,impl):
    text=(ROOT/'docs/specs/stdlib/modules'/spec).read_text().split('## Public type members')[0].split('## Behavioral')[0]
    expected=set(re.findall(r'^- `([A-Za-z_][A-Za-z_0-9]*)\s*(?:::|:)',text,re.M))
    if module=='Math':
        names=[]
        for file in ['math_types.jai','math_scalar.jai','math_vectors.jai','math_quaternion.jai','math_matrix.jai','math_geometry.jai']:
            names.extend(n for n in re.findall(r'^(\w+)\s*(?:::|:)',(BASE/file).read_text(),re.M) if n in expected and n not in names)
    elif module=='Sloppy_Math': names=['min_of_type','max_of_type','is_approximately_zero','values_are_close']
    elif module=='Float16': names=['Float16','Vector2_Float16','Vector3_Float16','Vector4_Float16','Matrix3_Float16','half_to_float_fast4','float_to_half_fast3','float16_to_float32','float32_to_float16','matrix3_float16','matrix3_float32','vector2_float16','vector2_float32','vector3_float16','vector3_float32','vector4_float16']
    else: names=['GAMMA','Interpolation','Color_Keyframe','rgb_to_hsv','hsv_to_rgb','linear_srgb_to_oklab','oklab_to_linear_srgb','pow','easeInOutQuad','hermite','make_colors','interpolate_one_sample','generate_color_samples']
    assert set(names)==expected, (module,expected-set(names))
    p=ROOT/'stdlib'/module; p.mkdir(parents=True,exist_ok=True)
    header='#module_parameters(ASSERT_ON_OVERFLOW := false);\n' if module=='Float16' else ''
    args='(ASSERT_ON_OVERFLOW=ASSERT_ON_OVERFLOW)' if module=='Float16' else ''
    write_generated(p/'module.jai',header+'\n'.join(f'{n} :: Impl.{n};' for n in names)+'\n#scope_file\nImpl :: #import "internal/numerics/'+impl+'"'+args+';\n')

if __name__ == '__main__':
    facade('Math','math.md','math')
    facade('Sloppy_Math','sloppy-math.md','sloppy')
    facade('Float16','float16.md','half')
    facade('Color_Interpolation','color-interpolation.md','color')
