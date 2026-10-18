//! Decoding mnemonics through the lowering's own lookups and building the table.
use super::AsmInstruction;
use super::describe::describe;
use super::features::{core_feature, vector_features};
use super::forms::forms;
use crate::ir::Ty;
use crate::sema::asm::mask::{KOp, lookup_kop};
use crate::sema::asm::scalar::{XOp, lookup_xop};
use crate::sema::asm::vec::{VOp, lookup_vec};
use crate::sema::asm::{Op, lookup_op};
use std::collections::BTreeMap;


// Decoding: the lowering's own lookups, in the order `asm_inst` tries them.

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Core(Op),
    Scalar(XOp),
    Mask(KOp, Ty),
    Vector(VOp),
}

pub(super) fn decode(name: &str) -> Option<Kind> {
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
    "{rdtsc,rdtscp,rdrand,rdseed,cpuid,syscall}",
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
    "{mpsadbw,pmadd52luq,pmadd52huq}",
    "pcmp{i,e}str{i,m}",
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

pub(super) fn build() -> Vec<AsmInstruction> {
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
