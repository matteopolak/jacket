//! Directive flag sets of procedure and struct headers, and the check bits of blocks.
use super::{Expr, Ident, LambdaKind};
use std::rc::Rc;

/// The on/off directives of a procedure header, as one set of bits.
///
/// Read them through the `ProcFlags` accessors (`flags.c_call()`); the parser sets them with
/// `ProcFlags::set(ProcBits::C_CALL)`. Adding a flag is one line in the `proc_bits!` list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProcBits(u16);

macro_rules! proc_bits {
    ($($(#[$doc:meta])* $bit:ident $getter:ident = $shift:literal,)*) => {
        impl ProcBits {
            $($(#[$doc])* pub const $bit: Self = Self(1 << $shift);)*
        }

        impl ProcBits {
            /// The directive's spelling after the `#`, e.g. `no_abc`.
            #[must_use]
            pub fn name(self) -> &'static str {
                $(if self == Self::$bit { return stringify!($getter); })*
                ""
            }
        }

        impl ProcFlags {
            $($(#[$doc])* #[must_use] pub fn $getter(&self) -> bool { self.bits.0 & ProcBits::$bit.0 != 0 })*
        }
    };
}

proc_bits! {
    /// `#c_call`: the C calling convention, with no implicit context.
    C_CALL c_call = 0,
    NO_CONTEXT no_context = 1,
    EXPAND expand = 2,
    COMPILER compiler = 3,
    INTRINSIC intrinsic = 4,
    SYMMETRIC symmetric = 5,
    CPP_METHOD cpp_method = 6,
    CPP_RETURN_TYPE_IS_NON_POD cpp_return_type_is_non_pod = 7,
    RUNTIME_SUPPORT runtime_support = 8,
    NO_DEBUG no_debug = 9,
    NO_ABC no_abc = 10,
    NO_AOC no_aoc = 11,
    NO_CALL no_call = 12,
    TYPE_INFO_NONE type_info_none = 13,
}

/// The runtime checks a `#no_abc` / `#no_aoc` directive turns off, as one set of bits.
///
/// A block, a lowering state and a procedure's directives all carry one; `a.with(b)` merges two.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CheckBits(u8);

impl CheckBits {
    pub const NONE: Self = Self(0);

    /// `#no_abc`: no array bounds checks.
    pub const NO_ABC: Self = Self(1);

    /// `#no_aoc`: no arithmetic overflow checks.
    pub const NO_AOC: Self = Self(2);

    #[must_use]
    pub fn no_abc(self) -> bool {
        self.0 & Self::NO_ABC.0 != 0
    }

    #[must_use]
    pub fn no_aoc(self) -> bool {
        self.0 & Self::NO_AOC.0 != 0
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Both sets of bits.
    #[must_use]
    pub fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether `other` turns off a check this set leaves on.
    #[must_use]
    pub fn adds_to(self, outer: Self) -> bool {
        self.0 & !outer.0 != 0
    }

    /// The raw bits, as exported to `Block.block_flags` (`NO_ABC` is 1, `NO_AOC` is 2).
    #[must_use]
    pub fn bits(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProcFlags {
    /// Set on the header of a lambda's desugared procedure literal.
    pub lambda: LambdaKind,
    pub(crate) bits: ProcBits,
    pub elsewhere: Option<Option<Ident>>,
    pub inline: CallHintFlag,
    pub deprecated: Option<Option<Rc<[u8]>>>,
    pub program_export: Option<Option<Rc<[u8]>>>,
    /// The string after `#target_features`: CPU features (`"avx2,bmi2"`) the native code of
    /// this procedure may use on top of the build's.
    pub target_features: Option<Rc<str>>,
    /// The string after `#compiler "name"` / `#intrinsic "name"` selecting the builtin.
    pub builtin_name: Option<Rc<str>>,
    /// Any other `#directive` flag (`#dump`, `#entry_point`, `#no_alias`, ...), kept by name.
    pub other: Vec<Ident>,
}

impl ProcFlags {
    /// The checks this procedure's `#no_abc` / `#no_aoc` turn off.
    #[must_use]
    pub fn checks_off(&self) -> CheckBits {
        let mut checks = CheckBits::NONE;
        if self.no_abc() {
            checks = checks.with(CheckBits::NO_ABC);
        }
        if self.no_aoc() {
            checks = checks.with(CheckBits::NO_AOC);
        }
        checks
    }

    /// Whether the directive `bit` is on.
    #[must_use]
    pub fn has(&self, bit: ProcBits) -> bool {
        self.bits.0 & bit.0 != 0
    }

    /// Turn on the directive `bit`.
    pub fn set(&mut self, bit: ProcBits) {
        self.bits.0 |= bit.0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CallHintFlag {
    #[default]
    None,
    Inline,
    NoInline,
}


/// The on/off directives of a struct header, as one set of bits.
///
/// The bit values are the ones `Code_Struct.flags` reports to metaprograms (bit 1 is `union`,
/// bit 6 is a tagged union; both come from the literal, not from directives).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StructBits(u8);

impl StructBits {
    pub const NO_PADDING: Self = Self(0x04);
    pub const TYPE_INFO_NONE: Self = Self(0x08);
    pub const TYPE_INFO_NO_SIZE_COMPLAINT: Self = Self(0x10);
    pub const TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS: Self = Self(0x20);

    /// The bit for the directive `name` (without the `#`), if it is a flag directive.
    #[must_use]
    pub fn from_directive(name: &str) -> Option<Self> {
        Some(match name {
            "no_padding" => Self::NO_PADDING,
            "type_info_none" => Self::TYPE_INFO_NONE,
            "type_info_no_size_complaint" => Self::TYPE_INFO_NO_SIZE_COMPLAINT,
            "type_info_procedures_are_void_pointers" => {
                Self::TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS
            }
            _ => return None,
        })
    }

    /// The raw bits, as exported to `Code_Struct.flags`.
    #[must_use]
    pub fn bits(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct StructFlags {
    bits: StructBits,
    pub align: Option<Expr>,
}

impl StructFlags {
    /// Whether the directive `bit` is on.
    #[must_use]
    pub fn has(&self, bit: StructBits) -> bool {
        self.bits.0 & bit.0 != 0
    }

    /// Turn on the directive `bit`.
    pub fn set(&mut self, bit: StructBits) {
        self.bits.0 |= bit.0;
    }

    /// The directive bits, as exported to `Code_Struct.flags`.
    #[must_use]
    pub fn bits(&self) -> StructBits {
        self.bits
    }
}
