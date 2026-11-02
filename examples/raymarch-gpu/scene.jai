// Shared WGSL for the browser and native raymarch examples.
GPU_SCENE :: #string WGSL
struct Uniforms { time: f32, aspect: f32, palette: f32, _pad: f32 }
@group(0) @binding(0) var<uniform> u: Uniforms;

struct Out { @builtin(position) position: vec4f, @location(0) uv: vec2f }

@vertex fn vs(@builtin(vertex_index) i: u32) -> Out {
    let p = array(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    var out: Out;
    out.position = vec4f(p[i], 0.0, 1.0);
    out.uv = p[i];
    return out;
}

fn rot(a: f32) -> mat2x2f { let c = cos(a); let s = sin(a); return mat2x2f(c, s, -s, c); }

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

// x: distance, y: material (0 floor, 1 blob, 2 torus)
fn map(p: vec3f) -> vec2f {
    let t = u.time;
    var q = p - vec3f(0.0, 1.0, 0.0);
    let wobble = 0.08 * sin(4.0 * q.x + t * 2.0) * sin(4.0 * q.y + t * 1.7) * sin(4.0 * q.z + t * 1.3);
    let blob = smin(length(q) - 0.7, length(q - vec3f(0.6 * sin(t), 0.4 * cos(t * 1.3), 0.0)) - 0.35, 0.3) + wobble;

    var r = q - vec3f(1.6 * cos(t * 0.7), 0.2 * sin(t * 1.1), 1.6 * sin(t * 0.7));
    let rxz = rot(t * 1.5) * r.xz;
    r = vec3f(rxz.x, r.y, rxz.y);
    let ryz = rot(t) * r.yz;
    r = vec3f(r.x, ryz.x, ryz.y);
    let torus = length(vec2f(length(r.xz) - 0.45, r.y)) - 0.15;

    let floor_d = p.y;
    var d = vec2f(floor_d, 0.0);
    if blob < d.x { d = vec2f(blob, 1.0); }
    if torus < d.x { d = vec2f(torus, 2.0); }
    return d;
}

fn normal(p: vec3f) -> vec3f {
    let e = vec2f(0.001, 0.0);
    return normalize(vec3f(
        map(p + e.xyy).x - map(p - e.xyy).x,
        map(p + e.yxy).x - map(p - e.yxy).x,
        map(p + e.yyx).x - map(p - e.yyx).x));
}

fn shadow(ro: vec3f, rd: vec3f) -> f32 {
    var res = 1.0;
    var t = 0.02;
    for (var i = 0; i < 48; i++) {
        let h = map(ro + rd * t).x;
        res = min(res, 10.0 * h / t);
        t += clamp(h, 0.02, 0.3);
        if res < 0.001 || t > 8.0 { break; }
    }
    return clamp(res, 0.0, 1.0);
}

fn palette(m: f32, p: vec3f) -> vec3f {
    let k = u.palette;
    if m < 0.5 {
        let c = (floor(p.x) + floor(p.z)) % 2.0;
        return mix(vec3f(0.18), vec3f(0.42), abs(c));
    }
    let a = vec3f(0.5);
    let b = vec3f(0.5);
    let base = select(vec3f(0.0, 0.33, 0.67), vec3f(0.6, 0.2, 0.1), m > 1.5);
    return a + b * cos(6.2831 * (base + k * 0.37 + 0.15 * p.y));
}

@fragment fn fs(in: Out) -> @location(0) vec4f {
    let uv = vec2f(in.uv.x * u.aspect, in.uv.y);
    let a = u.time * 0.2;
    let ro = vec3f(4.5 * sin(a), 2.2, 4.5 * cos(a));
    let ta = vec3f(0.0, 0.9, 0.0);
    let f = normalize(ta - ro);
    let r = normalize(cross(vec3f(0.0, 1.0, 0.0), f));
    let up = cross(f, r);
    let rd = normalize(uv.x * r + uv.y * up + 1.6 * f);

    var t = 0.0;
    var m = -1.0;
    for (var i = 0; i < 128; i++) {
        let h = map(ro + rd * t);
        if h.x < 0.0005 { m = h.y; break; }
        t += h.x;
        if t > 30.0 { break; }
    }

    let sky = mix(vec3f(0.55, 0.7, 0.95), vec3f(0.1, 0.15, 0.35), clamp(rd.y * 1.5, 0.0, 1.0));
    var col = sky;
    if m >= 0.0 {
        let p = ro + rd * t;
        let n = normal(p);
        let light = normalize(vec3f(0.6, 0.8, 0.4));
        let diff = clamp(dot(n, light), 0.0, 1.0) * shadow(p + n * 0.01, light);
        let spec = pow(clamp(dot(reflect(rd, n), light), 0.0, 1.0), 32.0);
        let amb = 0.25 + 0.15 * n.y;
        col = palette(m, p) * (amb + 0.9 * diff) + 0.4 * spec * diff;
        col = mix(col, sky, 1.0 - exp(-0.003 * t * t));
    }
    col = pow(col, vec3f(0.4545));
    return vec4f(col, 1.0);
}
WGSL
