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

    // -- found while migrating call sites that built their message elsewhere ----------
    LexIntegerTooBig = "lex-integer-too-big", Error,
        "an integer literal larger than 64 bits";
    ParseExpectedToken = "parse-expected-token", Error,
        "the parser found a token that cannot continue the construct it is in";
    CompileTimeTrap = "compile-time-trap", Error,
        "compile-time code, or a read of compile-time memory, failed";
    CallIntrinsicWidth = "call-intrinsic-width", Error,
        "an intrinsic whose `width` is not a compile-time 16, 32 or 64";
    LocalNotAccessible = "local-not-accessible", Error,
        "a local used where only its runtime value would exist: in a compile-time expression or an inner procedure";
    TooManyInstances = "too-many-instances", Error,
        "a polymorphic procedure or struct instantiated more times than the compiler allows";
    FormatMissingArgument = "format-missing-argument", Warning,
        "a format string with more `%` than arguments";
    FormatExtraArgument = "format-extra-argument", Warning,
        "a format string given arguments that no `%` prints";
    AssignTargetInvalid = "assign-target-invalid", Error,
        "an assignment to something that is not a variable, member, element or dereference";
    RemoveUnsupported = "remove-unsupported", Error,
        "`remove` in a loop over something that cannot remove elements";
    MetaprogramReport = "metaprogram-report", Error,
        "an error a metaprogram reported itself";
    HostImportUnsupported = "host-import-unsupported", Error,
        "a compile-time value that cannot be moved into the host interpreter";

    ParseKeywordDeclared = "parse-keyword-declared", Error,
        "a keyword used as the name of a declaration";

    // -- the tools around the compiler -------------------------------------------------
    CliUsage = "cli-usage", Error,
        "a command-line mistake: an unknown flag, a missing value, or flags that contradict each other";
    CliFailure = "cli-failure", Error,
        "a tool could not do what it was asked, for a reason outside the program's code";
    MemoryLimit = "memory-limit", Error,
        "the process hit the memory limit set for it";
    InternalError = "internal-error", Error,
        "a bug in the compiler itself";

    // -- the playground ------------------------------------------------------------------
    PlaygroundInput = "playground-input", Error,
        "the playground was given input it cannot run";

    // -- build -----------------------------------------------------------------------------------
    ApiCompilerOperation = "api-compiler-operation", Error,
        "compiler operation needs a `<name>` record";
    ApiCompilerOperationFinds = "api-compiler-operation-finds", Error,
        "compiler operation finds more than one `<name>` record";
    ApiIncompatibleSignatureCompiler = "api-incompatible-signature-compiler", Error,
        "incompatible signature for compiler operation <…>";
    ApiIncompatibleRecordsCompiler = "api-incompatible-records-compiler", Error,
        "incompatible records for compiler operation <…>: <…>";

    // -- lexer -----------------------------------------------------------------------------------
    LexNoteEndsFirst = "lex-note-ends-first", Error,
        "a note ends at the first whitespace, so it cannot be quoted";
    LexUnexpectedCharacter = "lex-unexpected-character", Error,
        "unexpected character `<…>`";
    LexUnterminatedBlockComment = "lex-unterminated-block-comment", Error,
        "unterminated block comment";
    LexExpectedDigits = "lex-expected-digits", Error,
        "expected digits after numeric prefix";
    LexHexFloatBit = "lex-hex-float-bit", Error,
        "a hex float bit pattern requires 4, 8 or 16 hex digits";
    LexInvalidFloatLiteral = "lex-invalid-float-literal", Error,
        "invalid float literal";
    LexUnterminatedStringLiteral = "lex-unterminated-string-literal", Error,
        "unterminated string literal";
    LexExpectedThreeDecimal = "lex-expected-three-decimal", Error,
        "expected three decimal digits after \\d";
    LexInvalidUnicodeEscape = "lex-invalid-unicode-escape", Error,
        "invalid unicode escape";
    LexUnknownEscapeSequence = "lex-unknown-escape-sequence", Error,
        "unknown escape sequence '\\<ch>'";
    LexExpectedHexDigits = "lex-expected-hex-digits", Error,
        "expected hex digits in escape";
    LexExpectedStringTerminator = "lex-expected-string-terminator", Error,
        "expected here-string terminator";
    LexUnterminatedString = "lex-unterminated-string", Error,
        "unterminated here-string";

    // -- parser ----------------------------------------------------------------------------------
    ParseUnterminatedAsmBlock = "parse-unterminated-asm-block", Error,
        "unterminated `#asm` block";
    ParseExpectedNameVariables = "parse-expected-name-variables", Error,
        "expected a name, or a variable's member, index or dereference";
    ParseOnlyNameDeclared = "parse-only-name-declared", Error,
        "only a name can be declared, not a member, index or dereference";
    ParseCharNeedsString = "parse-char-needs-string", Error,
        "`#char` needs a string of exactly one character, found <…> bytes";
    ParseExpectedCall = "parse-expected-call", Error,
        "expected a call after the bake directive";
    ParseExpectedDeclaration = "parse-expected-declaration", Error,
        "expected a declaration after `#add_context`";
    ParseExpectedStatementBody = "parse-expected-statement-body", Error,
        "expected a statement body before the next sibling declaration";
    ParseProcedureTypeReturn = "parse-procedure-type-return", Error,
        "a procedure type in a return type must be parenthesised before the body: write `-> ((A, B) -> C) {`";
    ParseCaseOutsideSwitch = "parse-case-outside-switch", Error,
        "`case` outside of a switch";
    ParseExpectedExpressionAssignment = "parse-expected-expression-assignment", Error,
        "expected an expression or an assignment in this `ifx` branch";
    ParseDeferNeedsStatement = "parse-defer-needs-statement", Error,
        "`defer` needs a statement to defer";

    // -- sema ------------------------------------------------------------------------------------
    AsmUnsupported = "asm-unsupported", Error,
        "an `#asm` feature, instruction or operand size jaic does not support";
    CallBakeArguments = "call-bake-arguments", Error,
        "`#bake_arguments` given something it cannot bake";
    TypesOnlyCompared = "types-only-compared", Error,
        "types can only be compared with == and !=";
    ContextualTypeInference = "contextual-type-inference", Error,
        "contextual type inference requires compile-time execution";
    DeclarationNotFound = "declaration-not-found", Error,
        "`<name>` not found";
    NotType = "not-type", Error,
        "`<name>` is not a type";
    NotValidCode = "not-valid-code", Error,
        "<id> is not a valid Code value";
    CompileTimeValueUnreadable = "compile-time-value-unreadable", Error,
        "a compile-time value the compiler cannot turn into data";
    InsertStringsNested = "insert-strings-nested", Error,
        "#insert strings are nested more than <MAX_INSERT_DEPTH> deep";
    DeclarationCountMismatch = "declaration-count-mismatch", Error,
        "<names> <noun> but <…>";
    MultiplePlaceholderDefinitions = "multiple-placeholder-definitions", Error,
        "multiple placeholder definitions must form a procedure overload set";
    CircularDependency = "circular-dependency", Error,
        "a declaration that depends on itself";
    InternalLocalResolved = "internal-local-resolved", Error,
        "internal: local `<name>` resolved as a declaration";
    PlaceholderNeverDefined = "placeholder-never-defined", Error,
        "`<name>` is a #placeholder that was never defined";
    BuiltinProcedureWithout = "builtin-procedure-without", Error,
        "`<name>` is a builtin procedure without a declaration";
    PreloadMissingDeclaration = "preload-missing-declaration", Error,
        "a declaration the preload module is required to provide is missing or is not what it must be";
    EnumLookupFailed = "enum-lookup-failed", Error,
        "a name looked up in an enum that is not an enum, not a type, or has no such member";
    ConstantDeclaration = "constant-declaration", Error,
        "constant declaration needs a value";
    ExpectedProcedureType = "expected-procedure-type", Error,
        "expected a procedure type";
    RunReturns = "run-returns", Error,
        "this #run returns no value, so it cannot be a constant's value; to only run it at compile time, write the #run as a statement";
    ExpressionNoValue = "expression-no-value", Error,
        "this expression has no value, so it cannot be a constant's value";
    ConstantDeclarationRequiresCompile = "constant-declaration-requires-compile", Error,
        "constant declaration requires a compile-time value; use #run to compute it";
    GlobalNeedsType = "global-needs-type", Error,
        "global needs a type or initializer";
    ExpectedType = "expected-type", Error,
        "expected a type, found <…>";
    ExpectedConstantInteger = "expected-constant-integer", Error,
        "expected a constant integer, found <…>";
    ExpectedConstant = "expected-constant", Error,
        "expected a constant, found <…>";
    ConditionMustCompile = "condition-must-compile", Error,
        "#if condition must be a compile-time constant, found <…>";
    LibraryDirective = "library-directive", Error,
        "library directive needs a name";
    LibraryName = "library-name", Error,
        "library name must be a string literal";
    CompileTimeDependency = "compile-time-dependency", Error,
        "compile-time dependency has no declared procedure";
    CyclicCompileTime = "cyclic-compile-time", Error,
        "cyclic compile-time body dependency for `<…>`";
    CompileTimeDependencyDeadlock = "compile-time-dependency-deadlock", Error,
        "compile-time dependency deadlock: body of `<…>` awaits an unresolved #placeholder";
    RuntimeConfigurationInvalid = "runtime-configuration-invalid", Error,
        "a runtime hook, size or configuration value the runtime does not accept";
    BuildNoMainModule = "build-no-main-module", Error,
        "a build step that needs a main module and the program has none";
    BuildStringScopeInvalid = "build-string-scope-invalid", Error,
        "`add_build_string` given a scope from another workspace";
    ProgramNoProcedure = "program-no-procedure", Error,
        "the program has no '<name> :: () { ... }' procedure";
    EnumValueOut = "enum-value-out", Error,
        "enum value <value> is out of range for the enum's type `<current>`";
    EnumMemberDeclared = "enum-member-declared", Error,
        "enum member `<name>` is declared more than once";
    EnumBodyInvalid = "enum-body-invalid", Error,
        "`using` or `#insert` in an enum body that cannot supply members";
    EnumValue = "enum-value", Error,
        "enum value must be an integer constant, found <…>";
    EnumDeclarationOrder = "enum-declaration-order", Error,
        "enum declaration-order dependency: conditional/injected declarations must establish this implicit member's predecessor";
    OnlyAllowedDeclaration = "only-allowed-declaration", Error,
        "`---` is only allowed as a declaration initializer";
    ContextNotAvailable = "context-not-available", Error,
        "`context` is not available here (procedure is #c_call or #no_context; use push_context)";
    ArraySize = "array-size", Error,
        "array size must be a non-negative constant, found <…>";
    InsertScope = "insert-scope", Error,
        "#insert,scope(...) needs a Code value";
    BacktickNames = "backtick-names", Error,
        "backtick names are only valid inside macros";
    CallerCode = "caller-code", Error,
        "#caller_code is only valid as a parameter default";
    DirectiveNotSupported = "directive-not-supported", Error,
        "directive `#<…>` is not supported in this position";
    RunHostUnsupported = "run-host-unsupported", Error,
        "`#run,host` used where the compiler host cannot run it or move its result";
    IsolatedRunHost = "isolated-run-host", Error,
        "an isolated #run,host procedure must take no arguments";
    InsertNeedsString = "insert-needs-string", Error,
        "#insert needs a string or Code, found <…>";
    InsertStatementExpression = "insert-statement-expression", Error,
        "#insert of a statement in expression position";
    InsertStatementBlock = "insert-statement-block", Error,
        "#insert of a statement block in expression position";
    CouldNotParse = "could-not-parse", Error,
        "could not parse inserted expression";
    InterfaceRestriction = "interface-restriction", Error,
        "an interface restriction must name a struct contract";
    MixInlineHoles = "mix-inline-holes", Error,
        "a mix of inline holes and positional arguments";
    InlineHole = "inline-hole", Error,
        "an inline hole must be valid UTF-8 text";
    InlineHoleEmpty = "inline-hole-empty", Error,
        "an inline hole is empty";
    LambdaSignatureMismatch = "lambda-signature-mismatch", Error,
        "a lambda whose parameters do not match the procedure type it is used as";
    ArrayTypeToo = "array-type-too", Error,
        "array type <…> is too large (the limit is <…> bytes)";
    SimdRegisterOperand = "simd-register-operand", Error,
        "a SIMD register operand cannot be loaded as a scalar; extract a lane in #asm";
    ExpressionNoValue2 = "expression-no-value-2", Error,
        "expression has no value";
    ModuleNotValue = "module-not-value", Error,
        "a module is not a value";
    BuiltinProcedure = "builtin-procedure", Error,
        "builtin procedure is not a value";
    LibraryNotValue = "library-not-value", Error,
        "a library is not a value";
    PolymorphicTypePattern = "polymorphic-type-pattern", Error,
        "the polymorphic type `<…>` is a pattern, not a value: its type variables stand for any type";
    ConstantOutOfRange = "constant-out-of-range", Error,
        "a constant that does not fit the integer or float type it is given";
    VoidValue = "void-value", Error,
        "void value used";
    ModifyRejectedArguments = "modify-rejected-arguments", Error,
        "#modify rejected the arguments to <what><text>";
    UnreachableStatements = "unreachable-statements", Warning,
        "statements that follow a `return`, `break` or `continue` and never run";
    TypeNoMember = "type-no-member", Error,
        "type `<shown>` has no member `<name>`";
    NoMember = "no-member", Error,
        "<what> `<shown>` has no member `<name>`";
    RuntimeTrap = "runtime-trap", Error,
        "<prefix>: <message>";
    UsingMap = "using-map", Error,
        "using,map requires an aggregate member target; module mapping is not supported";
    UsingFilterInvalid = "using-filter-invalid", Error,
        "a `using` filter or mapper that is not a compile-time array of strings or procedure";
    AmbiguousMoreOne = "ambiguous-more-one", Error,
        "`<name>` is ambiguous: more than one `using` variable has a member of that name";

    // -- sema/asm --------------------------------------------------------------------------------
    AsmRequiresX8664 = "asm-requires-x86-64", Error,
        "an `#asm` instruction or intrinsic that only exists on x86-64, used for another target";
    AsmRegisterSize = "asm-register-size", Error,
        "an `#asm` operand that is the wrong register size for its instruction";
    AsmSyscallArgumentOperand = "asm-syscall-argument-operand", Error,
        "a `syscall` argument operand must be a register pinned to di, si, d, 10, 8 or 9";
    AsmLeaSource = "asm-lea-source", Error,
        "#asm lea source must be a bracketed address expression";
    AsmInvalidOperandSizes = "asm-invalid-operand-sizes", Error,
        "invalid operand sizes in `<…>`";
    AsmNeedsSourceDestination = "asm-needs-source-destination", Error,
        "`<…>` needs source and destination sizes (for example movzxbw)";
    AsmVectorOperand = "asm-vector-operand", Error,
        "an `#asm` vector operand, size suffix or index that the instruction does not accept";
    AsmSizeSuffix = "asm-size-suffix", Error,
        "the size after `?` must be a type or a number of bits";
    AsmNoInstructionForm = "asm-no-instruction-form", Error,
        "no #asm instruction form with a <bits>-bit operand";
    AsmOperandCount = "asm-operand-count", Error,
        "an `#asm` instruction given the wrong number of operands";
    AsmMaskOperand = "asm-mask-operand", Error,
        "an `#asm` instruction that needs, or cannot take, a mask register";
    AsmOperandForm = "asm-operand-form", Error,
        "an `#asm` instruction whose operands are not in a form it accepts";
    AsmBootstrapReg = "asm-bootstrap-reg", Error,
        "bootstrap __reg must be a type";
    AsmSimdRegisterOperand = "asm-simd-register-operand", Error,
        "a SIMD register operand has no standalone storage or runtime ABI; use #asm and expanded parameters";
    AsmUnknownRegisterClass = "asm-unknown-register-class", Error,
        "unknown #asm register class `<class>`";
    AsmRegisterPin = "asm-register-pin", Error,
        "a register pin is only valid as a separate statement";
    AsmRegisterArgumentNested = "asm-register-argument-nested", Error,
        "#asm register argument is nested too deeply";
    AsmOperandMustVariable = "asm-operand-must-variable", Error,
        "an #asm operand must be a variable";
    AsmUnsupportedConstantOperand = "asm-unsupported-constant-operand", Error,
        "unsupported constant in an #asm operand";
    AsmOperandMustVariableConstant = "asm-operand-must-variable-constant", Error,
        "an #asm operand must be a variable or a constant";
    AsmOperandType = "asm-operand-type", Error,
        "#asm operand of type `<…>` is not a scalar";
    AsmScaleMemoryOperand = "asm-scale-memory-operand", Error,
        "the scale of a memory operand must be a constant";
    AsmMemoryOperandTerm = "asm-memory-operand-term", Error,
        "a memory operand term must be an integer variable or constant";
    AsmImmediate = "asm-immediate", Error,
        "an `#asm` immediate that does not fit or is not allowed there";
    AsmCannotTakeLock = "asm-cannot-take-lock", Error,
        "`<name>` cannot take the lock_ prefix";
    AsmNeedsElementSize = "asm-needs-element-size", Error,
        "`<name>` needs an element size (.b/.w/.d/.q)";
    AsmCacheOperation = "asm-cache-operation", Error,
        "<name> cache operation is not implemented";
    AsmOnlyInt3 = "asm-only-int-3", Error,
        "only 'int 3' is supported (software interrupts have no meaning outside the OS)";
    AsmNeeds8Bit = "asm-needs-8-bit", Error,
        "`<name>` needs an 8-bit control constant";
    AsmExpectedGeneralPurpose = "asm-expected-general-purpose", Error,
        "expected a general-purpose register or memory operand";
    AsmExpectedGeneralPurposeRegister = "asm-expected-general-purpose-register", Error,
        "expected a general-purpose register or memory destination";

    // -- sema/binary -----------------------------------------------------------------------------
    BinarySignednessMismatchConstant = "binary-signedness-mismatch-constant", Error,
        "signedness mismatch: the constant <v> does not fit in `<…>`";
    BinaryOperatorNotDefined = "binary-operator-not-defined", Error,
        "operator `<symbol>` is not defined for bool";
    BinaryOperatorNotDefined2 = "binary-operator-not-defined-2", Error,
        "operator <…> is not defined for <…>";
    BinaryArithmeticOperator = "binary-arithmetic-operator", Error,
        "arithmetic operator <…> is not defined for <…>";
    BinaryArithmeticOverflow = "binary-arithmetic-overflow", Error,
        "`__arithmetic_overflow` is not a procedure";
    BinaryArithmeticOverflowMustDefined = "binary-arithmetic-overflow-must-defined", Error,
        "`__arithmetic_overflow` must be defined in Jai";
    BinaryNegativeShiftAmount = "binary-negative-shift-amount", Error,
        "negative shift amount <b> in constant expression";
    BinaryDivisionZeroConstant = "binary-division-zero-constant", Error,
        "division by zero in constant expression";
    BinaryFloatingLiteral = "binary-floating-literal", Error,
        "this floating literal requires float64 precision (use an explicit cast to round it)";
    BinaryStringsOnlyCompared = "binary-strings-only-compared", Error,
        "strings can only be compared with == and !=";
    BinaryArraysCannotCompared = "binary-arrays-cannot-compared", Error,
        "arrays cannot be compared with `<symbol>`";
    BinaryTypeMismatch = "binary-type-mismatch", Error,
        "type mismatch: `<…>` and `<…>` cannot be combined";

    // -- sema/calls ------------------------------------------------------------------------------
    CallMissingValue = "call-missing-value", Error,
        "missing value";
    CallBakedVariadicPack = "call-baked-variadic-pack", Error,
        "baked variadic pack is too large";
    CallBakedSpreadPointer = "call-baked-spread-pointer", Error,
        "baked spread pointer overflow";
    CallBuiltinNeedsArgument = "call-builtin-needs-argument", Error,
        "builtin needs an argument";
    CallNeedsConstantType = "call-needs-constant-type", Error,
        "`<name>` needs a constant type, but this is a `Type` variable";
    CallMemberNameOffset = "call-member-name-offset", Error,
        "the member name of `offset_of` must be a constant string";
    CallOffsetTakesType = "call-offset-takes-type", Error,
        "`offset_of` takes `(Type, \"member\")` or `Type.member`";
    CallNoMemberFixed = "call-no-member-fixed", Error,
        "`<tn>` has no member `<part>` at a fixed offset";
    CallProcedureNeedsExpression = "call-procedure-needs-expression", Error,
        "#procedure_of_call needs a call expression";
    CallProcedureNeedsProcedure = "call-procedure-needs-procedure", Error,
        "#procedure_of_call needs a procedure call";
    CallNoProcedureMatches = "call-no-procedure-matches", Error,
        "no procedure matches this call";
    CallArgumentType = "call-argument-type", Error,
        "argument of type `<…>` does not match parameter type `<…>`";
    CallRegisterOperands = "call-register-operands", Error,
        "register operands require an expanded procedure";
    CallExpectedAsmRegister = "call-expected-asm-register", Error,
        "expected an #asm register operand, not a scalar or memory value";
    CallLambdaCannotPassed = "call-lambda-cannot-passed", Error,
        "a lambda cannot be passed as <…>";
    CallCannotInfer = "call-cannot-infer", Error,
        "<CANNOT_INFER> <…>";
    CallArgumentNoValue = "call-argument-no-value", Error,
        "the argument has no value";
    CallProcedureNotMatch = "call-procedure-not-match", Error,
        "procedure does not match parameter type <…>";
    CallCannotWithoutContext = "call-cannot-without-context", Error,
        "cannot call `<…>` without a context (use push_context in #c_call code)";
    CallNoParameterNamed = "call-no-parameter-named", Error,
        "no parameter named `<name>`";
    CallArgument = "call-argument", Error,
        "argument <…> supplied more than once";
    CallCannotValueType = "call-cannot-value-type", Error,
        "cannot call a value of type <…>";
    CallNamedArguments = "call-named-arguments", Error,
        "named arguments require a known callable declaration";
    CallJaiVariadicPack = "call-jai-variadic-pack", Error,
        "a Jai variadic pack requires an array-view parameter";
    CallResultMarked = "call-result-marked", Error,
        "the result of `<name>` is marked #must and cannot be discarded";
    CallMustGivenType = "call-must-given-type", Error,
        "`$<name>` must be given a type";
    CallMissingArgumentBaked = "call-missing-argument-baked", Error,
        "missing argument for baked parameter `<name>`";
    CallArgumentMustCompile = "call-argument-must-compile", Error,
        "argument for `$<name>` must be a compile-time constant";
    CallCannotInferParameter = "call-cannot-infer-parameter", Error,
        "cannot infer the parameter types of this procedure";
    CallNoOverloadMatches = "call-no-overload-matches", Error,
        "no overload matches the procedure parameter";
    CallUnequalArrayLengths = "call-unequal-array-lengths", Error,
        "unequal array lengths (wanted <wanted>, given <given>)";
    CallCouldNotInfer = "call-could-not-infer", Error,
        "could not infer polymorphic type `$<name>`";
    CallNotInstance = "call-not-instance", Error,
        "a type that is not an instance of the polymorphic struct a parameter names";
    CallUnknownPolymorphicStruct = "call-unknown-polymorphic-struct", Error,
        "unknown polymorphic struct parameter";
    CallMissingPolymorphicStruct = "call-missing-polymorphic-struct", Error,
        "missing polymorphic struct parameter binding";
    CallInterfaceConstraint = "call-interface-constraint", Error,
        "interface constraint requires a concrete type binding";
    CallNotSatisfyRestriction = "call-not-satisfy-restriction", Error,
        "<…> does not satisfy the restriction `<…>`";
    CallNotOneTypes = "call-not-one-types", Error,
        "<…> is not one of the types the restriction allows";
    CallExpectedPointer = "call-expected-pointer", Error,
        "expected a pointer, found <…>";
    CallExpectedArray = "call-expected-array", Error,
        "expected an array, found <…>";
    CallArrayKindMismatch = "call-array-kind-mismatch", Error,
        "array kind mismatch with <…>";
    CallExpectedProcedure = "call-expected-procedure", Error,
        "expected a procedure, found <…>";
    CallConflictingBindings = "call-conflicting-bindings", Error,
        "<CONFLICT><name>`: <…> and <…>";
    CallNoArchitecturalCycle = "call-no-architectural-cycle", Error,
        "no architectural cycle counter is available on a WebAssembly target";
    CallUnknownIntrinsic = "call-unknown-intrinsic", Error,
        "unknown intrinsic `<other>`";
    CallFloatingIntrinsicSignature = "call-floating-intrinsic-signature", Error,
        "floating intrinsic signature requires one float32/float64 argument and the same result type";
    CallRequiresNoArguments = "call-requires-no-arguments", Error,
        "<name> requires no arguments and one 64-bit integer result";
    CallAtomicOperation = "call-atomic-operation", Error,
        "an atomic operation with an unsupported type or memory ordering";
    CallBlackBox = "call-black-box", Error,
        "`black_box` used with the wrong arguments or a type without a runtime form";
    CallNoOperator = "call-no-operator", Error,
        "an operator that no `operator` overload handles for these operand types";
    CallRegisterOperandsNeedExplicit = "call-register-operands-need-explicit", Error,
        "register operands need an explicit argument";
    CallRegisterOperandsNeedCaller = "call-register-operands-need-caller", Error,
        "register operands need a caller register";
    CallExpectedAsmRegisterOperand = "call-expected-asm-register-operand", Error,
        "expected an #asm register operand";
    CallCodeParameter = "call-code-parameter", Error,
        "Code parameter needs a code argument";
    CallCallerCode = "call-caller-code", Error,
        "#caller_code did not produce Code";
    CallCodeParameterNeedsArgument = "call-code-parameter-needs-argument", Error,
        "Code parameter needs an argument";
    CallMacroNoBody = "call-macro-no-body", Error,
        "macro has no body";
    CallMacroExpansionToo = "call-macro-expansion-too", Error,
        "macro expansion is too deeply nested";
    CallFailed = "call-failed", Error,
        "in call to `<name>`";
    CallInternalSignatureParameter = "call-internal-signature-parameter", Error,
        "internal: signature/parameter mismatch";
    CallRuntimeSpread = "call-runtime-spread", Error,
        "a runtime spread cannot determine a C variadic ABI";
    CallCannotType = "call-cannot-type", Error,
        "cannot call type <name>";
    CallProcedureMatchesPossible = "call-procedure-matches-possible", Error,
        "Procedure call matches <…> possible overloads: `<name>` is ambiguous for these arguments.";
    CallNoOverloadMatchesArguments = "call-no-overload-matches-arguments", Error,
        "no overload of `<name>` matches these arguments";
    CallTypeCannotOperand = "call-type-cannot-operand", Error,
        "a type cannot be an operand of an overloaded operator";
    UnsupportedOperation = "unsupported-operation", Error,
        "cannot <what> <…>";
    CallContextArguments = "call-context-arguments", Error,
        "`,,` context arguments need a context (not in #c_call code)";
    CallContextNoMember = "call-context-no-member", Error,
        "Context has no member `<name>`";
    CallParameterGivenTwice = "call-parameter-given-twice", Error,
        "parameter `<name>` given twice";
    CallVariadicParameterAlready = "call-variadic-parameter-already", Error,
        "variadic parameter already given";

    // -- sema/code_export ------------------------------------------------------------------------
    CodeExportModifyProcedureInvalid = "code-export-modify-procedure-invalid", Error,
        "`compiler_modify_procedure` used on a procedure it cannot change";

    // -- sema/convert ----------------------------------------------------------------------------
    ConvertCannotCast = "convert-cannot-cast", Error,
        "cannot cast <…> to <…>";
    ConvertCannotTo = "convert-cannot-to", Error,
        "cannot convert <…> to <…>";
    ConvertNoOverloadMatches = "convert-no-overload-matches", Error,
        "no overload matches procedure type <…>";
    ConvertLiteralFloat32 = "convert-literal-float32", Warning,
        "this literal is <…> as a float32";

    // -- sema/expr -------------------------------------------------------------------------------
    JaicTypeExpects = "jaic-type-expects", Error,
        "#jaic_type expects a name, as in '#jaic_type long_double'";
    UnknownJaicExtension = "unknown-jaic-extension", Error,
        "unknown jaic extension type `<other>`";
    BlockUsedValue = "block-used-value", Error,
        "a block used as a value must end in an expression";
    TypeMustKnown = "type-must-known", Error,
        "type must be known at compile time";
    DeclaredMoreOnce = "declared-more-once", Error,
        "`<name>` is declared more than once";
    DiscardParameter = "discard-parameter", Error,
        "`<…>` is a #discard parameter and cannot be used in the procedure";
    ExpectedConstantFoundVariable = "expected-constant-found-variable", Error,
        "<what>, but `<name>` is a variable";
    CannotInferType = "cannot-infer-type", Error,
        "cannot infer the type of `.<…>` here";
    CannotInferType2 = "cannot-infer-type-2", Error,
        "cannot infer `.<…>` for type <…>";
    UsedOutsideStruct = "used-outside-struct", Error,
        "#this used outside of a struct or procedure";
    ProcedureTypeParameter = "procedure-type-parameter", Error,
        "procedure type parameter needs a type";
    ProcedureTypeResult = "procedure-type-result", Error,
        "procedure type result needs a type";
    BranchesIfxDifferent = "branches-ifx-different", Error,
        "the branches of `ifx` have different types";
    OperatorMustReturn = "operator-must-return", Error,
        "operator *[] must return a pointer";
    ArrayIndex = "array-index", Error,
        "array index must be an integer, found <…>";
    CannotIndexValue = "cannot-index-value", Error,
        "cannot index a value of type <…>";
    CannotDereferenceValue = "cannot-dereference-value", Error,
        "cannot dereference a value of type <…>";
    CannotTakeAddress = "cannot-take-address", Error,
        "cannot take the address of a number or bool constant: it has no storage (assign it to a variable first)";
    CannotNegate = "cannot-negate", Error,
        "cannot negate <…>";
    CannotBitComplement = "cannot-bit-complement", Error,
        "cannot bit-complement <…>";
    ValueType = "value-type", Error,
        "a value of type <…> cannot be used as a condition";

    // -- sema/modules ----------------------------------------------------------------------------
    ModuleStatementNotAllowed = "module-statement-not-allowed", Error,
        "this statement is not allowed at file scope";
    ModuleAlreadyLoaded = "module-already-loaded", Error,
        "`<path>` is already loaded into this module";
    ModuleParameters = "module-parameters", Error,
        "#module_parameters can only be used at the top level of an imported module, not in a program file";
    ModuleMissingArgumentParameter = "module-missing-argument-parameter", Error,
        "missing argument for module parameter `<…>`";
    ModulePokeName = "module-poke-name", Error,
        "#poke_name takes a module and a name";
    ModulePokeName2 = "module-poke-name-2", Error,
        "#poke_name needs a module";
    ModuleParameterInvalid = "module-parameter-invalid", Error,
        "a module parameter that is unknown, repeated, mistyped or set too late";
    ModuleProgramParameters = "module-program-parameters", Error,
        "PROGRAM parameters may only be supplied by the main program of this workspace";
    ModuleTooManyParameters = "module-too-many-parameters", Error,
        "Too many <label> parameters: Wanted <…>, got <…>.";
    ModuleStandardLibraryInternal = "module-standard-library-internal", Error,
        "standard-library internal sources are private; import a public library module";
    ModuleNot = "module-not", Error,
        "module `<name>` not found";
    ModuleDirectoryNotFile = "module-directory-not-file", Error,
        "`<shown>` is a directory, not a file";
    ModuleParameterMustBind = "module-parameter-must-bind", Error,
        "a module parameter must bind a constant";

    // -- sema/procs ------------------------------------------------------------------------------
    ProcNotProcedureType = "proc-not-procedure-type", Error,
        "<…> is not a procedure type";
    ProcMacro = "proc-macro", Error,
        "macro `<…>` cannot be used as a procedure value";
    ProcMust = "proc-must", Error,
        "`<name>` must not take parameters";
    ProcMustBody = "proc-must-body", Error,
        "`<…>` must have a body";
    ProcNotLibrary = "proc-not-library", Error,
        "`<…>` is not a library";
    ProcNotAllControl = "proc-not-all-control", Warning,
        "not all control paths return a value";
    ProcCVariadicParameter = "proc-c-variadic-parameter", Error,
        "a C variadic parameter must be the last runtime parameter";
    ProcPolymorphicProcedure = "proc-polymorphic-procedure", Error,
        "polymorphic procedure `<…>` needs arguments to determine its types";
    ProcSignatureDependsItself = "proc-signature-depends-itself", Error,
        "signature of `<…>` depends on itself";
    ProcAlreadyDeclaredScope = "proc-already-declared-scope", Error,
        "`<…>` is already declared in this scope";
    ProcParameterNeedsType = "proc-parameter-needs-type", Error,
        "parameter needs a type";
    ProcResultNeedsType = "proc-result-needs-type", Error,
        "result needs a type";
    ProcTwoProceduresName = "proc-two-procedures-name", Error,
        "two procedures with this name have identical parameter types";

    // -- sema/scope ------------------------------------------------------------------------------
    AlreadyDeclaredScope = "already-declared-scope", Error,
        "`<name>` is already declared in this scope";

    // -- sema/stmt -------------------------------------------------------------------------------
    CannotAssignConstant = "cannot-assign-constant", Error,
        "cannot assign into the constant `<…>`";
    CantAssignLarge = "cant-assign-large", Error,
        "Can't assign to a large-value iterator. Perhaps you want to iterate by pointer (for *).";
    CompoundAssignment = "compound-assignment", Error,
        "compound assignment takes one target and one value";
    NamesButValues = "names-but-values", Error,
        "<…> names but <…> values";
    ExpectedValues = "expected-values", Error,
        "expected <…> values, found <…>";
    CannotAssignConstant2 = "cannot-assign-constant-2", Error,
        "cannot assign to constant `<written>`";
    CantAssignImmutable = "cant-assign-immutable", Error,
        "Can't assign to an immutable argument.";
    PushContext = "push-context", Error,
        "push_context needs a Context, found <…>";
    OverlayOnlyAllowed = "overlay-only-allowed", Error,
        "`#overlay` is only allowed in a struct body";
    AddContext = "add-context", Error,
        "#add_context is only allowed at file scope";
    ModuleParametersOnlyAllowed = "module-parameters-only-allowed", Error,
        "#module_parameters is only allowed at file scope";
    StatementsNotSupport = "statements-not-support", Error,
        "'if ==' statements do not support struct or array types at this time.";
    ConditionsListedCase = "conditions-listed-case", Error,
        "Conditions listed by 'case' must be constant.";
    DuplicateCase = "duplicate-case", Error,
        "Duplicate case.";
    CompleteOnlyApplies = "complete-only-applies", Error,
        "#complete only applies to 'if' on enum types.";
    AssignmentNeedsValue = "assignment-needs-value", Error,
        "assignment needs a value";
    DeclarationNeedsType = "declaration-needs-type", Error,
        "declaration needs a type or a value";
    NamesButCall = "names-but-call", Error,
        "<names> names but the call returns <count> value<…>";
    CannotUseUsing = "cannot-use-using", Error,
        "cannot use `using` on <…>";
    UsingNeedsStruct = "using-needs-struct", Error,
        "`using` needs a struct, an enum or a pointer to one; this is `<…>`";
    NoEnclosingLoop = "no-enclosing-loop", Error,
        "<what>: no enclosing loop named `<l>`";
    OutsideLoop = "outside-loop", Error,
        "<what> outside of a loop";
    RemoveNoEnclosing = "remove-no-enclosing", Error,
        "remove: no enclosing loop named `<l>`";
    RemoveOutsideLoop = "remove-outside-loop", Error,
        "remove outside of a for loop";
    RemoveNeedsAddressable = "remove-needs-addressable", Error,
        "remove needs an addressable array";
    LoopsValueIndex = "loops-value-index", Error,
        "the loop's value and index are both named `<it_name>`";
    NeedsNameExpansion = "needs-name-expansion", Error,
        "`for :` needs the name of a `for_expansion` procedure";
    NoVisibleIterating = "no-visible-iterating", Error,
        "no `<macro_name>` is visible for iterating over <ty>";
    CannotIterateOver = "cannot-iterate-over", Error,
        "cannot iterate over a value of type <…>";
    NamedReturnValues = "named-return-values", Error,
        "named return values need a procedure";
    NoReturnValue = "no-return-value", Error,
        "no return value named `<…>`";
    TooManyReturn = "too-many-return", Error,
        "too many return values";
    ReturnValue = "return-value", Error,
        "return value has no name to assign";
    TooManyReturnValues = "too-many-return-values", Error,
        "too many return values (procedure returns <…>)";
    MissingReturnValue = "missing-return-value", Error,
        "missing return value <…> of <…>";
    MissingReturnValue2 = "missing-return-value-2", Error,
        "missing return value";

    // -- sema/structs ----------------------------------------------------------------------------
    StructOnlyCodeExpression = "struct-only-code-expression", Error,
        "only a Code expression has a type";
    StructFixedArray = "struct-fixed-array", Error,
        "fixed array has no member `<name>`";
    StructTypeNoConstant = "struct-type-no-constant", Error,
        "type <…> has no constant member `<name>`";
    StructPlaceNeedsField = "struct-place-needs-field", Error,
        "#place needs a field name";
    StructOverlayNeedsField = "struct-overlay-needs-field", Error,
        "#overlay needs a field name";
    StructUnexpectedExpressionBody = "struct-unexpected-expression-body", Error,
        "unexpected expression in struct body";
    StructCycleFieldTypes = "struct-cycle-field-types", Error,
        "cycle in struct field types";
    StructFieldNeedsType = "struct-field-needs-type", Error,
        "field needs a type";
    StructContextGottenBigger = "struct-context-gotten-bigger", Error,
        "The #Context has gotten bigger than the limit. The maximum size configured in Build_Options is <capacity> bytes, but this declaration would make it <size> bytes.";
    StructCycleFieldLayout = "struct-cycle-field-layout", Error,
        "cycle in struct field layout";
    StructTooLargeLimit = "struct-too-large-limit", Error,
        "struct is too large (the limit is <MAX_SIZE> bytes)";
    StructContainsItselfUse = "struct-contains-itself-use", Error,
        "struct `<name>` contains itself (use a pointer)";
    StructUnionTag = "struct-union-tag", Error,
        "union tag needs a type";
    StructMemberDeclared = "struct-member-declared", Error,
        "member `<name>` is declared more than once";
    StructOverlayNeedsFollowing = "struct-overlay-needs-following", Error,
        "#overlay needs a following stored member";
    StructNoMemberNamed = "struct-no-member-named", Error,
        "no member named `<name>` to anchor on";
    StructNoFieldNamed = "struct-no-field-named", Error,
        "no field named `<name>` in this struct";
    StructOverlayTargetOffset = "struct-overlay-target-offset", Error,
        "overlay target offset is too large";
    StructBehindUsingPointer = "struct-behind-using-pointer", Error,
        "`<name>` is behind a using pointer and has no inline storage offset";
    StructNameBroughtTwice = "struct-name-brought-twice", Error,
        "the name `<name>` is brought in twice";
    StructAlignmentOutRange = "struct-alignment-out-range", Error,
        "alignment <n> is out of range (0 to <MAX_ALIGN>)";
    StructNotEnoughValues = "struct-not-enough-values", Error,
        "not enough values in this struct literal";
    StructTooManyValues = "struct-too-many-values", Error,
        "too many values for `<name>`";
    StructCannotInferType = "struct-cannot-infer-type", Error,
        "cannot infer the type of `.<…>` here; write `Type.<…>`";
    StructCannotUseLiteral = "struct-cannot-use-literal", Error,
        "cannot use a struct literal for type <…>";
    StructFieldTarget = "struct-field-target", Error,
        "this field target cannot be assigned";
    StructLiteralFieldTarget = "struct-literal-field-target", Error,
        "a literal field target must be a member or index path";
    StructCannotInferElement = "struct-cannot-infer-element", Error,
        "cannot infer the element type of an empty array literal";
    StructStringConstant = "struct-string-constant", Error,
        "string constant cannot initialize <…>";
    StructPreloadsSourceCode = "struct-preloads-source-code", Error,
        "Preload's `Source_Code_Location` has no `<name>`";
    StructNotMember = "struct-not-member", Error,
        "`<step>` is not a member of <…>";
    StructNoParameter = "struct-no-parameter", Error,
        "`<name>` has no parameter `<n>`";
    StructTooManyArguments = "struct-too-many-arguments", Error,
        "too many arguments for `<name>`";
    StructMissingArgumentParameter = "struct-missing-argument-parameter", Error,
        "missing argument for parameter `<pname>` of `<name>`";
    StructInvalidUsingTarget = "struct-invalid-using-target", Error,
        "invalid using target";

    // -- sema/typeinfo ---------------------------------------------------------------------------
    TypeInfoTypeInfoStruct = "type-info-type-info-struct", Error,
        "Type_Info_Struct has no initializer field";
    TypeInfoGenericDeclarationMetadata = "type-info-generic-declaration-metadata", Error,
        "generic declaration metadata is unavailable";
    TypeInfoGenericInstanceDeclaration = "type-info-generic-instance-declaration", Error,
        "generic instance declaration metadata is unavailable";
    TypeInfoNoField = "type-info-no-field", Error,
        "<…> has no field `<name>`";
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
