//! Every diagnostic code, defined once.
//!
//! A code is a stable kebab-case name (`type-mismatch`) that stands for one kind of problem. The
//! name is what users see in `error[type-mismatch]`, what `jaic --explain` takes, what `-A`/`-W`/
//! `-D` flags accept, and what tools match on instead of message text. Wording can change; a name
//! stays. To retire one, move it to [`RETIRED`] so `--explain` and the flags still know it.
//!
//! Each entry is `Variant = "name", Severity, "summary"`, optionally followed by `explain = "..."`
//! (a short paragraph) and `example = "..."` (a few lines of Jai that trigger it). The summary
//! says what the code means in one line; the message at the call site says what happened this time.
use super::Severity;

macro_rules! catalog {
    ($(
        $(#[$meta:meta])*
        $variant:ident = $name:literal, $severity:ident, $summary:literal
        $(, explain = $explain:literal $(, example = $example:literal)?)?;
    )*) => {
        /// What a diagnostic reports. See the module documentation.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Code {
            $($(#[$meta])* $variant,)*
            /// A `jailint` rule, named by the rule (`index_only_loop`). The rules live in
            /// `jailint`, so their names are not in this table.
            Lint(&'static str),
        }

        impl Code {
            /// Every compiler code, in the order of the table.
            pub const ALL: &'static [Code] = &[$(Code::$variant,)*];

            /// The code's name, as printed and as the flags and `--explain` take it.
            pub fn name(self) -> &'static str {
                match self {
                    $(Code::$variant => $name,)*
                    Code::Lint(rule) => rule,
                }
            }

            /// Whether this is an error or a warning when nothing says otherwise.
            pub fn severity(self) -> Severity {
                match self {
                    $(Code::$variant => Severity::$severity,)*
                    Code::Lint(_) => Severity::Warning,
                }
            }

            /// What the code means, in one line.
            pub fn summary(self) -> &'static str {
                match self {
                    $(Code::$variant => $summary,)*
                    Code::Lint(_) => "a lint rule",
                }
            }

            /// A paragraph on what the code means and how to fix it, when one is written.
            pub fn explanation(self) -> Option<&'static str> {
                match self {
                    $(Code::$variant => {
                        let text: Option<&'static str> = None;
                        $(let text = Some($explain);)?
                        text
                    })*
                    Code::Lint(_) => None,
                }
            }

            /// A few lines of Jai that trigger the code, when one is written.
            pub fn example(self) -> Option<&'static str> {
                match self {
                    $(Code::$variant => {
                        let text: Option<&'static str> = None;
                        $($(let text = Some($example);)?)?
                        text
                    })*
                    Code::Lint(_) => None,
                }
            }
        }
    };
}

catalog! {
    // -- names a program resolves ------------------------------------------------------------
    UnknownIdentifier = "unknown-identifier", Error,
        "a name that no scope in reach declares";
    UnknownModuleMember = "unknown-module-member", Error,
        "`Module.name` where the module exports no `name`";
    UseBeforeDeclaration = "use-before-declaration", Error,
        "a constant, procedure or type of a body used above the line that declares it";
    ShadowsBuiltinType = "shadows-builtin-type", Warning,
        "a declaration named like a builtin type";
    UnknownLibrary = "unknown-library", Error,
        "a `#foreign` procedure names a library that no `#library` declares";
    MissingFile = "missing-file", Error,
        "a `#load` or `#import` names a file that does not exist";

    // -- types -------------------------------------------------------------------------------
    TypeMismatch = "type-mismatch", Error,
        "a value whose type does not convert to the type expected there";
    CallMismatch = "call-mismatch", Error,
        "a call that the callee does not accept, with the reason in the message";
    ArgumentCount = "argument-count", Error,
        "a call passes more arguments than the callee takes, or leaves out one it needs";
    AmbiguousProcedureValue = "ambiguous-procedure-value", Error,
        "a procedure used as a value that two or more overloads fit equally";
    InterfaceConstraint = "interface-constraint", Error,
        "a generic argument fails a structural interface restriction";
    ModifyWithoutPolymorphs = "modify-without-polymorphs", Error,
        "a `#modify` block on a procedure or struct with no polymorph variables";
    UnboundGenericParameter = "unbound-generic-parameter", Error,
        "a declaration reflected without concrete generic arguments";
    ImplicitNarrowing = "implicit-narrowing", Error,
        "an `ifx` of integer constants (an `s64`) used where a narrower signed integer is wanted";
    IncompleteSwitch = "incomplete-switch", Error,
        "a `#complete` switch that leaves out members of its enum";
    StaticAssert = "static-assert", Error,
        "a failed `#assert`";

    // -- compile-time execution --------------------------------------------------------------
    Runtime = "runtime-error", Error,
        "the compiled program stopped with a trap while running `main`";
    ExecutionBudgetExhausted = "execution-budget-exhausted", Error,
        "compile-time code used up the steps it is allowed to run";
    CheckingTimeLimit = "checking-time-limit", Warning,
        "the editor stopped checking after its time limit, so the results are partial";
    Unavailable = "unavailable", Error,
        "the program called a foreign procedure this host cannot provide";
    Untrusted = "untrusted", Error,
        "compile-time code was not run because the workspace is untrusted";
    Unknown = "unknown-result", Error,
        "compile-time code could not know its result here";

    // -- the parser --------------------------------------------------------------------------
    NestingLimit = "nesting-limit", Error,
        "the parser gave up past the nesting limit";

    // -- code Jacket accepts although the official compiler rejects it -----------------------
    OperatorOutOfScope = "operator-out-of-scope", Warning,
        "an operator overload found through the module of an operand's type";
    HidesImport = "declaration-hides-import", Warning,
        "a declaration of the program that an import of the program also provides";
    NamespacedForExpansion = "namespaced-for-expansion", Warning,
        "`for x` over a type whose `for_expansion` only its module's namespace holds";
    PlaceDirective = "place-directive", Warning,
        "`#place a;` in a struct body";
    LooseEnum = "loose-enum", Warning,
        "`Enum.loose` as a type";
    ForeignLibrary = "foreign-library-directive", Warning,
        "`#foreign_library` and `#foreign_system_library`";
    UnnamedLibrary = "unnamed-library-directive", Warning,
        "`#library,link_always \"x\";` written without a name";
    MustAfterReturnType = "must-after-return-type", Warning,
        "`#must` after a return type";
    IfxThenSemicolon = "ifx-then-semicolon", Warning,
        "`#ifx c then a; else b;`";
    AddressOfLiteral = "address-of-struct-literal", Warning,
        "`*T.{...}`, the address of a struct literal";
    FloatExponent = "float-exponent-without-fraction", Warning,
        "a decimal float literal with an exponent and no fraction: `1e20`";
    StructAlign = "struct-align-directive", Warning,
        "`#align N` on a struct as a whole";
    ForOverArrayLiteral = "for-over-array-literal", Warning,
        "`for` over an array literal that names no element type";
    ArrayFromStructLiteral = "struct-literal-as-dynamic-array", Warning,
        "an untyped `.{..}` struct literal used as a dynamic array";
    TargetFeatures = "target-features-directive", Warning,
        "`#target_features \"...\"` in a procedure header";
    PushContextStatement = "push-context-without-block", Warning,
        "`push_context ctx statement;` with a statement that is not a block";
    MainReturnsValue = "main-returns-value", Warning,
        "`main :: () -> T`, an entry point that returns a value";
    CallerLocationOutsideDefault = "caller-location-outside-default", Warning,
        "`#caller_location` anywhere but the default value of a parameter";
    MultiValueInCodeArgument = "multi-value-in-code-argument", Warning,
        "`a, b := x, y;` inside a `#code` block passed as a call argument";
    SpreadWithOtherArguments = "spread-with-other-varargs", Warning,
        "a `..xs` spread next to other arguments that fill the same variadic parameter";
    ExistsOnExpression = "exists-on-expression", Warning,
        "`#exists(e)` on an expression that is not a name or a dotted chain of names";
    ProcedureCastToInteger = "procedure-cast-to-integer", Warning,
        "a procedure cast straight to an integer type";
    AddressOfMacroConstant = "address-of-macro-constant", Warning,
        "`*x` of a macro parameter that the call binds to a number or bool constant";
    PositionalLiteralDefaults = "positional-literal-defaults", Warning,
        "`.{1, 2}` that leaves trailing members with defaults to those defaults";

    // -- the playground ------------------------------------------------------------------
    PlaygroundInput = "playground-input", Error,
        "the playground was given input it cannot run";

    // -- transitional: removed once every call site names its code ---------------------------
    Uncoded = "uncoded", Error, "a diagnostic that has no code yet";
}

/// The code-like names of codes that no longer exist. `--explain` and the flags still accept
/// them and say so.
pub const RETIRED: &[&str] = &[];

impl Code {
    /// The code called `name`.
    pub fn parse(name: &str) -> Option<Code> {
        Code::ALL.iter().copied().find(|code| code.name() == name)
    }

    /// The leniency codes, in the order `docs/language/leniencies.md` lists them.
    pub const LENIENCIES: [Code; 24] = [
        Code::OperatorOutOfScope,
        Code::HidesImport,
        Code::NamespacedForExpansion,
        Code::PlaceDirective,
        Code::LooseEnum,
        Code::ForeignLibrary,
        Code::UnnamedLibrary,
        Code::MustAfterReturnType,
        Code::IfxThenSemicolon,
        Code::AddressOfLiteral,
        Code::FloatExponent,
        Code::StructAlign,
        Code::ForOverArrayLiteral,
        Code::ArrayFromStructLiteral,
        Code::TargetFeatures,
        Code::PushContextStatement,
        Code::MainReturnsValue,
        Code::CallerLocationOutsideDefault,
        Code::MultiValueInCodeArgument,
        Code::SpreadWithOtherArguments,
        Code::ExistsOnExpression,
        Code::ProcedureCastToInteger,
        Code::AddressOfMacroConstant,
        Code::PositionalLiteralDefaults,
    ];

    /// Whether this is one of the [`Code::LENIENCIES`].
    pub fn is_leniency(self) -> bool {
        Code::LENIENCIES.contains(&self)
    }

    /// Compile-time code's result is unknown, not wrong: editors salvage and hide it.
    pub fn is_unknown_result(self) -> bool {
        matches!(self, Code::Untrusted | Code::Unavailable | Code::Unknown)
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
