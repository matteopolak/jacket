//! The `#asm` instruction table as editors see it: every mnemonic the lowering accepts,
//! with its operand forms, the CPU feature it needs on hardware and a one-line summary.
//!
//! Nothing here is a second list of instructions. Candidate spellings are expanded from
//! `PATTERNS` and kept only when the lowering's own lookups (`lookup_op`, `lookup_xop`,
//! `lookup_kop`, `lookup_vec`) accept them; forms, features and summaries are derived from
//! the decoded semantics by exhaustive matches, so a new semantic variant does not compile
//! until it is described here. The tests check that every spelling the lookups accept
//! (from the string literals of the `asm` sources) is listed.
use super::mask::{KOp, lookup_kop};
use super::scalar::{Rep, StrOp, XOp, lookup_xop};
use super::simd::{AesOp, B2, Conv, DupKind, FmaKind, GfOp, SOp, Sat, ShaOp, U1};
use super::vec::{Cvt, Lane, Shift, VOp, lookup_vec};
use super::{Alu, AsmReg, BitTest, Cond, FEATURES, Op, REG_CLASSES, ShiftKind, lookup_op};
use crate::ir::{BinOp, Ty};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// One `#asm` mnemonic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsmInstruction {
    /// As written in a block, e.g. `vpaddd`.
    pub mnemonic: String,
    /// Operand forms, most common first, e.g. `vpaddd dst: vec, a: vec, b: vec/mem`.
    pub forms: Vec<String>,
    /// The CPUID feature hardware needs for this spelling (`AVX2`), or `x86-64`.
    pub feature: &'static str,
    /// One sentence on what it does.
    pub description: String,
}

/// A register class a declaration may name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegisterClass {
    pub name: &'static str,
    pub description: &'static str,
}

/// Every supported mnemonic, sorted.
pub fn instructions() -> &'static [AsmInstruction] {
    static TABLE: OnceLock<Vec<AsmInstruction>> = OnceLock::new();
    TABLE.get_or_init(build)
}

/// The entry for `name` (a `.x`/`?T` size suffix must already be stripped). Spellings the
/// lowering accepts but the table lists differently (`addps` for an AVX-only `vpermd`
/// written `permd`) resolve to the listed one.
pub fn instruction(name: &str) -> Option<&'static AsmInstruction> {
    let table = instructions();
    let find = |n: &str| {
        table
            .binary_search_by(|i| i.mnemonic.as_str().cmp(n))
            .ok()
            .map(|at| &table[at])
    };
    let kind = decode(name)?;
    find(name)
        .or_else(|| find(&format!("v{name}")))
        .or_else(|| name.strip_prefix('v').and_then(find))
        // Lenient spellings (`movzxbw`, `lock_mov`): the entry with the same semantics.
        .or_else(|| {
            table
                .iter()
                .find(|i| !i.mnemonic.starts_with("lock_") && decode(&i.mnemonic) == Some(kind))
        })
}

/// Whether the lowering accepts `name` as a mnemonic.
pub fn is_supported(name: &str) -> bool {
    decode(name).is_some()
}

/// Feature modifiers accepted after `#asm` (`#asm AVX2 { ... }`).
pub fn feature_modifiers() -> &'static [&'static str] {
    FEATURES
}

/// The classes of `x: class;` declarations.
pub fn register_classes() -> Vec<RegisterClass> {
    REG_CLASSES
        .iter()
        .map(|&(name, reg, _)| RegisterClass {
            name,
            description: match (reg, name) {
                (AsmReg::Gpr, _) => "general-purpose register (a 64-bit local)",
                (AsmReg::Vec, "str") => {
                    "MMX register: a vector register used by the 8-byte `.q` forms"
                }
                (AsmReg::Vec, _) => {
                    "vector register: xmm, ymm or zmm by the size suffix (a 64-byte local)"
                }
                (AsmReg::Mask, "kmask") => "AVX-512 op-mask register (same as `omr`)",
                (AsmReg::Mask, _) => "AVX-512 op-mask register k0-k7 (a 64-bit local)",
            },
        })
        .collect()
}

/// Hardware register names accepted after `===`. Pins are parsed and ignored: registers are
/// not modelled, so these only document intent.
pub fn pin_names() -> Vec<String> {
    let mut out: Vec<String> = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    out.extend((8..16).map(|i| format!("r{i}")));
    for (prefix, count) in [("xmm", 16), ("ymm", 16), ("zmm", 32), ("k", 8)] {
        out.extend((0..count).map(|i| format!("{prefix}{i}")));
    }
    out
}

/// The class a fresh `name:` operand declares at `operand` (0-based) of `mnemonic`, given
/// the vector width in bytes (the `.x/.y/.z` suffix, else the block default).
pub fn inline_register_class(mnemonic: &str, width: u64, operand: usize) -> Option<&'static str> {
    Some(match decode(mnemonic)? {
        Kind::Core(_) | Kind::Scalar(_) => "gpr",
        Kind::Mask(..) => "omr",
        Kind::Vector(op) if operand == 0 => super::vec::dst_class(op, width),
        Kind::Vector(_) => "vec",
    })
}

// ---------------------------------------------------------------------------
// Decoding: the lowering's own lookups, in the order `asm_inst` tries them.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Core(Op),
    Scalar(XOp),
    Mask(KOp, Ty),
    Vector(VOp),
}

fn decode(name: &str) -> Option<Kind> {
    if let Some(base) = name.strip_prefix("lock_") {
        // `lock_` goes with the integer core and the 8/16-byte compare-exchange only.
        return match (lookup_op(base), lookup_xop(base)) {
            (Some(op), _) => Some(Kind::Core(op)),
            (None, Some(x @ XOp::CmpxchgPair(_))) => Some(Kind::Scalar(x)),
            _ => None,
        };
    }
    if let Some(op) = lookup_op(name) {
        return Some(Kind::Core(op));
    }
    if let Some(op) = lookup_xop(name) {
        return Some(Kind::Scalar(op));
    }
    if let Some((op, ty)) = lookup_kop(name) {
        return Some(Kind::Mask(op, ty));
    }
    lookup_vec(name)
        .or_else(|| name.strip_prefix('v').and_then(lookup_vec))
        .map(Kind::Vector)
}

/// Candidate spellings, `{a,b}` alternatives expanded. Vector mnemonics are written without
/// the `v`; both spellings are listed where hardware has them.
const PATTERNS: &[&str] = &[
    // Integer core.
    "mov",
    "movnti",
    "movbe",
    "lea",
    "xchg",
    "xadd",
    "cmpxchg",
    "{add,sub,adc,sbb,and,or,xor,cmp,test}",
    "{inc,dec,neg,not}",
    "{shl,sal,shr,sar,rol,ror}",
    "{bt,bts,btr,btc}",
    "{bsf,bsr,popcnt,lzcnt,tzcnt,bswap}",
    "{blsr,blsi,blsmsk}",
    "{imul,mul}",
    "{nop,mfence,lfence,sfence,pause,int3}",
    "{clc,stc,cmc}",
    "{rdtsc,rdtscp,rdrand,rdseed,cpuid}",
    "{movzx,movsx,movsxd}",
    "{set,cmov}{e,z,ne,nz,b,c,nae,ae,nb,nc,be,na,a,nbe,s,ns,o,no,l,nge,ge,nl,le,ng,g,nle,p,pe,np,po}",
    "lock_{add,sub,adc,sbb,and,or,xor,inc,dec,neg,not,xadd,xchg,cmpxchg,bts,btr,btc,cmpxchg8b,cmpxchg16b}",
    // Less common general-purpose instructions.
    "{div,idiv,cbw,cwde,cdqe,cwd,cdq,cqo,shld,shrd,rcl,rcr}",
    "{mulx,adcx,adox,andn,bextr,bzhi,pdep,pext,shlx,shrx,sarx,rorx}",
    "{crc32,crc32d,crc32q,lahf,sahf,xlat,xlatb}",
    "{prefetcht0,prefetcht1,prefetcht2,prefetchnta,prefetchw,prefetchwt1}",
    "{clflush,clflushopt,clwb,cldemote,cld,std}",
    "{cmpxchg8b,cmpxchg16b,int,xgetbv,stmxcsr,ldmxcsr,rdpid}",
    "{,rep_,repe_,repz_,repne_,repnz_}{movs,stos,lods,cmps,scas}",
    // Op-mask registers.
    "k{mov,and,andn,or,xor,xnor,add,not,shiftl,shiftr,test,ortest}{b,w,d,q}",
    "kunpck{bw,wd,dq}",
    // Vector moves and core lane operations.
    "mov{u,a}{ps,pd}",
    "movdq{u,a,u8,u16,u32,u64,a32,a64}",
    "{lddqu,movntdq,movntps,movntpd,movntdqa}",
    "{movd,movq,movss,movsd}",
    "broadcast{ss,sd}",
    "pbroadcast{b,w,d,q,mw2d,mb2q}",
    "broadcast{i,f}{32x2,128,32x4,64x2,32x8,64x4}",
    "{and,or,xor,andn}{ps,pd}",
    "p{and,or,xor,andn}{,d,q}",
    "pmull{w,d,q}",
    "{pshufd,shufps,shufpd,cvtdq2ps,cvtps2dq,cvttps2dq}",
    "{movmskps,movmskpd,pmovmskb}",
    "gather{d,q}{ps,pd}",
    "pgather{d,q}{d,q}",
    "{zeroupper,zeroall,emms}",
    "{add,sub,mul,div,min,max,sqrt}{ps,pd,ss,sd}",
    "p{add,sub,cmpeq,cmpgt,mins,maxs,minu,maxu,abs}{b,w,d,q}",
    "p{sll,srl,sra}{w,d,q}",
    // The wider SIMD set.
    "p{add,sub}{s,us}{b,w}",
    "pavg{b,w}",
    "pmulh{w,uw,rsw}",
    "pmul{udq,dq}",
    "{pmaddwd,pmaddubsw,psadbw,pdpbusd,pdpwssd,phminposuw}",
    "ph{add,sub}{w,d,sw}",
    "h{add,sub}{ps,pd}",
    "addsub{ps,pd}",
    "unpck{l,h}{ps,pd}",
    "punpck{l,h}{bw,wd,dq,qdq}",
    "pshuf{hw,lw,b}",
    "{palignr,alignd,alignq,pslldq,psrldq}",
    "pblend{w,d,vb}",
    "blend{ps,pd,vps,vpd,mps,mpd}",
    "pblendm{b,w,d,q}",
    "perm{ps,pd,b,w,d,q}",
    "permil{ps,pd}",
    "perm2{i,f}128",
    "shuf{i,f}{32x4,64x2}",
    "{insert,extract}{i,f}{128,32x4,64x2,32x8,64x4}",
    "{insertps,extractps}",
    "mov{h,l}{ps,pd}",
    "{movhlps,movlhps,movddup,movshdup,movsldup}",
    "pack{sswb,ssdw,uswb,usdw}",
    "cvt{dq,udq,qq,uqq}2{ps,pd}",
    "cvt{,t}{ps,pd}2{dq,udq,qq,uqq}",
    "{cvtps2pd,cvtpd2ps,cvtss2sd,cvtsd2ss}",
    "cvt{si,usi}2{ss,sd}",
    "cvt{,t}{ss,sd}2{si,usi}",
    "{,u}comis{s,d}",
    "{ptest,pternlogd,pternlogq,dpps,dppd}",
    "pclmul{qdq,lqlqdq,hqlqdq,lqhqdq,hqhqdq}",
    "aes{enc,enclast,dec,declast,imc,keygenassist}",
    "{cvtph2ps,cvtps2ph}",
    "sha1{rnds4,nexte,msg1,msg2}",
    "sha256{rnds2,msg1,msg2}",
    "gf2p8{mulb,affineqb,affineinvqb}",
    "{pmaskmovd,pmaskmovq,maskmovps,maskmovpd}",
    "{compress,expand}{ps,pd}",
    "pscatter{d,q}{d,q}",
    "scatter{d,q}{ps,pd}",
    "p{sllv,srlv,srav,rolv,rorv,sign,rol,ror,lzcnt,conflict}{b,w,d,q}",
    "popcnt{b,w,d,q}",
    "perm{t2,i2}{b,w,d,q,ps,pd}",
    "p{compress,expand,movm2,insr,extr,testm,testnm}{b,w,d,q}",
    "pmov{b,w,d,q}2m",
    "pmov{sx,zx}{bw,bd,bq,wd,wq,dq}",
    "pmov{,s,us}{wb,db,dw,qb,qw,qd}",
    "pcmp{,u}{b,w,d,q}",
    "{cmp,round,rndscale}{ps,pd,ss,sd}",
    "{rcp,rsqrt}{,14,28}{ps,pd}",
    "f{,n}m{add,sub}{132,213,231}{ps,pd,ss,sd}",
    "fm{addsub,subadd}{132,213,231}{ps,pd}",
];

fn expand(pattern: &str) -> Vec<String> {
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_owned()];
    };
    let close = open + pattern[open..].find('}').expect("closed alternative");
    let (head, tail) = (&pattern[..open], &pattern[close + 1..]);
    pattern[open + 1..close]
        .split(',')
        .flat_map(|alt| expand(&format!("{head}{alt}{tail}")))
        .collect()
}

fn build() -> Vec<AsmInstruction> {
    let mut table = BTreeMap::new();
    let mut add = |mnemonic: String, kind: Kind, feature: &'static str, vex: bool| {
        let entry = AsmInstruction {
            forms: forms(&mnemonic, kind, vex),
            description: describe(&mnemonic, kind),
            feature,
            mnemonic: mnemonic.clone(),
        };
        table.entry(mnemonic).or_insert(entry);
    };
    for pattern in PATTERNS {
        for name in expand(pattern) {
            let Some(kind) = decode(&name) else {
                continue;
            };
            match kind {
                Kind::Vector(op) => {
                    let (legacy, vex) = vector_features(&name, op);
                    if let Some(feature) = legacy {
                        add(name.clone(), kind, feature, false);
                    }
                    let v = format!("v{name}");
                    if let Some(feature) = vex
                        && decode(&v).is_some()
                    {
                        add(v, kind, feature, true);
                    }
                }
                _ => add(name.clone(), kind, core_feature(&name, kind), false),
            }
        }
    }
    table.into_values().collect()
}

// ---------------------------------------------------------------------------
// Features
// ---------------------------------------------------------------------------

fn core_feature(name: &str, kind: Kind) -> &'static str {
    let name = name.strip_prefix("lock_").unwrap_or(name);
    match kind {
        Kind::Core(op) => match op {
            Op::Movbe => "MOVBE",
            Op::Popcnt => "POPCNT",
            Op::Lzcnt => "LZCNT",
            Op::Tzcnt | Op::Blsr | Op::Blsi | Op::Blsmsk => "BMI1",
            Op::Cmovcc(_) => "CMOV",
            Op::Rdtsc => "TSC",
            Op::Rdtscp => "RDTSCP",
            Op::Rdrand if name == "rdseed" => "RDSEED",
            Op::Rdrand => "RDRAND",
            Op::Nop if name == "sfence" => "SSE",
            Op::Nop if name != "nop" => "SSE2",
            Op::Pause => "SSE2",
            Op::Mov if name == "movnti" => "SSE2",
            Op::Mov
            | Op::Movzx
            | Op::Movsx
            | Op::Lea
            | Op::Xchg
            | Op::Xadd
            | Op::Cmpxchg
            | Op::Alu(_)
            | Op::Inc
            | Op::Dec
            | Op::Neg
            | Op::Not
            | Op::Shift(_)
            | Op::Bt(_)
            | Op::Bsf
            | Op::Bsr
            | Op::Bswap
            | Op::Imul
            | Op::Mul
            | Op::Setcc(_)
            | Op::Nop
            | Op::Int3
            | Op::SetCarry(_)
            | Op::Cpuid => "x86-64",
        },
        Kind::Scalar(op) => match op {
            XOp::Mulx | XOp::Bzhi | XOp::Pdep | XOp::Pext | XOp::ShiftX(_) | XOp::Rorx => "BMI2",
            XOp::Andn | XOp::Bextr => "BMI1",
            XOp::AddCarry(_) => "ADX",
            XOp::Crc32 => "SSE4_2",
            XOp::Lahf | XOp::Sahf => "LAHF_SAHF",
            XOp::Hint => match name {
                "prefetchw" => "PREFETCHW",
                "prefetchwt1" => "PREFETCHWT1",
                "clflush" => "CLFLUSH",
                "clflushopt" => "CLFLUSHOPT",
                "clwb" => "CLWB",
                "cldemote" => "CLDEMOTE",
                _ => "SSE",
            },
            XOp::CmpxchgPair(Ty::I32) => "CX8",
            XOp::CmpxchgPair(_) => "CMPXCHG16B",
            XOp::Xgetbv => "XSAVE",
            XOp::Stmxcsr | XOp::Ldmxcsr => "SSE",
            XOp::Rdpid => "RDPID",
            XOp::Div(_)
            | XOp::WidenA(_)
            | XOp::SignFill(_)
            | XOp::DoubleShift(_)
            | XOp::RotateCarry(_)
            | XOp::Xlat
            | XOp::Direction(_)
            | XOp::Str(..)
            | XOp::Int => "x86-64",
        },
        Kind::Mask(op, ty) => match (op, ty) {
            (KOp::Add | KOp::Test, Ty::I8 | Ty::I16) => "AVX512DQ",
            (KOp::Unpack, Ty::I16) => "AVX512F",
            (_, Ty::I8) => "AVX512DQ",
            (_, Ty::I16) => "AVX512F",
            _ => "AVX512BW",
        },
        Kind::Vector(_) => unreachable!("vector features come from vector_features"),
    }
}

/// Legacy SSE form and VEX/EVEX (`v`) form: `sse("SSE2", true)` has both (`AVX2` for the
/// integer `v` form, `AVX` for floats); `avx(..)` has only the `v` form.
fn sse(feature: &'static str, int: bool) -> (Option<&'static str>, Option<&'static str>) {
    (
        Some(feature),
        Some(if int {
            "AVX2"
        } else {
            "AVX"
        }),
    )
}

fn avx(feature: &'static str) -> (Option<&'static str>, Option<&'static str>) {
    (None, Some(feature))
}

/// `SSE` for single precision (`ps`/`ss`), `SSE2` for double.
fn fp(name: &str) -> (Option<&'static str>, Option<&'static str>) {
    if name.ends_with("ps") || name.ends_with("ss") {
        sse("SSE", false)
    } else {
        sse("SSE2", false)
    }
}

fn byte_word(ty: Ty, small: &'static str, large: &'static str) -> &'static str {
    if matches!(ty, Ty::I8 | Ty::I16) {
        small
    } else {
        large
    }
}

fn vector_features(name: &str, op: VOp) -> (Option<&'static str>, Option<&'static str>) {
    match op {
        VOp::Move => match name {
            "movups" | "movaps" | "movntps" => sse("SSE", false),
            "movupd" | "movapd" | "movntpd" => sse("SSE2", false),
            "movdqu" | "movdqa" | "movntdq" => sse("SSE2", true),
            "lddqu" => sse("SSE3", true),
            "movntdqa" => sse("SSE4_1", true),
            "movdqu8" | "movdqu16" => avx("AVX512BW"),
            _ => avx("AVX512F"),
        },
        VOp::MovScalarInt(_) => sse("SSE2", false),
        VOp::MovScalarFloat(ty) => sse(
            if ty == Ty::F32 {
                "SSE"
            } else {
                "SSE2"
            },
            false,
        ),
        VOp::Broadcast(_) => match name {
            "broadcastss" | "broadcastsd" | "broadcastf128" => avx("AVX"),
            "pbroadcastb" | "pbroadcastw" | "pbroadcastd" | "pbroadcastq" | "broadcasti128" => {
                avx("AVX2")
            }
            _ if name.contains("32x2") || name.contains("64x2") || name.contains("32x8") => {
                avx("AVX512DQ")
            }
            _ => avx("AVX512F"),
        },
        VOp::Bin(lane, ty, _) => match lane {
            Lane::FAdd | Lane::FSub | Lane::FMul | Lane::FDiv | Lane::FMin | Lane::FMax => fp(name),
            Lane::And | Lane::Or | Lane::Xor | Lane::AndNot => {
                if !name.starts_with('p') {
                    fp(name)
                } else if name.ends_with('d') || name.ends_with('q') {
                    avx("AVX512F")
                } else {
                    sse("SSE2", true)
                }
            }
            Lane::Mul => match ty {
                Ty::I16 => sse("SSE2", true),
                Ty::I32 => sse("SSE4_1", true),
                _ => avx("AVX512DQ"),
            },
            Lane::Add | Lane::Sub => sse("SSE2", true),
            Lane::CmpEq if ty == Ty::I64 => sse("SSE4_1", true),
            Lane::CmpGt if ty == Ty::I64 => sse("SSE4_2", true),
            Lane::CmpEq | Lane::CmpGt => sse("SSE2", true),
            Lane::MinS | Lane::MaxS | Lane::MinU | Lane::MaxU => match (lane, ty) {
                (_, Ty::I64) => avx("AVX512F"),
                (Lane::MinS | Lane::MaxS, Ty::I16) | (Lane::MinU | Lane::MaxU, Ty::I8) => {
                    sse("SSE2", true)
                }
                _ => sse("SSE4_1", true),
            },
        },
        VOp::Sqrt(..) | VOp::Shufps => fp(name),
        VOp::Abs(Ty::I64) | VOp::Shift(Shift::Arith, Ty::I64) => avx("AVX512F"),
        VOp::Abs(_) => sse("SSSE3", true),
        VOp::Shift(..) | VOp::Pshufd => sse("SSE2", true),
        VOp::Cvt(_) => sse("SSE2", false),
        VOp::Movmsk(4) => sse("SSE", false),
        VOp::Movmsk(8) => sse("SSE2", false),
        VOp::Movmsk(_) => sse("SSE2", true),
        VOp::Gather(..) => avx("AVX2"),
        VOp::Nop if name == "emms" => (Some("MMX"), None),
        VOp::Nop => avx("AVX"),
        VOp::Ext(s) => simd_features(name, s),
    }
}

fn simd_features(name: &str, op: SOp) -> (Option<&'static str>, Option<&'static str>) {
    match op {
        SOp::Bin(b, ty) => match b {
            B2::AddSatS | B2::AddSatU | B2::SubSatS | B2::SubSatU | B2::Avg => sse("SSE2", true),
            B2::MulHiS | B2::MulHiU => sse("SSE2", true),
            B2::MulHrs | B2::Sign => sse("SSSE3", true),
            B2::Shlv | B2::Shrv | B2::Sarv if ty == Ty::I16 => avx("AVX512BW"),
            B2::Sarv if ty == Ty::I64 => avx("AVX512F"),
            B2::Shlv | B2::Shrv | B2::Sarv => avx("AVX2"),
            B2::Rolv | B2::Rorv => avx("AVX512F"),
        },
        SOp::RotImm(..) | SOp::Valign(_) | SOp::Shuf128(_) | SOp::Ternlog(_) => avx("AVX512F"),
        SOp::Scatter(..) => avx("AVX512F"),
        SOp::Unary(u, ty) => match u {
            U1::Popcnt => avx(byte_word(ty, "AVX512_BITALG", "AVX512_VPOPCNTDQ")),
            U1::Lzcnt | U1::Conflict => avx("AVX512CD"),
            U1::Rcp | U1::Rsqrt if name.contains("14") => avx("AVX512F"),
            U1::Rcp | U1::Rsqrt if name.contains("28") => avx("AVX512ER"),
            U1::Rcp | U1::Rsqrt => fp(name),
        },
        SOp::Round(..) if name.starts_with("rndscale") => avx("AVX512F"),
        SOp::Round(..) | SOp::Dp(_) | SOp::Insertps | SOp::Extractps => sse("SSE4_1", false),
        SOp::BlendVar(_) => sse("SSE4_1", !name.starts_with('b')),
        SOp::Fma(..) => avx("FMA"),
        SOp::Horizontal(_, _, Ty::F32 | Ty::F64) | SOp::AddSub(_) | SOp::Dup(_) => {
            sse("SSE3", false)
        }
        SOp::Horizontal(..) | SOp::Pmaddubsw | SOp::Palignr | SOp::Pshufb => sse("SSSE3", true),
        SOp::Pmaddwd | SOp::Psadbw | SOp::MulEven(false) | SOp::ShufHalf(_) => sse("SSE2", true),
        SOp::ByteShift(_) => sse("SSE2", true),
        SOp::MulEven(true) | SOp::Phminposuw | SOp::Ptest | SOp::Extend(..) => sse("SSE4_1", true),
        SOp::DotAcc(_) => avx("AVX512_VNNI"),
        SOp::Unpack(..) if name.starts_with('u') => fp(name),
        SOp::Unpack(..) => sse("SSE2", true),
        SOp::Shufpd | SOp::MovHalf(_) if name.ends_with("pd") => sse("SSE2", false),
        SOp::Shufpd | SOp::MovHalf(_) | SOp::Movhlps | SOp::Movlhps => sse("SSE", false),
        SOp::BlendImm(_) => match name {
            "pblendd" => avx("AVX2"),
            "pblendw" => sse("SSE4_1", true),
            _ => sse("SSE4_1", false),
        },
        SOp::PermTwo(Ty::I8, _) => avx("AVX512VBMI"),
        SOp::BlendMask(ty) | SOp::IntCmp(ty, ..) | SOp::Testm(ty, _) | SOp::PermTwo(ty, _) => {
            avx(byte_word(ty, "AVX512BW", "AVX512F"))
        }
        SOp::Narrow(from, ..) => avx(if from == Ty::I16 {
            "AVX512BW"
        } else {
            "AVX512F"
        }),
        SOp::MaskToVec(ty) | SOp::VecToMask(ty) => avx(byte_word(ty, "AVX512BW", "AVX512DQ")),
        SOp::BroadcastMask(_) => avx("AVX512CD"),
        SOp::Compress(ty) | SOp::Expand(ty) => avx(byte_word(ty, "AVX512_VBMI2", "AVX512F")),
        SOp::Perm(ty) => match ty {
            Ty::I8 => avx("AVX512VBMI"),
            Ty::I16 => avx("AVX512BW"),
            _ => avx("AVX2"),
        },
        SOp::PermilImm(_) => avx("AVX"),
        SOp::Perm2x128 | SOp::Insert(..) | SOp::Extract(..) => {
            if name.contains("128") {
                // `perm2f128` / `insertf128`: AVX; the integer `i128` forms: AVX2.
                avx(if name.contains("f128") {
                    "AVX"
                } else {
                    "AVX2"
                })
            } else if name.contains("32x8") || name.contains("64x2") {
                avx("AVX512DQ")
            } else {
                avx("AVX512F")
            }
        }
        SOp::Pinsr(Ty::I16) | SOp::Pextr(Ty::I16) => sse("SSE2", true),
        SOp::Pinsr(_) | SOp::Pextr(_) => sse("SSE4_1", true),
        SOp::Pack(Ty::I32, true) => sse("SSE4_1", true),
        SOp::Pack(..) => sse("SSE2", true),
        SOp::Cvt(conv) => match conv {
            _ if name.contains("qq") => avx("AVX512DQ"),
            _ if name.contains("2u") || name.starts_with("cvtu") => avx("AVX512F"),
            Conv::GprToScalar(Ty::F32, _) | Conv::ScalarToGpr(Ty::F32, ..) => sse("SSE", false),
            Conv::IntToFloat(..)
            | Conv::FloatToInt(..)
            | Conv::FloatToFloat(..)
            | Conv::ScalarFloat(..)
            | Conv::GprToScalar(..)
            | Conv::ScalarToGpr(..) => sse("SSE2", false),
        },
        SOp::Comis(Ty::F32) => sse("SSE", false),
        SOp::Comis(_) => sse("SSE2", false),
        SOp::FCmp(..) => fp(name),
        SOp::Pclmul(_) => (Some("PCLMULQDQ"), Some("PCLMULQDQ")),
        SOp::Aes(_) => (Some("AES"), Some("AES")),
        SOp::Half(_) => avx("F16C"),
        SOp::Sha(_) => (Some("SHA"), None),
        SOp::Gf(_) => (Some("GFNI"), Some("GFNI")),
        SOp::MaskMov(_) if name.starts_with('p') => avx("AVX2"),
        SOp::MaskMov(_) => avx("AVX"),
    }
}

// ---------------------------------------------------------------------------
// Operand forms
// ---------------------------------------------------------------------------

const BIN: &str = "dst: vec, src: vec/mem";
const BIN3: &str = "dst: vec, a: vec, b: vec/mem";
const BIN_IMM: &str = "dst: vec, src: vec/mem, imm";
const BIN3_IMM: &str = "dst: vec, a: vec, b: vec/mem, imm";
const UNARY: &str = "dst: vec, src: vec/mem";

/// Two-source vector forms: the three-operand one first for `v` spellings.
fn binary(vex: bool, imm: bool) -> Vec<&'static str> {
    let (two, three) = if imm {
        (BIN_IMM, BIN3_IMM)
    } else {
        (BIN, BIN3)
    };
    if vex {
        vec![three, two]
    } else {
        vec![two, three]
    }
}

fn forms(name: &str, kind: Kind, vex: bool) -> Vec<String> {
    let shapes: Vec<&str> = match kind {
        Kind::Core(op) => match op {
            Op::Mov => vec!["dst: gpr/mem, src: gpr/mem/imm"],
            Op::Movzx | Op::Movsx => vec!["dst: gpr, src: gpr/mem"],
            Op::Movbe => vec!["dst: gpr/mem, src: gpr/mem"],
            Op::Lea => vec!["dst: gpr, src: mem"],
            Op::Xchg => vec!["a: gpr/mem, b: gpr/mem"],
            Op::Xadd => vec!["dst: gpr/mem, src: gpr"],
            Op::Cmpxchg => vec!["dst: gpr/mem, src: gpr, acc: gpr"],
            Op::Alu(Alu::Cmp | Alu::Test) => vec!["a: gpr/mem, b: gpr/mem/imm"],
            Op::Alu(_) => vec!["dst: gpr/mem, src: gpr/mem/imm"],
            Op::Inc | Op::Dec | Op::Neg | Op::Not => vec!["dst: gpr/mem"],
            Op::Shift(_) => vec!["dst: gpr/mem, count: gpr/imm", "dst: gpr/mem"],
            Op::Bt(_) => vec!["base: gpr/mem, bit: gpr/imm"],
            Op::Bsf | Op::Bsr | Op::Popcnt | Op::Lzcnt | Op::Tzcnt => {
                vec!["dst: gpr, src: gpr/mem"]
            }
            Op::Bswap => vec!["dst: gpr"],
            Op::Blsr | Op::Blsi | Op::Blsmsk => vec!["dst: gpr, src: gpr/mem"],
            Op::Imul => vec![
                "dst: gpr, src: gpr/mem",
                "dst: gpr, src: gpr/mem, imm",
                "hi: gpr, lo: gpr, src: gpr/mem",
            ],
            Op::Mul => vec!["hi: gpr, lo: gpr, src: gpr/mem"],
            Op::Setcc(_) => vec!["dst: gpr/mem"],
            Op::Cmovcc(_) => vec!["dst: gpr, src: gpr/mem"],
            Op::Nop | Op::Pause | Op::Int3 | Op::SetCarry(_) => vec![""],
            Op::Rdtsc => vec!["hi: gpr, lo: gpr"],
            Op::Rdtscp => vec!["hi: gpr, lo: gpr, id: gpr"],
            Op::Rdrand => vec!["dst: gpr"],
            Op::Cpuid => vec!["a: gpr, b: gpr, c: gpr, d: gpr"],
        },
        Kind::Scalar(op) => match op {
            XOp::Div(_) => vec![
                "hi: gpr, lo: gpr, divisor: gpr/mem",
                "ax: gpr, divisor: gpr/mem",
            ],
            XOp::WidenA(_) => vec!["a: gpr"],
            XOp::SignFill(_) => vec!["d: gpr, a: gpr"],
            XOp::DoubleShift(_) => vec!["dst: gpr/mem, src: gpr, count: gpr/imm"],
            XOp::RotateCarry(_) => vec!["dst: gpr/mem, count: gpr/imm", "dst: gpr/mem"],
            XOp::Mulx => vec!["hi: gpr, lo: gpr, src: gpr/mem, d: gpr"],
            XOp::AddCarry(_) => vec!["dst: gpr, src: gpr/mem"],
            XOp::Andn => vec!["dst: gpr, a: gpr, b: gpr/mem"],
            XOp::Bextr => vec!["dst: gpr, src: gpr/mem, control: gpr"],
            XOp::Bzhi => vec!["dst: gpr, src: gpr/mem, index: gpr"],
            XOp::Pdep | XOp::Pext => vec!["dst: gpr, src: gpr, mask: gpr/mem"],
            XOp::ShiftX(_) => vec!["dst: gpr, src: gpr/mem, count: gpr"],
            XOp::Rorx => vec!["dst: gpr, src: gpr/mem, imm"],
            XOp::Crc32 => vec!["crc: gpr, data: gpr/mem"],
            XOp::Lahf | XOp::Sahf => vec!["a: gpr"],
            XOp::Xlat => vec!["table: gpr, a: gpr"],
            XOp::Hint | XOp::Stmxcsr | XOp::Ldmxcsr => vec!["addr: mem"],
            XOp::Direction(_) => vec![""],
            XOp::CmpxchgPair(_) => vec!["d: gpr, a: gpr, target: mem, c: gpr, b: gpr"],
            XOp::Str(op, rep) => {
                let regs = match op {
                    StrOp::Movs | StrOp::Cmps => "di: gpr, si: gpr",
                    StrOp::Stos | StrOp::Scas => "di: gpr, a: gpr",
                    StrOp::Lods => "a: gpr, si: gpr",
                };
                return vec![match rep {
                    Rep::Once => format!("{name}.size {regs}"),
                    Rep::Count | Rep::Repe | Rep::Repne => format!("{name}.size {regs}, c: gpr"),
                }];
            }
            XOp::Int => vec!["imm"],
            XOp::Xgetbv => vec!["d: gpr, a: gpr, c: gpr"],
            XOp::Rdpid => vec!["dst: gpr"],
        },
        Kind::Mask(op, _) => match op {
            KOp::Mov => vec!["dst: omr/gpr/mem, src: omr/gpr/mem"],
            KOp::Bin(_) | KOp::AndNot | KOp::Xnor | KOp::Add | KOp::Unpack => {
                vec!["dst: omr, a: omr, b: omr"]
            }
            KOp::Not => vec!["dst: omr, src: omr"],
            KOp::ShiftLeft | KOp::ShiftRight => vec!["dst: omr, src: omr, imm"],
            KOp::Test | KOp::OrTest => vec!["a: omr, b: omr"],
        },
        Kind::Vector(op) => vector_forms(op, vex),
    };
    shapes
        .into_iter()
        .map(|s| {
            if s.is_empty() {
                name.to_owned()
            } else {
                format!("{name} {s}")
            }
        })
        .collect()
}

fn vector_forms(op: VOp, vex: bool) -> Vec<&'static str> {
    match op {
        VOp::Move => vec!["dst: vec/mem, src: vec/mem"],
        VOp::MovScalarInt(_) => vec!["dst: vec/gpr/mem, src: vec/gpr/mem"],
        VOp::MovScalarFloat(_) => vec!["dst: vec/mem, src: vec/mem", "dst: vec, a: vec, b: vec"],
        VOp::Broadcast(_) => vec!["dst: vec, src: vec/gpr/mem"],
        VOp::Bin(..) => binary(vex, false),
        VOp::Sqrt(_, true) => binary(vex, false),
        VOp::Sqrt(_, false) | VOp::Abs(_) | VOp::Cvt(_) => vec![UNARY],
        VOp::Shift(..) => {
            if vex {
                vec![
                    "dst: vec, src: vec/mem, count: vec/imm",
                    "dst: vec, count: vec/imm",
                ]
            } else {
                vec![
                    "dst: vec, count: vec/imm",
                    "dst: vec, src: vec/mem, count: vec/imm",
                ]
            }
        }
        VOp::Pshufd => vec![BIN_IMM],
        VOp::Shufps => binary(vex, true),
        VOp::Movmsk(_) => vec!["dst: gpr, src: vec"],
        VOp::Gather(..) => vec![
            "dst: vec, [base + vindex*scale], mask: vec",
            "dst: vec &k, [base + vindex*scale]",
        ],
        VOp::Nop => vec![""],
        VOp::Ext(s) => simd_forms(s, vex),
    }
}

fn simd_forms(op: SOp, vex: bool) -> Vec<&'static str> {
    match op {
        SOp::Bin(..)
        | SOp::Horizontal(..)
        | SOp::AddSub(_)
        | SOp::Pmaddwd
        | SOp::Pmaddubsw
        | SOp::Psadbw
        | SOp::MulEven(_)
        | SOp::Unpack(..)
        | SOp::Pshufb
        | SOp::Pack(..)
        | SOp::BlendMask(_)
        | SOp::Movhlps
        | SOp::Movlhps
        | SOp::Pclmul(Some(_))
        | SOp::Cvt(Conv::ScalarFloat(..)) => binary(vex, false),
        SOp::IntCmp(_, _, Some(_)) | SOp::Testm(..) => vec!["dst: omr, a: vec, b: vec/mem"],
        SOp::IntCmp(_, _, None) => vec!["dst: omr, a: vec, b: vec/mem, imm"],
        SOp::Aes(AesOp::Imc) => vec![UNARY],
        SOp::Aes(AesOp::KeygenAssist) => vec![BIN_IMM],
        SOp::Aes(_) | SOp::Gf(GfOp::Mul) => binary(vex, false),
        SOp::Sha(ShaOp::Sha1Rnds4) => vec![BIN_IMM],
        SOp::Sha(ShaOp::Sha256Rnds2) => vec!["cdgh: vec, abef: vec, wk: vec"],
        SOp::Sha(_) => vec![BIN],
        SOp::RotImm(..) | SOp::ShufHalf(_) => vec![BIN_IMM],
        SOp::Round(_, false) => vec![BIN_IMM],
        SOp::Unary(..) | SOp::Dup(_) | SOp::Extend(..) | SOp::Phminposuw => vec![UNARY],
        SOp::Cvt(Conv::GprToScalar(..)) => {
            if vex {
                vec!["dst: vec, a: vec, b: gpr/mem", "dst: vec, src: gpr/mem"]
            } else {
                vec!["dst: vec, src: gpr/mem", "dst: vec, a: vec, b: gpr/mem"]
            }
        }
        SOp::Cvt(Conv::ScalarToGpr(..)) => vec!["dst: gpr, src: vec/mem"],
        SOp::Cvt(_) => vec![UNARY],
        SOp::Fma(..) => vec![BIN3],
        SOp::DotAcc(_) => vec!["acc: vec, a: vec, b: vec/mem"],
        SOp::Round(_, true)
        | SOp::Shufpd
        | SOp::Palignr
        | SOp::Valign(_)
        | SOp::BlendImm(_)
        | SOp::Shuf128(_)
        | SOp::Perm2x128
        | SOp::Insertps
        | SOp::Dp(_)
        | SOp::Pclmul(None)
        | SOp::FCmp(..)
        | SOp::Ternlog(_)
        | SOp::Gf(GfOp::Affine(_)) => binary(vex, true),
        SOp::ByteShift(_) => {
            if vex {
                vec!["dst: vec, src: vec, imm", "dst: vec, imm"]
            } else {
                vec!["dst: vec, imm", "dst: vec, src: vec, imm"]
            }
        }
        SOp::BlendVar(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: vec/mem, mask: vec"]
            } else {
                vec!["dst: vec, src: vec/mem, mask: vec"]
            }
        }
        SOp::Perm(Ty::I64) => vec!["dst: vec, idx: vec, src: vec/mem", BIN_IMM],
        SOp::Perm(_) => vec!["dst: vec, idx: vec, src: vec/mem"],
        SOp::PermTwo(_, false) => vec!["dst: vec, idx: vec, table: vec/mem"],
        SOp::PermTwo(_, true) => vec!["idx: vec, a: vec, table: vec/mem"],
        SOp::PermilImm(_) => vec![BIN_IMM, "dst: vec, src: vec, control: vec/mem"],
        SOp::Insert(..) => vec![BIN3_IMM],
        SOp::Extract(..) => vec!["dst: vec/mem, src: vec, imm"],
        SOp::Pinsr(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: gpr/mem, imm"]
            } else {
                vec!["dst: vec, src: gpr/mem, imm"]
            }
        }
        SOp::Pextr(_) | SOp::Extractps => vec!["dst: gpr/mem, src: vec, imm"],
        SOp::MovHalf(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: mem", "dst: mem, src: vec"]
            } else {
                vec!["dst: vec, src: mem", "dst: mem, src: vec"]
            }
        }
        SOp::Narrow(..) => vec!["dst: vec/mem, src: vec"],
        SOp::Half(false) => vec![UNARY],
        SOp::Half(true) => vec!["dst: vec/mem, src: vec, imm"],
        SOp::Comis(_) | SOp::Ptest => vec!["a: vec, b: vec/mem"],
        SOp::MaskToVec(_) | SOp::BroadcastMask(_) => vec!["dst: vec, src: omr"],
        SOp::VecToMask(_) => vec!["dst: omr, src: vec"],
        SOp::Compress(_) => vec!["dst: vec/mem &k, src: vec"],
        SOp::Expand(_) => vec!["dst: vec &k, src: vec/mem"],
        SOp::Scatter(..) => vec!["[base + vindex*scale] &k, src: vec"],
        SOp::MaskMov(_) => vec![
            "dst: vec, mask: vec, src: mem",
            "dst: mem, mask: vec, src: vec",
        ],
    }
}

// ---------------------------------------------------------------------------
// Descriptions
// ---------------------------------------------------------------------------

fn lanes(ty: Ty) -> &'static str {
    match ty {
        Ty::I8 => "bytes",
        Ty::I16 => "words",
        Ty::I32 => "dwords",
        Ty::I64 => "qwords",
        Ty::F32 => "single-precision floats",
        Ty::F64 => "double-precision floats",
        _ => "elements",
    }
}

fn float_lanes(ty: Ty, scalar: bool) -> String {
    let precision = if matches!(ty, Ty::F32 | Ty::I32) {
        "single"
    } else {
        "double"
    };
    if scalar {
        format!("the low {precision}-precision float (upper lanes from the first source)")
    } else {
        format!("packed {precision}-precision floats")
    }
}

fn bits_of(ty: Ty) -> u64 {
    ty.size() * 8
}

fn condition(c: Cond) -> &'static str {
    match c {
        Cond::E => "equal / zero (ZF=1)",
        Cond::Ne => "not equal / not zero (ZF=0)",
        Cond::B => "below, unsigned (CF=1)",
        Cond::Ae => "above or equal, unsigned (CF=0)",
        Cond::Be => "below or equal, unsigned (CF=1 or ZF=1)",
        Cond::A => "above, unsigned (CF=0 and ZF=0)",
        Cond::S => "negative (SF=1)",
        Cond::Ns => "not negative (SF=0)",
        Cond::O => "overflow (OF=1)",
        Cond::No => "no overflow (OF=0)",
        Cond::L => "less, signed (SF!=OF)",
        Cond::Ge => "greater or equal, signed (SF=OF)",
        Cond::Le => "less or equal, signed (ZF=1 or SF!=OF)",
        Cond::G => "greater, signed (ZF=0 and SF=OF)",
        Cond::P => "parity even (PF=1)",
        Cond::Np => "parity odd (PF=0)",
    }
}

fn describe(name: &str, kind: Kind) -> String {
    if let Some(base) = name.strip_prefix("lock_") {
        return format!(
            "Atomic with a memory destination: {}",
            lower_first(&describe(base, kind))
        );
    }
    match kind {
        Kind::Core(op) => describe_core(name, op),
        Kind::Scalar(op) => describe_scalar(name, op),
        Kind::Mask(op, ty) => describe_mask(op, ty),
        Kind::Vector(op) => describe_vector(name.strip_prefix('v').unwrap_or(name), op),
    }
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_lowercase().chain(c).collect(),
        None => String::new(),
    }
}

fn describe_core(name: &str, op: Op) -> String {
    let s: &str = match op {
        Op::Mov if name == "movnti" => {
            "Store a register to memory, hinting that it bypass the cache."
        }
        Op::Mov => "Copy the source into the destination.",
        Op::Movzx => "Copy a narrower source into the destination, zero-filling the upper bits.",
        Op::Movsx => "Copy a narrower source into the destination, sign-extending it.",
        Op::Movbe => "Copy with the byte order reversed (big-endian loads and stores).",
        Op::Lea => "Compute the address of a memory operand without accessing memory.",
        Op::Xchg => "Swap the two operands; with a memory operand the swap is atomic.",
        Op::Xadd => {
            "Add the source into the destination and hand the old destination back in the source."
        }
        Op::Cmpxchg => {
            "Store the source if the destination equals the accumulator, else load the destination into it; ZF tells which."
        }
        Op::Alu(alu) => match alu {
            Alu::Add => "Add the source to the destination, setting the flags.",
            Alu::Sub => "Subtract the source from the destination, setting the flags.",
            Alu::Adc => "Add the source and the carry flag to the destination.",
            Alu::Sbb => "Subtract the source and the carry flag from the destination.",
            Alu::And => "Bitwise AND into the destination; clears CF and OF.",
            Alu::Or => "Bitwise OR into the destination; clears CF and OF.",
            Alu::Xor => "Bitwise exclusive OR into the destination; clears CF and OF.",
            Alu::Cmp => "Set the flags from a - b without storing the difference.",
            Alu::Test => "Set the flags from a AND b without storing the result.",
        },
        Op::Inc => "Add one; CF is left as it was.",
        Op::Dec => "Subtract one; CF is left as it was.",
        Op::Neg => "Replace the operand with its two's-complement negation.",
        Op::Not => "Invert every bit; the flags are untouched.",
        Op::Shift(kind) => match kind {
            ShiftKind::Shl if name == "sal" => {
                "Shift left (same as shl), filling with zeros; the count defaults to 1."
            }
            ShiftKind::Shl => "Shift left, filling with zeros; the count defaults to 1.",
            ShiftKind::Shr => {
                "Shift right, filling with zeros (unsigned divide by a power of two)."
            }
            ShiftKind::Sar => {
                "Shift right, filling with copies of the sign bit (signed divide, rounding down)."
            }
            ShiftKind::Rol => "Rotate the bits left; bits leaving the top re-enter at the bottom.",
            ShiftKind::Ror => "Rotate the bits right; bits leaving the bottom re-enter at the top.",
        },
        Op::Bt(kind) => match kind {
            BitTest::Bt => "Copy the selected bit into CF.",
            BitTest::Bts => "Copy the selected bit into CF, then set it.",
            BitTest::Btr => "Copy the selected bit into CF, then clear it.",
            BitTest::Btc => "Copy the selected bit into CF, then flip it.",
        },
        Op::Bsf => {
            "Index of the lowest set bit; ZF=1 and the destination unchanged when the source is zero."
        }
        Op::Bsr => {
            "Index of the highest set bit; ZF=1 and the destination unchanged when the source is zero."
        }
        Op::Popcnt => "Count the set bits of the source.",
        Op::Lzcnt => "Count the zero bits above the highest set bit (the width for zero).",
        Op::Tzcnt => "Count the zero bits below the lowest set bit (the width for zero).",
        Op::Bswap => "Reverse the order of the bytes in a register.",
        Op::Blsr => "Clear the lowest set bit of the source.",
        Op::Blsi => "Keep only the lowest set bit of the source.",
        Op::Blsmsk => "Set every bit up to and including the lowest set bit of the source.",
        Op::Imul => {
            "Signed multiply: two-operand and immediate forms keep the low half; `hi, lo, src` gives the full product."
        }
        Op::Mul => "Unsigned widening multiply: hi:lo = lo * src.",
        Op::Setcc(c) => return format!("Set a byte to 1 when {}, else to 0.", condition(c)),
        Op::Cmovcc(c) => {
            return format!(
                "Copy the source into the destination when {}.",
                condition(c)
            );
        }
        Op::Nop if name == "nop" => "Do nothing.",
        Op::Nop => {
            "Memory fence: order earlier loads/stores before later ones (lowered code is already ordered)."
        }
        Op::Pause => "Spin-loop hint: lets the core relax while waiting.",
        Op::Int3 => "Breakpoint: stop in the debugger.",
        Op::SetCarry(Some(false)) => "Clear the carry flag.",
        Op::SetCarry(Some(true)) => "Set the carry flag.",
        Op::SetCarry(None) => "Flip the carry flag.",
        Op::Rdtsc => "Read the time-stamp counter into hi:lo (a monotonic counter here).",
        Op::Rdtscp => "Read the time-stamp counter into hi:lo, and the processor id (0 here).",
        Op::Rdrand if name == "rdseed" => "Store a random seed value; CF=1 on success.",
        Op::Rdrand => "Store a random value; CF=1 on success.",
        Op::Cpuid => "Query the processor (every output reads 0 here: no features are reported).",
    };
    s.to_owned()
}

fn describe_scalar(name: &str, op: XOp) -> String {
    let s: &str = match op {
        XOp::Div(false) => {
            "Unsigned divide hi:lo by the divisor: quotient into lo, remainder into hi; traps on zero or overflow."
        }
        XOp::Div(true) => {
            "Signed divide hi:lo by the divisor: quotient into lo, remainder into hi; traps on zero or overflow."
        }
        XOp::WidenA(ty) => {
            return format!(
                "Sign-extend the low half of the accumulator to {} bits.",
                bits_of(ty)
            );
        }
        XOp::SignFill(ty) => {
            return format!(
                "Fill d with copies of the sign bit of the {}-bit accumulator (sets up a signed divide).",
                bits_of(ty)
            );
        }
        XOp::DoubleShift(true) => {
            "Shift the destination left, filling from the top bits of the source."
        }
        XOp::DoubleShift(false) => {
            "Shift the destination right, filling from the low bits of the source."
        }
        XOp::RotateCarry(true) => "Rotate left through the carry flag.",
        XOp::RotateCarry(false) => "Rotate right through the carry flag.",
        XOp::Mulx => "Unsigned multiply d by src into hi:lo without touching the flags.",
        XOp::AddCarry(true) => "Add with carry using only CF (the other flags are kept).",
        XOp::AddCarry(false) => "Add with carry using OF as the carry (the other flags are kept).",
        XOp::Andn => "AND b with the inverted a.",
        XOp::Bextr => {
            "Extract a bit field: start in bits 0-7 and length in bits 8-15 of the control."
        }
        XOp::Bzhi => "Clear the bits of the source from the given index upwards.",
        XOp::Pdep => {
            "Scatter the low bits of the source to the positions of the set bits of the mask."
        }
        XOp::Pext => "Gather the source bits under the set bits of the mask into the low bits.",
        XOp::ShiftX(ShiftKind::Shl) => "Shift left by a register count without touching the flags.",
        XOp::ShiftX(ShiftKind::Shr) => {
            "Logical right shift by a register count without touching the flags."
        }
        XOp::ShiftX(ShiftKind::Sar) => {
            "Arithmetic right shift by a register count without touching the flags."
        }
        XOp::ShiftX(ShiftKind::Rol | ShiftKind::Ror) | XOp::Rorx => {
            "Rotate right by an immediate without touching the flags."
        }
        XOp::Crc32 => "Fold the data into a CRC-32C (Castagnoli) checksum accumulator.",
        XOp::Lahf => "Load SF, ZF, PF and CF into the second byte (AH) of the operand.",
        XOp::Sahf => "Store the second byte (AH) of the operand into SF, ZF, PF and CF.",
        XOp::Xlat => "Table lookup: replace the low byte of a with table[a].",
        XOp::Hint => "Cache hint for the address (no effect here; the address is still evaluated).",
        XOp::Direction(false) => "Clear the direction flag: string instructions walk upwards.",
        XOp::Direction(true) => {
            "Set the direction flag: string instructions in this block walk downwards."
        }
        XOp::CmpxchgPair(Ty::I32) => {
            "Compare d:a with 8 bytes of memory; store c:b if equal, else load them into d:a."
        }
        XOp::CmpxchgPair(_) => {
            "Compare d:a with 16 bytes of memory; store c:b if equal, else load them into d:a."
        }
        XOp::Str(op, rep) => {
            let what = match op {
                StrOp::Movs => "copy an element from [si] to [di]",
                StrOp::Stos => "store a to [di]",
                StrOp::Lods => "load [si] into a",
                StrOp::Cmps => "compare [si] with [di]",
                StrOp::Scas => "compare a with [di]",
            };
            let how = match rep {
                Rep::Once => "once",
                Rep::Count => "c times",
                Rep::Repe => "while equal, at most c times",
                Rep::Repne => "while not equal, at most c times",
            };
            return format!(
                "String instruction ({}): {what}, {how}, advancing the pointers by the element size.",
                if name.starts_with("rep") {
                    "repeated"
                } else {
                    "single"
                }
            );
        }
        XOp::Int => "Software interrupt; only `int 3` (a breakpoint) is meaningful.",
        XOp::Xgetbv => {
            "Read an extended control register into d:a (0 here: no extended state is reported)."
        }
        XOp::Stmxcsr => "Store MXCSR to memory (the default 0x1f80 here).",
        XOp::Ldmxcsr => "Load MXCSR from memory (accepted and ignored).",
        XOp::Rdpid => "Read the processor id (0 here).",
    };
    s.to_owned()
}

fn describe_mask(op: KOp, ty: Ty) -> String {
    let bits = bits_of(ty);
    match op {
        KOp::Mov => format!(
            "Move {bits} mask bits between mask registers, general-purpose registers and memory."
        ),
        KOp::Bin(BinOp::And) => format!("AND two {bits}-bit masks."),
        KOp::Bin(BinOp::Or) => format!("OR two {bits}-bit masks."),
        KOp::Bin(_) => format!("Exclusive OR two {bits}-bit masks."),
        KOp::AndNot => format!("AND the second {bits}-bit mask with the inverted first."),
        KOp::Xnor => format!("Exclusive NOR of two {bits}-bit masks."),
        KOp::Add => format!("Add two {bits}-bit masks as integers."),
        KOp::Not => format!("Invert a {bits}-bit mask."),
        KOp::ShiftLeft => format!("Shift a {bits}-bit mask left by an immediate."),
        KOp::ShiftRight => format!("Shift a {bits}-bit mask right by an immediate."),
        KOp::Test => {
            format!("Set ZF if a AND b is zero and CF if a AND NOT b is zero ({bits} bits).")
        }
        KOp::OrTest => {
            format!("Set ZF if a OR b is all zeros and CF if it is all ones ({bits} bits).")
        }
        KOp::Unpack => format!("Concatenate the low halves of two masks into a {bits}-bit mask."),
    }
}

fn describe_vector(name: &str, op: VOp) -> String {
    match op {
        VOp::Move if name.starts_with("movnt") => {
            "Move a whole vector to or from memory with a non-temporal (cache-bypassing) hint.".into()
        }
        VOp::Move if name.starts_with("mova") || name.starts_with("movdqa") => {
            "Move a whole vector between registers and memory; hardware wants memory aligned to the vector size.".into()
        }
        VOp::Move => "Move a whole vector between registers and memory, with no alignment requirement.".into(),
        VOp::MovScalarInt(bytes) => format!(
            "Move {} bits between a vector's low lane and a general-purpose register or memory.",
            bytes * 8
        ),
        VOp::MovScalarFloat(ty) => format!(
            "Move one {} to or from the low lane; the three-operand form merges into a copy of the first source.",
            if ty == Ty::F32 { "single-precision float" } else { "double-precision float" }
        ),
        VOp::Broadcast(bytes) => format!(
            "Repeat the first {bytes} bytes of the source across the whole destination."
        ),
        VOp::Bin(lane, ty, scalar) => {
            let f = float_lanes(ty, scalar);
            let i = lanes(ty);
            match lane {
                Lane::And => "Bitwise AND of the sources.".into(),
                Lane::Or => "Bitwise OR of the sources.".into(),
                Lane::Xor => "Bitwise exclusive OR of the sources.".into(),
                Lane::AndNot => "Bitwise AND of the second source with the inverted first source.".into(),
                Lane::Add => format!("Add packed {i}, wrapping on overflow."),
                Lane::Sub => format!("Subtract packed {i}, wrapping on overflow."),
                Lane::Mul => format!("Multiply packed {i}, keeping the low half of each product."),
                Lane::CmpEq => format!("Compare packed {i} for equality: all ones where equal (a mask bit per lane into an omr)."),
                Lane::CmpGt => format!("Compare packed signed {i} for greater-than: all ones where true (a mask bit per lane into an omr)."),
                Lane::MinS => format!("Lane-wise minimum of packed signed {i}."),
                Lane::MaxS => format!("Lane-wise maximum of packed signed {i}."),
                Lane::MinU => format!("Lane-wise minimum of packed unsigned {i}."),
                Lane::MaxU => format!("Lane-wise maximum of packed unsigned {i}."),
                Lane::FAdd => format!("Add {f}."),
                Lane::FSub => format!("Subtract {f}."),
                Lane::FMul => format!("Multiply {f}."),
                Lane::FDiv => format!("Divide {f}."),
                Lane::FMin => format!("Minimum of {f} (the second source when either is NaN)."),
                Lane::FMax => format!("Maximum of {f} (the second source when either is NaN)."),
            }
        }
        VOp::Sqrt(ty, scalar) => format!("Square root of {}.", float_lanes(ty, scalar)),
        VOp::Abs(ty) => format!("Absolute value of packed signed {}.", lanes(ty)),
        VOp::Shift(kind, ty) => {
            let what = match kind {
                Shift::Left => "left, filling with zeros",
                Shift::Logical => "right, filling with zeros",
                Shift::Arith => "right, filling with the sign bit",
            };
            format!(
                "Shift packed {} {what}, by an immediate or the low qword of a vector.",
                lanes(ty)
            )
        }
        VOp::Pshufd => "Rearrange the dwords within each 128-bit block as chosen by the immediate's 2-bit fields.".into(),
        VOp::Shufps => "Take two floats per 128-bit block from each source, as chosen by the immediate.".into(),
        VOp::Cvt(c) => match c {
            Cvt::IntToFloat => "Convert packed signed dwords to single-precision floats.".into(),
            Cvt::FloatToInt => "Convert packed single-precision floats to signed dwords, rounding to nearest even (or the `!n/!d/!u/!z` mode).".into(),
            Cvt::FloatToIntTrunc => "Convert packed single-precision floats to signed dwords, truncating toward zero.".into(),
        },
        VOp::Movmsk(bytes) => format!(
            "Collect the sign bit of each {}-byte lane into the low bits of a general-purpose register.",
            bytes
        ),
        VOp::Gather(index, elem) => format!(
            "Load {elem}-byte elements from base + index*scale for each {index}-byte index whose mask lane is set, clearing the mask."
        ),
        VOp::Nop if name == "emms" => "Leave MMX state (no effect here).".into(),
        VOp::Nop => "Clear the upper parts of the vector registers (no effect here: declared registers are locals).".into(),
        VOp::Ext(s) => describe_simd(name, s),
    }
}

fn describe_simd(name: &str, op: SOp) -> String {
    match op {
        SOp::Bin(b, ty) => {
            let i = lanes(ty);
            match b {
                B2::AddSatS => format!("Add packed signed {i}, clamping to the signed range."),
                B2::AddSatU => format!("Add packed unsigned {i}, clamping to the unsigned range."),
                B2::SubSatS => format!("Subtract packed signed {i}, clamping to the signed range."),
                B2::SubSatU => format!("Subtract packed unsigned {i}, clamping at zero."),
                B2::Avg => format!("Average packed unsigned {i}, rounding up."),
                B2::MulHiS => format!("Multiply packed signed {i}, keeping the high half of each product."),
                B2::MulHiU => format!("Multiply packed unsigned {i}, keeping the high half of each product."),
                B2::MulHrs => "Multiply packed signed words as fixed point, rounding and keeping bits 15-30 of each product.".into(),
                B2::Sign => format!("Negate, zero or keep each of the packed {i} by the sign of the second source."),
                B2::Shlv => format!("Shift each of the packed {i} left by its own count."),
                B2::Shrv => format!("Shift each of the packed {i} right (zero-filling) by its own count."),
                B2::Sarv => format!("Shift each of the packed {i} right (sign-filling) by its own count."),
                B2::Rolv => format!("Rotate each of the packed {i} left by its own count."),
                B2::Rorv => format!("Rotate each of the packed {i} right by its own count."),
            }
        }
        SOp::RotImm(left, ty) => format!(
            "Rotate packed {} {} by an immediate.",
            lanes(ty),
            if left { "left" } else { "right" }
        ),
        SOp::Unary(u, ty) => match u {
            U1::Popcnt => format!("Count the set bits in each of the packed {}.", lanes(ty)),
            U1::Lzcnt => format!("Count the leading zero bits in each of the packed {}.", lanes(ty)),
            U1::Conflict => format!("For each of the packed {}, a bit mask of the earlier lanes holding the same value.", lanes(ty)),
            U1::Rcp => format!("Reciprocal of {} (exact here; hardware approximates).", float_lanes(ty, false)),
            U1::Rsqrt => format!("Reciprocal square root of {} (exact here; hardware approximates).", float_lanes(ty, false)),
        },
        SOp::Round(ty, scalar) => format!(
            "Round {} to an integral value with the mode in the immediate.",
            float_lanes(ty, scalar)
        ),
        SOp::Fma(kind, order, ty, scalar) => {
            let (x, y, z) = match order {
                132 => ("dst", "b", "a"),
                213 => ("a", "dst", "b"),
                _ => ("a", "b", "dst"),
            };
            let what = match kind {
                FmaKind::Madd => format!("dst = {x}*{y} + {z}"),
                FmaKind::Msub => format!("dst = {x}*{y} - {z}"),
                FmaKind::Nmadd => format!("dst = -({x}*{y}) + {z}"),
                FmaKind::Nmsub => format!("dst = -({x}*{y}) - {z}"),
                FmaKind::MaddSub => format!("dst = {x}*{y} - {z} in even lanes, + {z} in odd lanes"),
                FmaKind::MsubAdd => format!("dst = {x}*{y} + {z} in even lanes, - {z} in odd lanes"),
            };
            format!("Fused multiply-add with one rounding on {}: {what}.", float_lanes(ty, scalar))
        }
        SOp::Horizontal(sub, sat, ty) => format!(
            "{} adjacent pairs of {} from both sources{}.",
            if sub { "Subtract" } else { "Add" },
            lanes(ty),
            if sat { " with signed saturation" } else { "" }
        ),
        SOp::AddSub(ty) => format!(
            "Subtract in even lanes and add in odd lanes of {}.",
            float_lanes(ty, false)
        ),
        SOp::Pmaddwd => "Multiply signed words and add adjacent products into dwords.".into(),
        SOp::Pmaddubsw => "Multiply unsigned bytes by signed bytes and add adjacent products into saturated words.".into(),
        SOp::Psadbw => "Sum of absolute byte differences over each group of 8 bytes, into a qword.".into(),
        SOp::MulEven(signed) => format!(
            "Multiply the even {} dwords into full 64-bit products.",
            if signed { "signed" } else { "unsigned" }
        ),
        SOp::DotAcc(Ty::I8) => "Dot product of groups of 4 unsigned by signed bytes, added into the dword accumulator.".into(),
        SOp::DotAcc(_) => "Dot product of pairs of signed words, added into the dword accumulator.".into(),
        SOp::Phminposuw => "Find the smallest unsigned word and its index, into the low dword.".into(),
        SOp::Unpack(high, ty) => format!(
            "Interleave the {} {} of each 128-bit block of the two sources.",
            if high { "upper" } else { "lower" },
            if name.starts_with('u') { lanes(if ty == Ty::I32 { Ty::F32 } else { Ty::F64 }) } else { lanes(ty) }
        ),
        SOp::ShufHalf(high) => format!(
            "Rearrange the four {} words of each 128-bit block by the immediate; the rest is copied.",
            if high { "upper" } else { "lower" }
        ),
        SOp::Shufpd => "Pick one double per 128-bit block from each source, as chosen by the immediate.".into(),
        SOp::Palignr => "Concatenate each pair of 128-bit blocks and extract 16 bytes at the immediate byte offset.".into(),
        SOp::Valign(ty) => format!(
            "Concatenate the sources and extract a vector shifted right by the immediate in {}.",
            lanes(ty)
        ),
        SOp::ByteShift(left) => format!(
            "Shift each 128-bit block {} by the immediate number of bytes.",
            if left { "left" } else { "right" }
        ),
        SOp::BlendImm(ty) => format!("Choose each of the {} from either source by the immediate's bits.", lanes(ty)),
        SOp::BlendVar(ty) => format!("Choose each of the {} from either source by the sign bit of the mask vector.", lanes(ty)),
        SOp::BlendMask(ty) => format!("Choose each of the {} from either source by the `&k` mask.", lanes(ty)),
        SOp::Pshufb => "Shuffle bytes within each 128-bit block by the indexes in the second source; a set top bit zeroes.".into(),
        SOp::Perm(Ty::I64) => "Permute qwords across the whole vector by an index vector, or within each 256-bit half by the immediate.".into(),
        SOp::Perm(ty) => format!("Permute {} across the whole vector by an index vector.", lanes(ty)),
        SOp::PermTwo(ty, overwrite_index) => format!(
            "Permute {} from a table of two vectors, overwriting the {}.",
            lanes(ty),
            if overwrite_index { "index operand" } else { "first table operand" }
        ),
        SOp::PermilImm(ty) => format!(
            "Permute {} within each 128-bit block by an immediate or a control vector.",
            lanes(if ty == Ty::I32 { Ty::F32 } else { Ty::F64 })
        ),
        SOp::Perm2x128 => "Build each 128-bit half from any half of either source, or zero, by the immediate.".into(),
        SOp::Shuf128(_) => "Pick 128-bit blocks from both sources by the immediate.".into(),
        SOp::Insert(bytes, _) => format!(
            "Copy the first source and replace the {}-bit block selected by the immediate with the second.",
            bytes * 8
        ),
        SOp::Extract(bytes, _) => format!("Extract the {}-bit block selected by the immediate.", bytes * 8),
        SOp::Pinsr(ty) => format!("Insert one of the {} from a register or memory at the lane in the immediate.", lanes(ty)),
        SOp::Pextr(ty) => format!("Extract one of the {} at the lane in the immediate.", lanes(ty)),
        SOp::Insertps => "Insert a float from the source at a lane, and zero lanes, as the immediate says.".into(),
        SOp::Extractps => "Extract the float at the lane in the immediate as 32 bits.".into(),
        SOp::MovHalf(high) => format!(
            "Load or store the {} 64 bits of a vector, keeping the rest.",
            if high { "upper" } else { "lower" }
        ),
        SOp::Movhlps => "Move the upper 64 bits of the source to the lower 64 bits of the destination.".into(),
        SOp::Movlhps => "Move the lower 64 bits of the source to the upper 64 bits of the destination.".into(),
        SOp::Dup(kind) => match kind {
            DupKind::Low64 => "Duplicate the even doubles.".into(),
            DupKind::OddF32 => "Duplicate the odd single-precision floats.".into(),
            DupKind::EvenF32 => "Duplicate the even single-precision floats.".into(),
        },
        SOp::Extend(from, to, signed) => format!(
            "{} packed {} to {}.",
            if signed { "Sign-extend" } else { "Zero-extend" },
            lanes(from),
            lanes(to)
        ),
        SOp::Narrow(from, to, sat) => format!(
            "Narrow packed {} to {}{}.",
            lanes(from),
            lanes(to),
            match sat {
                Sat::Wrap => ", truncating",
                Sat::Signed => " with signed saturation",
                Sat::Unsigned => " with unsigned saturation",
            }
        ),
        SOp::Pack(from, unsigned) => format!(
            "Narrow the {} of both sources to half width with {} saturation.",
            lanes(from),
            if unsigned { "unsigned" } else { "signed" }
        ),
        SOp::Cvt(conv) => match conv {
            Conv::IntToFloat(from, to, unsigned) => format!(
                "Convert packed {} {} to {}.",
                if unsigned { "unsigned" } else { "signed" },
                lanes(from),
                lanes(to)
            ),
            Conv::FloatToInt(from, to, trunc, unsigned) => format!(
                "Convert packed {} to {} {}, {}.",
                lanes(from),
                if unsigned { "unsigned" } else { "signed" },
                lanes(to),
                if trunc { "truncating" } else { "rounding to nearest even" }
            ),
            Conv::FloatToFloat(from, to) | Conv::ScalarFloat(from, to) => format!(
                "Convert {} to {}.",
                lanes(from),
                lanes(to)
            ),
            Conv::GprToScalar(to, unsigned) => format!(
                "Convert a {} integer to the low {} lane.",
                if unsigned { "unsigned" } else { "signed" },
                lanes(to).trim_end_matches('s')
            ),
            Conv::ScalarToGpr(from, trunc, unsigned) => format!(
                "Convert the low {} lane to a {} integer, {}.",
                lanes(from).trim_end_matches('s'),
                if unsigned { "unsigned" } else { "signed" },
                if trunc { "truncating" } else { "rounding to nearest even" }
            ),
        },
        SOp::FCmp(ty, scalar) => format!(
            "Compare {} with the predicate in the immediate: all ones where true (a mask bit per lane into an omr).",
            float_lanes(ty, scalar)
        ),
        SOp::IntCmp(ty, unsigned, _) => format!(
            "Compare packed {} {} into a mask register, with the predicate in the immediate or the name.",
            if unsigned { "unsigned" } else { "signed" },
            lanes(ty)
        ),
        SOp::Comis(ty) => format!(
            "Compare the low {} lanes and set ZF, PF and CF.",
            lanes(ty).trim_end_matches('s')
        ),
        SOp::Ptest => "Set ZF if a AND b is zero and CF if b AND NOT a is zero.".into(),
        SOp::Testm(ty, not) => format!(
            "Set a mask bit for each of the {} where a AND b is {}.",
            lanes(ty),
            if not { "zero" } else { "non-zero" }
        ),
        SOp::MaskToVec(ty) => format!("Expand each mask bit to all ones or zeros in the {}.", lanes(ty)),
        SOp::VecToMask(ty) => format!("Collect the sign bit of each of the {} into a mask register.", lanes(ty)),
        SOp::BroadcastMask(ty) => format!("Repeat the low bits of a mask register into every one of the {}.", lanes(ty)),
        SOp::Compress(ty) => format!("Pack the {} selected by the `&k` mask contiguously into the destination.", lanes(ty)),
        SOp::Expand(ty) => format!("Spread contiguous {} from the source to the lanes selected by the `&k` mask.", lanes(ty)),
        SOp::Ternlog(_) => "Any three-input bitwise function, given as an 8-bit truth table in the immediate.".into(),
        SOp::Dp(ty) => format!("Dot product of {} with lanes selected by the immediate.", float_lanes(ty, false)),
        SOp::Pclmul(_) => "Carry-less multiply of two qwords (selected by the immediate or the name) into 128 bits.".into(),
        SOp::Aes(aes) => match aes {
            AesOp::Enc => "One AES encryption round on each 128-bit block.".into(),
            AesOp::EncLast => "The last AES encryption round (no MixColumns).".into(),
            AesOp::Dec => "One AES decryption round on each 128-bit block.".into(),
            AesOp::DecLast => "The last AES decryption round.".into(),
            AesOp::Imc => "Apply inverse MixColumns to a round key (for decryption).".into(),
            AesOp::KeygenAssist => "Help expand an AES key: S-box and rotate words, XOR the round constant in the immediate.".into(),
        },
        SOp::Half(false) => "Convert packed half-precision floats to single precision.".into(),
        SOp::Half(true) => "Convert packed single-precision floats to half precision with the immediate's rounding mode.".into(),
        SOp::Sha(sha) => match sha {
            ShaOp::Sha1Rnds4 => "Four SHA-1 rounds, with the round function chosen by the immediate.",
            ShaOp::Sha1Nexte => "Compute the SHA-1 state variable E after four rounds.",
            ShaOp::Sha1Msg1 => "First step of the SHA-1 message schedule.",
            ShaOp::Sha1Msg2 => "Final step of the SHA-1 message schedule.",
            ShaOp::Sha256Rnds2 => "Two SHA-256 rounds; the message plus constants operand is the implicit xmm0.",
            ShaOp::Sha256Msg1 => "First step of the SHA-256 message schedule.",
            ShaOp::Sha256Msg2 => "Final step of the SHA-256 message schedule.",
        }
        .into(),
        SOp::Gf(GfOp::Mul) => "Multiply bytes in GF(2^8) modulo the AES polynomial.".into(),
        SOp::Gf(GfOp::Affine(false)) => "Affine transform of each byte by an 8x8 bit matrix, XOR the immediate.".into(),
        SOp::Gf(GfOp::Affine(true)) => "Affine transform of each byte's GF(2^8) inverse, XOR the immediate.".into(),
        SOp::Scatter(index, elem) => format!(
            "Store {elem}-byte elements to base + index*scale for each {index}-byte index selected by the `&k` mask."
        ),
        SOp::MaskMov(ty) => format!(
            "Load or store the {} whose mask lane has its sign bit set; other lanes load as zero.",
            lanes(ty)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_mnemonic_is_accepted_and_described() {
        let table = instructions();
        assert!(table.len() > 900, "{}", table.len());
        for i in table {
            assert!(is_supported(&i.mnemonic), "{}", i.mnemonic);
            assert!(
                !i.forms.is_empty() && !i.description.is_empty(),
                "{}",
                i.mnemonic
            );
            assert!(
                i.forms.iter().all(|f| f.starts_with(&i.mnemonic)),
                "{:?}",
                i.forms
            );
            assert_eq!(instruction(&i.mnemonic), Some(i));
        }
        let vpaddd = instruction("vpaddd").unwrap();
        assert_eq!(vpaddd.forms[0], "vpaddd dst: vec, a: vec, b: vec/mem");
        assert_eq!(vpaddd.feature, "AVX2");
        assert_eq!(instruction("paddd").unwrap().feature, "SSE2");
        // AVX-only spellings without the `v` resolve to the listed one.
        assert_eq!(instruction("permd").unwrap().mnemonic, "vpermd");
        assert_eq!(instruction("lock_add").unwrap().mnemonic, "lock_add");
        assert!(instruction("fsin").is_none());
    }

    /// Every spelling the lookups accept, built from the string literals of the `asm`
    /// sources (alone and joined with short suffixes), must be in the table.
    #[test]
    fn the_table_covers_every_accepted_spelling() {
        let sources = [
            include_str!("../asm.rs"),
            include_str!("scalar.rs"),
            include_str!("mask.rs"),
            include_str!("vec.rs"),
            include_str!("simd.rs"),
            include_str!("simd/ext.rs"),
        ];
        let mut literals = std::collections::BTreeSet::new();
        for source in sources {
            for (i, part) in source.split('"').enumerate() {
                if i % 2 == 1
                    && !part.is_empty()
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
                {
                    literals.insert(part.to_owned());
                }
            }
        }
        let letters = ["b", "w", "d", "q"];
        let mut tails: Vec<String> = vec![String::new()];
        tails.extend(literals.iter().filter(|l| l.len() <= 3).cloned());
        for a in letters {
            tails.push(a.to_owned());
            tails.push(format!("{a}2m"));
            for b in letters {
                tails.push(format!("{a}{b}"));
            }
        }
        let table = instructions();
        let listed = |n: &str| {
            table
                .binary_search_by(|i| i.mnemonic.as_str().cmp(n))
                .is_ok()
        };
        let mut missing = Vec::new();
        for head in &literals {
            for tail in &tails {
                let name = format!("{head}{tail}");
                let spelled = listed(&name)
                    || listed(&format!("v{name}"))
                    || name.strip_prefix('v').is_some_and(listed)
                    // `lock_` on any core instruction, and `movzx`/`movsx` with any suffix.
                    || name.starts_with("lock_")
                    || (name.starts_with("movzx") || name.starts_with("movsx"));
                if is_supported(&name) && !spelled {
                    missing.push(name);
                }
            }
        }
        assert!(missing.is_empty(), "accepted but not listed: {missing:?}");
    }

    #[test]
    fn inline_declarations_take_the_class_the_lowering_gives_them() {
        assert_eq!(inline_register_class("add", 16, 0), Some("gpr"));
        assert_eq!(inline_register_class("vpaddd", 32, 0), Some("vec"));
        assert_eq!(inline_register_class("pmovmskb", 16, 0), Some("gpr"));
        assert_eq!(inline_register_class("vpcmpeqd", 64, 0), Some("omr"));
        assert_eq!(inline_register_class("kandw", 16, 1), Some("omr"));
        assert!(register_classes().iter().any(|c| c.name == "kmask"));
    }
}
