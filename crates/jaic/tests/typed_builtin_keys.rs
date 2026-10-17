//! The typed builtin keys and values in `jaic::builtin_keys` must match the enums that stdlib
//! declares: the builtins receive the Jai enum's integer value.
use jaic::builtin_keys::*;
use std::path::PathBuf;

fn stdlib(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The members `(normalized name, value)` of the enum declared as `<name> :: enum` or
/// `<name>: enum`, with implicit values numbered from zero.
fn jai_enum(source: &str, name: &str) -> Vec<(String, i64)> {
    let start = [format!("{name} :: enum"), format!("{name}: enum")]
        .iter()
        .find_map(|needle| source.find(needle.as_str()))
        .unwrap_or_else(|| panic!("no enum {name}"));
    let open = start + source[start..].find('{').unwrap() + 1;
    let close = open + source[open..].find('}').unwrap();
    let mut next = 0;
    let mut members = Vec::new();
    for item in source[open..close].split(';') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (member, value) = match item.split_once("::") {
            Some((member, value)) => (member.trim(), value.trim().parse().unwrap()),
            None => (item, next),
        };
        next = value + 1;
        members.push((normalize(member), value));
    }
    members
}

fn normalize(name: &str) -> String {
    name.replace('_', "").to_lowercase()
}

fn declared(members: Vec<(&'static str, i64)>) -> Vec<(String, i64)> {
    members
        .into_iter()
        .map(|(n, v)| (normalize(n), v))
        .collect()
}

fn same(rust: Vec<(&'static str, i64)>, source: &str, name: &str) {
    assert_eq!(declared(rust), jai_enum(source, name), "{name}");
}

/// Every Rust member exists in the Jai enum with the same value (Jai declares more).
fn subset(rust: Vec<(&'static str, i64)>, source: &str, name: &str) {
    let jai = jai_enum(source, name);
    for member in declared(rust) {
        assert!(jai.contains(&member), "{name}: {member:?}");
    }
}

#[test]
fn workspace_option_enums_match_stdlib() {
    let primitives = stdlib("internal/compiler-api/primitives.jai");
    same(WorkspaceText::declared(), &primitives, "Workspace_Text");
    same(WorkspaceFlag::declared(), &primitives, "Workspace_Flag");
    same(WorkspaceNumber::declared(), &primitives, "Workspace_Number");
    same(WorkspaceList::declared(), &primitives, "Workspace_List");
}

#[test]
fn syntax_keys_match_stdlib() {
    let attributes = stdlib("internal/program-print/attributes.jai");
    same(SyntaxKey::declared(), &attributes, "Syntax_Key");
}

#[test]
fn build_option_values_match_stdlib() {
    let options = stdlib("Compiler/contract-records.jai");
    same(OutputType::declared(), &options, "Output_Type");
    same(ArrayBoundsCheck::declared(), &options, "array_bounds_check");
    same(CheckLevel::declared(), &options, "cast_bounds_check");
    same(
        CheckLevel::declared(),
        &options,
        "arithmetic_overflow_check",
    );
    same(Toggle::declared(), &options, "null_pointer_check");
    same(Toggle::declared(), &options, "backtrace_on_crash");
    same(
        RuntimeSupport::declared(),
        &options,
        "runtime_support_definitions",
    );
    same(
        DeadCodeSetting::declared(),
        &options,
        "dead_code_elimination",
    );
    same(DebugInfoSetting::declared(), &options, "emit_debug_info");
    same(
        BitcodeOptimization::declared(),
        &options,
        "Llvm_Bitcode_Optimization_Setting",
    );
    same(
        MachineCodeOptimization::declared(),
        &options,
        "Llvm_Machine_Code_Optimization_Setting",
    );
    let platform = stdlib("prelude/platform.jai");
    subset(OsTag::declared(), &platform, "Operating_System_Tag");
    subset(CpuTag::declared(), &platform, "CPU_Tag");
}
