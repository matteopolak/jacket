typedef struct { float x, y; } V2;
typedef struct { float x, y, z; } V3;
typedef struct { double a, b, c, d; } Big;
typedef struct { int a; char b; } Small;
typedef struct { long long a; double b; } Mixed;
typedef struct { double b; long long a; } Mixed2;
typedef struct { long long a, b, c; } Large;
V2 add_v2(V2 p, V2 q) { V2 r = {p.x + q.x, p.y + q.y}; return r; }
V3 scale_v3(V3 v, float k) { V3 r = {v.x * k, v.y * k, v.z * k}; return r; }
Big make_big(double base) { Big r = {base, base + 1, base + 2, base + 3}; return r; }
double sum_big(Big b) { return b.a + b.b + b.c + b.d; }
Small make_small(int a, char b) { Small r = {a, b}; return r; }
Mixed swap_mixed(Mixed2 m) { Mixed r = {m.a, m.b}; return r; }
Large add_large(Large x, Large y) { Large r = {x.a + y.a, x.b + y.b, x.c + y.c}; return r; }
V2 apply(V2 (*f)(V2, Big, Small), V2 v, Big b, Small s) { return f(v, b, s); }
Big apply_big(Big (*f)(Big, int)) { Big b = {1, 2, 3, 4}; return f(b, 10); }
Small apply_small(Small (*f)(Small)) { Small s = {7, 3}; return f(s); }
// More arguments than registers: x86-64 passes g, l and d9 on the stack, AArch64 only d9.
double many(long long a, long long b, long long c, long long d, long long e, long long f, long long g, Large l,
            double d1, double d2, double d3, double d4, double d5, double d6, double d7, double d8, double d9) {
    return a + b + c + d + e + f + g + l.a + l.b + l.c + d1 + d2 + d3 + d4 + d5 + d6 + d7 + d8 + d9 * 100;
}
double apply_many(double (*fn)(long long, long long, long long, long long, long long, long long, long long, Large,
                               double, double, double, double, double, double, double, double, double)) {
    Large l = {100, 200, 300};
    return fn(1, 2, 3, 4, 5, 6, 7, l, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 2);
}
// Microsoft x64 assigns argument positions rather than register classes: a float in position n
// is in XMMn, an integer in the n-th integer register, and the hidden result pointer of `Mixed`
// (16 bytes) takes position 0. The last two arguments of `f` are on the stack there.
Mixed apply_mixed(Mixed (*f)(double, int, float, long long, double, short)) { return f(1.5, 2, 3.25f, 4, 5.5, 6); }
V2 apply_mixed_v2(V2 (*f)(float, long long, double, int)) { return f(0.5f, 7, 2.25, 3); }
