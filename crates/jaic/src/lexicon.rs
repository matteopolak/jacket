//! The words of the language as the editors and linters see them: keywords, built-in types,
//! directives and the languages a here-string terminator can name. One table here replaces
//! the copies in the language server, the linter and the editor grammars; the grammars are
//! generated from `editors/lexicon.json`, which `to_json` writes and a test keeps current.
//!
//! The lexer itself treats keywords as plain identifiers (the parser gives them meaning), so
//! these lists describe, they do not decide how a file is read.
use crate::lexer::LANGUAGE_TAGS;

/// How an editor groups a word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordClass {
    Conditional,
    Repeat,
    /// Statements and statement modifiers, `using` among them.
    Statement,
    /// `struct`, `enum`, `operator`, `inline` and the other declaration words.
    Declaration,
    /// `cast` and `xx`.
    Operator,
    Boolean,
    Constant,
    /// `it`, `it_index` and `context`.
    Variable,
    /// `size_of` and the other compiler functions written like calls.
    Intrinsic,
}

impl WordClass {
    pub const ALL: [WordClass; 9] = [
        WordClass::Conditional,
        WordClass::Repeat,
        WordClass::Statement,
        WordClass::Declaration,
        WordClass::Operator,
        WordClass::Boolean,
        WordClass::Constant,
        WordClass::Variable,
        WordClass::Intrinsic,
    ];

    pub fn name(self) -> &'static str {
        match self {
            WordClass::Conditional => "conditional",
            WordClass::Repeat => "repeat",
            WordClass::Statement => "statement",
            WordClass::Declaration => "declaration",
            WordClass::Operator => "operator",
            WordClass::Boolean => "boolean",
            WordClass::Constant => "constant",
            WordClass::Variable => "variable",
            WordClass::Intrinsic => "intrinsic",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Word {
    pub name: &'static str,
    pub class: WordClass,
    /// Whether the name cannot be used for a declaration the user writes (renames refuse it,
    /// completion offers it as a keyword).
    pub reserved: bool,
}

/// Every keyword-like word, grouped by class.
pub const WORDS: &[Word] = &[
    Word {
        name: "if",
        class: WordClass::Conditional,
        reserved: true,
    },
    Word {
        name: "ifx",
        class: WordClass::Conditional,
        reserved: true,
    },
    Word {
        name: "then",
        class: WordClass::Conditional,
        reserved: true,
    },
    Word {
        name: "else",
        class: WordClass::Conditional,
        reserved: true,
    },
    Word {
        name: "case",
        class: WordClass::Conditional,
        reserved: true,
    },
    Word {
        name: "for",
        class: WordClass::Repeat,
        reserved: true,
    },
    Word {
        name: "while",
        class: WordClass::Repeat,
        reserved: true,
    },
    Word {
        name: "break",
        class: WordClass::Statement,
        reserved: true,
    },
    Word {
        name: "continue",
        class: WordClass::Statement,
        reserved: true,
    },
    Word {
        name: "return",
        class: WordClass::Statement,
        reserved: true,
    },
    Word {
        name: "remove",
        class: WordClass::Statement,
        reserved: true,
    },
    Word {
        name: "defer",
        class: WordClass::Statement,
        reserved: true,
    },
    Word {
        name: "push_context",
        class: WordClass::Statement,
        reserved: true,
    },
    Word {
        name: "using",
        class: WordClass::Statement,
        reserved: true,
    },
    Word {
        name: "struct",
        class: WordClass::Declaration,
        reserved: true,
    },
    Word {
        name: "union",
        class: WordClass::Declaration,
        reserved: true,
    },
    Word {
        name: "enum",
        class: WordClass::Declaration,
        reserved: true,
    },
    Word {
        name: "enum_flags",
        class: WordClass::Declaration,
        reserved: true,
    },
    Word {
        name: "operator",
        class: WordClass::Declaration,
        reserved: true,
    },
    Word {
        name: "inline",
        class: WordClass::Declaration,
        reserved: true,
    },
    Word {
        name: "no_inline",
        class: WordClass::Declaration,
        reserved: true,
    },
    Word {
        name: "interface",
        class: WordClass::Declaration,
        reserved: true,
    },
    Word {
        name: "cast",
        class: WordClass::Operator,
        reserved: true,
    },
    Word {
        name: "xx",
        class: WordClass::Operator,
        reserved: true,
    },
    Word {
        name: "true",
        class: WordClass::Boolean,
        reserved: true,
    },
    Word {
        name: "false",
        class: WordClass::Boolean,
        reserved: true,
    },
    Word {
        name: "null",
        class: WordClass::Constant,
        reserved: true,
    },
    Word {
        name: "it",
        class: WordClass::Variable,
        reserved: false,
    },
    Word {
        name: "it_index",
        class: WordClass::Variable,
        reserved: false,
    },
    Word {
        name: "context",
        class: WordClass::Variable,
        reserved: true,
    },
    Word {
        name: "size_of",
        class: WordClass::Intrinsic,
        reserved: true,
    },
    Word {
        name: "type_of",
        class: WordClass::Intrinsic,
        reserved: true,
    },
    Word {
        name: "code_of",
        class: WordClass::Intrinsic,
        reserved: true,
    },
    Word {
        name: "type_info",
        class: WordClass::Intrinsic,
        reserved: true,
    },
    Word {
        name: "initializer_of",
        class: WordClass::Intrinsic,
        reserved: true,
    },
    Word {
        name: "is_constant",
        class: WordClass::Intrinsic,
        reserved: true,
    },
    Word {
        name: "align_of",
        class: WordClass::Intrinsic,
        reserved: false,
    },
    Word {
        name: "offset_of",
        class: WordClass::Intrinsic,
        reserved: false,
    },
];

/// The words of `class`.
pub fn words_of(class: WordClass) -> impl Iterator<Item = &'static str> {
    WORDS
        .iter()
        .filter(move |w| w.class == class)
        .map(|w| w.name)
}

/// The reserved words, in table order.
pub fn reserved() -> impl Iterator<Item = &'static str> {
    WORDS.iter().filter(|w| w.reserved).map(|w| w.name)
}

/// The class of keyword `name`.
pub fn class_of(name: &str) -> Option<WordClass> {
    WORDS.iter().find(|w| w.name == name).map(|w| w.class)
}

/// Whether `name` is a reserved word.
pub fn is_reserved(name: &str) -> bool {
    WORDS.iter().any(|w| w.reserved && w.name == name)
}

/// The names of the types the language provides.
pub const TYPES: &[&str] = &[
    "bool", "int", "float", "float32", "float64", "string", "void", "s8", "s16", "s32", "s64",
    "s128", "u8", "u16", "u32", "u64", "u128", "Type", "Any", "Code",
];

/// The flags `cast` and `xx` take after a comma: `cast,no_check(T) x`.
pub const CAST_FLAGS: &[&str] = &["no_check", "trunc", "truncate", "force"];

/// What a directive does, for highlighting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectiveRole {
    /// `#import`, `#load` and the library directives.
    Import,
    /// `#if`, `#else` and friends.
    Conditional,
    /// A flag on the declaration that follows (`#c_call`, `#expand`, `#scope_file`).
    Modifier,
    /// Evaluates to a value (`#file`, `#line`, `#this`).
    Value,
    /// Runs or inserts code, or takes a literal (`#run`, `#insert`, `#string`, `#char`).
    Run,
}

impl DirectiveRole {
    pub const ALL: [DirectiveRole; 5] = [
        DirectiveRole::Import,
        DirectiveRole::Conditional,
        DirectiveRole::Modifier,
        DirectiveRole::Value,
        DirectiveRole::Run,
    ];

    pub fn name(self) -> &'static str {
        match self {
            DirectiveRole::Import => "import",
            DirectiveRole::Conditional => "conditional",
            DirectiveRole::Modifier => "modifier",
            DirectiveRole::Value => "value",
            DirectiveRole::Run => "run",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Directive {
    /// The name without the `#`.
    pub name: &'static str,
    pub role: Option<DirectiveRole>,
    /// One line for completion and hover; directives without one are not offered.
    pub about: Option<&'static str>,
}

/// The directives the compiler knows, sorted by name. Any other `#name` is still lexed as a
/// directive; this list only drives completion, hover and the editor grammars.
pub const DIRECTIVES: &[Directive] = &[
    Directive {
        name: "add_context",
        role: Some(DirectiveRole::Modifier),
        about: Some("Add a member to the Context"),
    },
    Directive {
        name: "align",
        role: Some(DirectiveRole::Modifier),
        about: Some("Alignment of a member or variable"),
    },
    Directive {
        name: "as",
        role: Some(DirectiveRole::Modifier),
        about: Some("Allow implicit conversion to this member"),
    },
    Directive {
        name: "asm",
        role: None,
        about: Some("Inline assembly"),
    },
    Directive {
        name: "assert",
        role: None,
        about: Some("Compile-time assertion"),
    },
    Directive {
        name: "bake_arguments",
        role: Some(DirectiveRole::Run),
        about: Some("A procedure with some arguments fixed"),
    },
    Directive {
        name: "bake_constants",
        role: Some(DirectiveRole::Run),
        about: Some("A procedure with polymorphic constants fixed"),
    },
    Directive {
        name: "bytes",
        role: Some(DirectiveRole::Value),
        about: Some("Literal bytes in the instruction stream"),
    },
    Directive {
        name: "c_call",
        role: Some(DirectiveRole::Modifier),
        about: Some("C calling convention"),
    },
    Directive {
        name: "caller_code",
        role: Some(DirectiveRole::Value),
        about: Some("The call site as a Code value"),
    },
    Directive {
        name: "caller_location",
        role: Some(DirectiveRole::Value),
        about: Some("The call site's source location"),
    },
    Directive {
        name: "char",
        role: Some(DirectiveRole::Run),
        about: Some("The value of a one-character string"),
    },
    Directive {
        name: "code",
        role: Some(DirectiveRole::Run),
        about: Some("A piece of code as a Code value"),
    },
    Directive {
        name: "compile_time",
        role: Some(DirectiveRole::Value),
        about: Some("True while running at compile time"),
    },
    Directive {
        name: "compiler",
        role: Some(DirectiveRole::Modifier),
        about: Some("Implemented by the compiler"),
    },
    Directive {
        name: "complete",
        role: Some(DirectiveRole::Conditional),
        about: Some("Require every enum value in an if-case"),
    },
    Directive {
        name: "cpp_method",
        role: Some(DirectiveRole::Modifier),
        about: Some("C++ method calling convention"),
    },
    Directive {
        name: "cpp_return_type_is_non_pod",
        role: Some(DirectiveRole::Modifier),
        about: Some("C++ non-POD return type"),
    },
    Directive {
        name: "deprecated",
        role: Some(DirectiveRole::Modifier),
        about: Some("Warn when used"),
    },
    Directive {
        name: "discard",
        role: Some(DirectiveRole::Modifier),
        about: Some("Ignore the value"),
    },
    Directive {
        name: "distinct",
        role: Some(DirectiveRole::Modifier),
        about: None,
    },
    Directive {
        name: "dump",
        role: None,
        about: Some("Print the procedure's compiled code"),
    },
    Directive {
        name: "else",
        role: Some(DirectiveRole::Conditional),
        about: Some("The other branch of a static if"),
    },
    Directive {
        name: "elsewhere",
        role: Some(DirectiveRole::Modifier),
        about: Some("Defined in another compilation unit"),
    },
    Directive {
        name: "endif",
        role: Some(DirectiveRole::Conditional),
        about: None,
    },
    Directive {
        name: "entry_point",
        role: None,
        about: Some("The procedure the program starts at"),
    },
    Directive {
        name: "exists",
        role: Some(DirectiveRole::Value),
        about: Some("Whether a name is defined"),
    },
    Directive {
        name: "expand",
        role: Some(DirectiveRole::Modifier),
        about: Some("A macro, expanded at the call site"),
    },
    Directive {
        name: "file",
        role: Some(DirectiveRole::Value),
        about: Some("The path of this file"),
    },
    Directive {
        name: "filepath",
        role: Some(DirectiveRole::Value),
        about: Some("The directory of this file"),
    },
    Directive {
        name: "foreign",
        role: Some(DirectiveRole::Modifier),
        about: Some("A procedure from a native library"),
    },
    Directive {
        name: "foreign_library",
        role: Some(DirectiveRole::Import),
        about: Some("A foreign library to link (alias of `#library`)"),
    },
    Directive {
        name: "foreign_system_library",
        role: Some(DirectiveRole::Import),
        about: Some("A system library to link (alias of `#system_library`)"),
    },
    Directive {
        name: "if",
        role: Some(DirectiveRole::Conditional),
        about: Some("Compile-time condition"),
    },
    Directive {
        name: "ifdef",
        role: Some(DirectiveRole::Conditional),
        about: None,
    },
    Directive {
        name: "ifx",
        role: Some(DirectiveRole::Conditional),
        about: Some("Compile-time choice between two expressions"),
    },
    Directive {
        name: "import",
        role: Some(DirectiveRole::Import),
        about: Some("Import a module"),
    },
    Directive {
        name: "insert",
        role: Some(DirectiveRole::Run),
        about: Some("Insert code (a Code value or a string) here"),
    },
    Directive {
        name: "intrinsic",
        role: Some(DirectiveRole::Modifier),
        about: Some("A compiler intrinsic"),
    },
    Directive {
        name: "isa",
        role: Some(DirectiveRole::Modifier),
        about: None,
    },
    Directive {
        name: "library",
        role: Some(DirectiveRole::Import),
        about: Some("Declare a library"),
    },
    Directive {
        name: "line",
        role: Some(DirectiveRole::Value),
        about: Some("The current line number"),
    },
    Directive {
        name: "load",
        role: Some(DirectiveRole::Import),
        about: Some("Load another source file into this scope"),
    },
    Directive {
        name: "location",
        role: Some(DirectiveRole::Value),
        about: Some("The source location of a Code value"),
    },
    Directive {
        name: "modify",
        role: Some(DirectiveRole::Modifier),
        about: Some("Check or change polymorphic parameters"),
    },
    Directive {
        name: "module_parameters",
        role: Some(DirectiveRole::Modifier),
        about: Some("Parameters of this module"),
    },
    Directive {
        name: "must",
        role: Some(DirectiveRole::Modifier),
        about: Some("The return value must be used"),
    },
    Directive {
        name: "no_abc",
        role: Some(DirectiveRole::Modifier),
        about: Some("No array bounds checks"),
    },
    Directive {
        name: "no_alias",
        role: None,
        about: Some("Pointer parameters do not alias"),
    },
    Directive {
        name: "no_aoc",
        role: Some(DirectiveRole::Modifier),
        about: Some("No arithmetic overflow checks"),
    },
    Directive {
        name: "no_call",
        role: Some(DirectiveRole::Modifier),
        about: Some("Not callable at run time"),
    },
    Directive {
        name: "no_context",
        role: Some(DirectiveRole::Modifier),
        about: Some("The procedure takes no context"),
    },
    Directive {
        name: "no_debug",
        role: Some(DirectiveRole::Modifier),
        about: Some("No debug information for this procedure"),
    },
    Directive {
        name: "no_padding",
        role: Some(DirectiveRole::Modifier),
        about: Some("Struct without padding"),
    },
    Directive {
        name: "no_reset",
        role: Some(DirectiveRole::Modifier),
        about: Some("Keep the value compile-time code left in this global"),
    },
    Directive {
        name: "overlay",
        role: Some(DirectiveRole::Modifier),
        about: Some("Place the next member at the offset of another"),
    },
    Directive {
        name: "place",
        role: Some(DirectiveRole::Modifier),
        about: Some("Place the next members at the offset of another"),
    },
    Directive {
        name: "placeholder",
        role: Some(DirectiveRole::Value),
        about: Some("A name a metaprogram will define"),
    },
    Directive {
        name: "poke_name",
        role: Some(DirectiveRole::Modifier),
        about: Some("Make a name visible inside another module"),
    },
    Directive {
        name: "procedure_name",
        role: Some(DirectiveRole::Value),
        about: Some("The name of the enclosing procedure"),
    },
    Directive {
        name: "procedure_of_call",
        role: Some(DirectiveRole::Value),
        about: Some("The procedure a call resolves to"),
    },
    Directive {
        name: "program_export",
        role: Some(DirectiveRole::Modifier),
        about: Some("Export a symbol from the executable"),
    },
    Directive {
        name: "run",
        role: Some(DirectiveRole::Run),
        about: Some("Run code at compile time"),
    },
    Directive {
        name: "runtime_support",
        role: Some(DirectiveRole::Modifier),
        about: Some("Part of the runtime support"),
    },
    Directive {
        name: "scope_export",
        role: Some(DirectiveRole::Modifier),
        about: Some("Following declarations are exported"),
    },
    Directive {
        name: "scope_file",
        role: Some(DirectiveRole::Modifier),
        about: Some("Following declarations are private to this file"),
    },
    Directive {
        name: "scope_module",
        role: Some(DirectiveRole::Modifier),
        about: Some("Following declarations are private to this module"),
    },
    Directive {
        name: "specified",
        role: Some(DirectiveRole::Modifier),
        about: Some("Enum values must be given explicitly"),
    },
    Directive {
        name: "string",
        role: Some(DirectiveRole::Run),
        about: Some("A raw multi-line string literal"),
    },
    Directive {
        name: "symmetric",
        role: Some(DirectiveRole::Modifier),
        about: Some("Operator with either argument order"),
    },
    Directive {
        name: "system_library",
        role: Some(DirectiveRole::Import),
        about: Some("Declare a system library"),
    },
    Directive {
        name: "this",
        role: Some(DirectiveRole::Value),
        about: Some("The enclosing procedure, struct or scope"),
    },
    Directive {
        name: "through",
        role: Some(DirectiveRole::Conditional),
        about: Some("Fall through to the next case"),
    },
    Directive {
        name: "type",
        role: Some(DirectiveRole::Run),
        about: Some("Parse what follows as a type"),
    },
    Directive {
        name: "type_info_no_size_complaint",
        role: Some(DirectiveRole::Modifier),
        about: Some("Allow large type info"),
    },
    Directive {
        name: "type_info_none",
        role: Some(DirectiveRole::Modifier),
        about: Some("Emit no type info for this struct"),
    },
    Directive {
        name: "type_info_procedures_are_void_pointers",
        role: Some(DirectiveRole::Modifier),
        about: Some("Procedure members' type info as void pointers"),
    },
];

/// The directives with a description, as `(name, description)`.
pub fn described_directives() -> impl Iterator<Item = (&'static str, &'static str)> {
    DIRECTIVES.iter().filter_map(|d| Some((d.name, d.about?)))
}

/// The description of directive `name`.
pub fn directive_about(name: &str) -> Option<&'static str> {
    DIRECTIVES.iter().find(|d| d.name == name)?.about
}

/// The directives of `role`.
pub fn directives_of(role: DirectiveRole) -> impl Iterator<Item = &'static str> {
    DIRECTIVES
        .iter()
        .filter(move |d| d.role == Some(role))
        .map(|d| d.name)
}

/// The text of `editors/lexicon.json`.
pub fn to_json() -> String {
    use std::fmt::Write;
    let list = |names: &mut dyn Iterator<Item = &str>| {
        let names: Vec<String> = names.map(|n| format!("{n:?}")).collect();
        format!("[{}]", names.join(", "))
    };
    let mut out = String::from("{\n");
    out.push_str("  \"words\": {\n");
    for (at, class) in WordClass::ALL.iter().enumerate() {
        let comma = if at + 1 < WordClass::ALL.len() {
            ","
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "    {:?}: {}{comma}",
            class.name(),
            list(&mut words_of(*class))
        );
    }
    out.push_str("  },\n");
    let _ = writeln!(out, "  \"reserved\": {},", list(&mut reserved()));
    let _ = writeln!(out, "  \"types\": {},", list(&mut TYPES.iter().copied()));
    let _ = writeln!(
        out,
        "  \"cast_flags\": {},",
        list(&mut CAST_FLAGS.iter().copied())
    );
    out.push_str("  \"directives\": {\n");
    for role in DirectiveRole::ALL {
        let _ = writeln!(
            out,
            "    {:?}: {},",
            role.name(),
            list(&mut directives_of(role))
        );
    }
    let _ = writeln!(
        out,
        "    \"other\": {}",
        list(
            &mut DIRECTIVES
                .iter()
                .filter(|d| d.role.is_none())
                .map(|d| d.name)
        )
    );
    out.push_str("  },\n");
    let _ = writeln!(
        out,
        "  \"here_string_languages\": {}",
        list(&mut LANGUAGE_TAGS.iter().copied())
    );
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editors_lexicon_json_is_current() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/lexicon.json");
        if std::env::var_os("JAI_BLESS").is_some() {
            std::fs::write(&path, to_json()).unwrap();
        }
        assert!(
            std::fs::read_to_string(&path).is_ok_and(|current| current == to_json()),
            "editors/lexicon.json is stale: run `JAI_BLESS=1 cargo test -p jaic --lib lexicon`, then `node editors/vscode/scripts/build-grammar.ts`"
        );
    }

    #[test]
    fn directives_are_sorted_and_unique() {
        assert!(DIRECTIVES.windows(2).all(|w| w[0].name < w[1].name));
    }

    #[test]
    fn words_are_unique() {
        let mut names: Vec<_> = WORDS.iter().map(|w| w.name).collect();
        names.sort_unstable();
        assert!(names.windows(2).all(|w| w[0] != w[1]));
    }
}
