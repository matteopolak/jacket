#import "Jaic_Extensions";

LdBox :: struct { v: Long_Double; }
LdTagged :: struct { tag: s8; v: Long_Double; }
LdPair :: struct { a, b: Long_Double; }

ld_third :: () -> Long_Double #foreign longdouble;
ld_add :: (a: Long_Double, b: Long_Double) -> Long_Double #foreign longdouble;
ld_mix :: (i: s32, d: float64, x: Long_Double, y: Long_Double, j: s32) -> Long_Double #foreign longdouble;
ld_to_double :: (x: Long_Double) -> float64 #foreign longdouble;
ld_beyond_double :: (x: Long_Double) -> s32 #foreign longdouble;
box_scale :: (b: LdBox, k: Long_Double) -> LdBox #foreign longdouble;
tagged_bump :: (t: LdTagged) -> LdTagged #foreign longdouble;
pair_swap :: (p: LdPair) -> LdPair #foreign longdouble;
ld_spill :: (d0: float64, d1: float64, d2: float64, d3: float64, d4: float64, d5: float64, d6: float64,
    d7: float64, d8: float64, x: Long_Double, k: s32, y: Long_Double) -> Long_Double #foreign longdouble;
ld_apply :: (f: (Long_Double, Long_Double) -> Long_Double #c_call, a: Long_Double, b: Long_Double) -> Long_Double #foreign longdouble;
box_apply :: (f: (LdBox, s32) -> LdBox #c_call, b: LdBox) -> LdBox #foreign longdouble;
