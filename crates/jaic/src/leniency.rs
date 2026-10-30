//! The places where Jacket accepts code that the official Jai compiler rejects.
//!
//! Each one is a warning of its own kind (a [`Leniency`]), so tools can tell them apart, and each
//! carries [`NOTE`]. `docs/language/leniencies.md` lists them with the portable spelling of each.

/// A construct Jacket accepts on purpose although the official compiler rejects it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Leniency {
    /// An operator overload found through the module of an operand's type.
    OperatorOutOfScope,
    /// A declaration of the program that an import of the program also provides.
    HidesImport,
    /// `for x` over a type whose `for_expansion` only its module's namespace holds.
    NamespacedForExpansion,
    /// `#place a;` in a struct body.
    PlaceDirective,
    /// `Enum.loose` as a type.
    LooseEnum,
    /// `#foreign_library` and `#foreign_system_library`.
    ForeignLibrary,
    /// `#library,link_always "x";` written without a name.
    UnnamedLibrary,
    /// `#must` after a return type.
    MustAfterReturnType,
    /// `#ifx c then a; else b;`
    IfxThenSemicolon,
    /// `*T.{...}`, the address of a struct literal.
    AddressOfLiteral,
    /// A local declaration used above the line that declares it.
    UseBeforeDeclaration,
}

/// What every leniency warning says besides its own message.
pub const NOTE: &str =
    "the official Jai compiler rejects this; Jacket accepts it but may stop in a future release";

impl Leniency {
    /// Every leniency, in the order `docs/language/leniencies.md` lists them.
    pub const ALL: [Leniency; 11] = [
        Leniency::OperatorOutOfScope,
        Leniency::HidesImport,
        Leniency::NamespacedForExpansion,
        Leniency::UseBeforeDeclaration,
        Leniency::PlaceDirective,
        Leniency::LooseEnum,
        Leniency::ForeignLibrary,
        Leniency::UnnamedLibrary,
        Leniency::MustAfterReturnType,
        Leniency::IfxThenSemicolon,
        Leniency::AddressOfLiteral,
    ];

    /// The diagnostic code, as `DiagnosticKind::code` reports it.
    pub fn code(self) -> &'static str {
        match self {
            Leniency::OperatorOutOfScope => "operator-out-of-scope",
            Leniency::HidesImport => "declaration-hides-import",
            Leniency::NamespacedForExpansion => "namespaced-for-expansion",
            Leniency::PlaceDirective => "place-directive",
            Leniency::LooseEnum => "loose-enum",
            Leniency::ForeignLibrary => "foreign-library-directive",
            Leniency::UnnamedLibrary => "unnamed-library-directive",
            Leniency::MustAfterReturnType => "must-after-return-type",
            Leniency::IfxThenSemicolon => "ifx-then-semicolon",
            Leniency::AddressOfLiteral => "address-of-struct-literal",
            Leniency::UseBeforeDeclaration => "use-before-declaration",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_leniency_is_listed_once_with_a_code_of_its_own() {
        let mut codes: Vec<_> = Leniency::ALL.iter().map(|l| l.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), Leniency::ALL.len());
        assert!(
            codes
                .iter()
                .all(|c| c.bytes().all(|b| b.is_ascii_lowercase() || b == b'-'))
        );
    }
}
