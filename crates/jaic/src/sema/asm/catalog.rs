//! The `#asm` instruction table as editors see it: every mnemonic the lowering accepts,
//! with its operand forms, the CPU feature it needs on hardware and a one-line summary.
//!
//! Nothing here is a second list of instructions. Candidate spellings are expanded from
//! `PATTERNS` and kept only when the lowering's own lookups (`lookup_op`, `lookup_xop`,
//! `lookup_kop`, `lookup_vec`) accept them; forms, features and summaries are derived from
//! the decoded semantics by exhaustive matches, so a new semantic variant does not compile
//! until it is described here. The tests check that every spelling the lookups accept
//! (from the string literals of the `asm` sources) is listed.
mod decode;
mod describe;
mod features;
mod forms;

use self::decode::{Kind, build, decode};
use super::{AsmReg, FEATURES, REG_CLASSES};
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
        .map(std::string::ToString::to_string)
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
        assert!(
            instruction("prefetcht0")
                .unwrap()
                .description
                .contains("Execute the x86")
        );
        assert!(
            instruction("clflush")
                .unwrap()
                .description
                .contains("unavailable")
        );
        assert!(
            instruction("rdtscp")
                .unwrap()
                .description
                .contains("IA32_TSC_AUX")
        );
        assert_eq!(instruction("rdseed").unwrap().feature, "RDSEED");
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
