//! C ABI check for the stdlib's hand-written bindings (`tests/abi/manifest.txt`).
//!
//! For one target, the manifest's entries become a C program (values from the real headers:
//! `sizeof`, `_Alignof`, `offsetof`, signedness, constants) and a Jai program (the same values
//! from the stdlib declarations: `size_of`, `align_of`, member offsets from `type_info`). Every
//! value is labelled with the same key, so the two outputs compare line by line.
//!
//! The host check builds and runs both. The cross check (`JAIC_ABI_CROSS`) reads the C values
//! from LLVM IR (`clang -target ... -S -emit-llvm`) and the Jai values from a `#run` under
//! `jaic check -target ...`, so neither program runs on the target.
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// Size, alignment and field offsets.
    Struct,
    /// Size and alignment.
    Type,
    /// Size and signedness.
    Int,
    /// An integer constant.
    Const,
}

struct Entry {
    kind: Kind,
    jai: String,
    c: String,
    /// (Jai field path, C field path).
    fields: Vec<(String, String)>,
}

/// What the manifest declares for one OS.
pub struct Manifest {
    c_prelude: Vec<String>,
    jai_prelude: Vec<String>,
    entries: Vec<Entry>,
}

/// One value both programs print: `size stat_t`, `offset stat_t.st_mode`, `const O_RDONLY`...
struct Check {
    key: String,
    c_expr: String,
    jai_expr: String,
}

fn split_rename(item: &str) -> (String, String) {
    match item.split_once('=') {
        Some((jai, c)) => (jai.trim().to_string(), c.trim().to_string()),
        None => (item.to_string(), item.to_string()),
    }
}

/// The entries of `text` that apply to `target` (`linux-x64`, `macos-arm64`...): sections
/// naming its OS or the whole target.
pub fn parse(text: &str, target: &str) -> Result<Manifest, String> {
    let os = target.split('-').next().unwrap_or(target);
    let mut manifest = Manifest {
        c_prelude: Vec::new(),
        jai_prelude: Vec::new(),
        entries: Vec::new(),
    };
    let mut active = false;
    for (number, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let error = |what: &str| format!("manifest line {}: {what}: {raw}", number + 1);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(section) = line.strip_prefix('[') {
            let section = section
                .strip_suffix(']')
                .ok_or_else(|| error("unclosed section"))?;
            active = section
                .split_whitespace()
                .any(|name| name == os || name == target);
            continue;
        }
        if !active {
            continue;
        }
        if let Some(text) = line.strip_prefix("c:") {
            manifest.c_prelude.push(text.trim().to_string());
            continue;
        }
        if let Some(text) = line.strip_prefix("jai:") {
            manifest.jai_prelude.push(text.trim().to_string());
            continue;
        }
        let (keyword, rest) = line.split_once(' ').ok_or_else(|| error("no item"))?;
        let kind = match keyword {
            "struct" => Kind::Struct,
            "type" => Kind::Type,
            "int" => Kind::Int,
            "const" => Kind::Const,
            _ => return Err(error("unknown keyword")),
        };
        if kind == Kind::Const {
            for item in rest.split_whitespace() {
                let (jai, c) = split_rename(item);
                manifest.entries.push(Entry {
                    kind,
                    jai,
                    c,
                    fields: Vec::new(),
                });
            }
            continue;
        }
        let (names, fields) = match rest.split_once(" : ") {
            Some((names, fields)) => (names, fields),
            None => (rest, ""),
        };
        if kind != Kind::Struct && !fields.is_empty() {
            return Err(error("only `struct` lists fields"));
        }
        let (jai, c) = match names.split_once('=') {
            Some((jai, c)) => (jai.trim().to_string(), c.trim().to_string()),
            None => (names.trim().to_string(), names.trim().to_string()),
        };
        manifest.entries.push(Entry {
            kind,
            jai,
            c,
            fields: fields.split_whitespace().map(split_rename).collect(),
        });
    }
    if manifest.entries.is_empty() {
        return Err(format!("the manifest has no entries for {target}"));
    }
    Ok(manifest)
}

fn checks(manifest: &Manifest) -> Vec<Check> {
    let mut out = Vec::new();
    let mut add = |key: String, c_expr: String, jai_expr: String| {
        out.push(Check {
            key,
            c_expr,
            jai_expr,
        })
    };
    for e in &manifest.entries {
        let (jai, c) = (&e.jai, &e.c);
        match e.kind {
            Kind::Const => add(
                format!("const {jai}"),
                format!("(long long)({c})"),
                format!("cast,trunc(s64) ({jai})"),
            ),
            Kind::Int => {
                add(
                    format!("size {jai}"),
                    format!("(long long)sizeof({c})"),
                    format!("size_of({jai})"),
                );
                add(
                    format!("signed {jai}"),
                    format!("(long long)(({c})-1 < ({c})0)"),
                    format!("abi_signed(type_info({jai}))"),
                );
            }
            Kind::Struct | Kind::Type => {
                add(
                    format!("size {jai}"),
                    format!("(long long)sizeof({c})"),
                    format!("size_of({jai})"),
                );
                add(
                    format!("align {jai}"),
                    format!("(long long)_Alignof({c})"),
                    format!("align_of({jai})"),
                );
                for (jai_field, c_field) in &e.fields {
                    // `name[]`: a C flexible array member, which has an offset but no size.
                    let flexible = c_field.ends_with("[]");
                    let jai_field = jai_field.trim_end_matches("[]");
                    let c_field = c_field.trim_end_matches("[]");
                    add(
                        format!("offset {jai}.{jai_field}"),
                        format!("(long long)offsetof({c}, {c_field})"),
                        format!("abi_field(type_info({jai}), \"{jai_field}\", false)"),
                    );
                    if !flexible {
                        add(
                            format!("fieldsize {jai}.{jai_field}"),
                            format!("(long long)sizeof((({c} *)0)->{c_field})"),
                            format!("abi_field(type_info({jai}), \"{jai_field}\", true)"),
                        );
                    }
                }
            }
        }
    }
    out
}

/// A C file printing every check (`printf`), or defining them as an array for the IR reader.
fn c_source(manifest: &Manifest, checks: &[Check], as_array: bool) -> String {
    let mut s = String::new();
    for line in &manifest.c_prelude {
        writeln!(s, "{line}").unwrap();
    }
    s.push_str("#include <stddef.h>\n#include <stdio.h>\n");
    if as_array {
        s.push_str("long long abi_values[] = {\n");
        for check in checks {
            writeln!(s, "    {},", check.c_expr).unwrap();
        }
        s.push_str("};\n");
    } else {
        s.push_str("int main(void) {\n");
        for check in checks {
            writeln!(
                s,
                "    printf(\"%s %lld\\n\", \"{}\", {});",
                check.key, check.c_expr
            )
            .unwrap();
        }
        s.push_str("    return 0;\n}\n");
    }
    s
}

/// The Jai side: `abi_report` prints every check, from `main` (built and run on the host) or
/// from a `#run` (cross check under `jaic check`).
fn jai_source(manifest: &Manifest, checks: &[Check], at_compile_time: bool) -> String {
    let mut s = String::from("#import \"Basic\";\n");
    for line in &manifest.jai_prelude {
        writeln!(s, "{line}").unwrap();
    }
    s.push_str(JAI_HELPERS);
    s.push_str("abi_report :: () {\n");
    for check in checks {
        writeln!(
            s,
            "    print(\"% %\\n\", \"{}\", {});",
            check.key, check.jai_expr
        )
        .unwrap();
    }
    s.push_str("}\n");
    if at_compile_time {
        s.push_str("#run abi_report();\nmain :: () {}\n");
    } else {
        s.push_str("main :: () { abi_report(); }\n");
    }
    s
}

/// A field's offset or size. Paths may be dotted and look through `using` and anonymous
/// members (Jai spells C's anonymous unions that way); a missing field prints -1.
const JAI_HELPERS: &str = r#"
abi_field :: (info: *Type_Info, path: string, want_size: bool) -> s64 {
    current := info;
    total := 0;
    start := 0;
    for i: 0..path.count {
        if i < path.count && path[i] != #char "." continue;
        name := string.{i - start, path.data + start};
        found, offset, type := abi_find_member(current, name);
        if !found return -1;
        total += offset;
        current = type;
        start = i + 1;
    }
    return ifx want_size then current.runtime_size else total;
}
abi_find_member :: (info: *Type_Info, name: string) -> bool, s64, *Type_Info {
    if info.type != .STRUCT return false, 0, null;
    s := cast(*Type_Info_Struct) info;
    for s.members {
        if it.flags & .CONSTANT continue;
        if it.name == name return true, it.offset_in_bytes, it.type;
    }
    for s.members {
        if it.flags & .CONSTANT continue;
        if !(it.flags & .USING) && it.name != "" continue;
        found, offset, type := abi_find_member(it.type, name);
        if found return true, it.offset_in_bytes + offset, type;
    }
    return false, 0, null;
}
abi_signed :: (info: *Type_Info) -> s64 {
    if info.type == .ENUM info = (cast(*Type_Info_Enum) info).internal_type;
    if info.type != .INTEGER return -1;
    return ifx (cast(*Type_Info_Integer) info).signed then 1 else 0;
}
"#;

fn parse_values(text: &str) -> BTreeMap<String, i64> {
    text.lines()
        .filter_map(|line| {
            let (key, value) = line.rsplit_once(' ')?;
            Some((key.to_string(), value.trim().parse().ok()?))
        })
        .collect()
}

fn describe(target: &str, key: &str, c: i64, jai: Option<i64>) -> String {
    let jai = jai.map_or("missing".to_string(), |v| v.to_string());
    let (what, name) = key.split_once(' ').unwrap_or((key, ""));
    let subject = match what {
        "offset" | "fieldsize" => {
            let (ty, field) = name.split_once('.').unwrap_or((name, ""));
            let jai = if jai == "-1" {
                "no such field"
            } else {
                &jai
            };
            let what = if what == "offset" {
                "offset"
            } else {
                "size"
            };
            return format!("[{target}] struct {ty} field {field}: {what} C={c} Jai={jai}");
        }
        "const" => format!("constant {name}"),
        "signed" => format!("type {name}: signedness"),
        _ => format!("type {name}: {what}"),
    };
    format!("[{target}] {subject} C={c} Jai={jai}")
}

/// Equal values, or for a constant the same 32-bit pattern: C often spells a 32-bit constant
/// through an unsigned type (`(DWORD)-10`) where the binding uses the signed value.
fn same_bits(key: &str, c: i64, jai: i64) -> bool {
    let is_32_bit = |v: i64| (i64::from(i32::MIN)..=i64::from(u32::MAX)).contains(&v);
    c == jai
        || key.starts_with("const ") && is_32_bit(c) && is_32_bit(jai) && c as u32 == jai as u32
}

/// Every check whose values differ, described for a person.
fn compare(target: &str, checks: &[Check], c: &str, jai: &str) -> Vec<String> {
    let (c, jai) = (parse_values(c), parse_values(jai));
    checks
        .iter()
        .filter_map(|check| {
            let Some(&c_value) = c.get(&check.key) else {
                return Some(format!("[{target}] {}: no value from C", check.key));
            };
            let jai_value = jai.get(&check.key).copied();
            let same = jai_value.is_some_and(|j| same_bits(&check.key, c_value, j));
            (!same).then(|| describe(target, &check.key, c_value, jai_value))
        })
        .collect()
}

fn run(command: &mut Command, what: &str) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|e| format!("{what}: cannot start {command:?}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{what} failed ({}):\n{}{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The C compiler for host builds: `CC`, else `clang` on Windows (MSVC hosts have no `cc`)
/// and `cc` elsewhere.
pub fn host_c_compiler() -> String {
    std::env::var("CC").unwrap_or_else(|_| {
        if cfg!(windows) {
            "clang"
        } else {
            "cc"
        }
        .into()
    })
}

/// Build and run both programs for the host in `dir`; returns the mismatches.
pub fn check_host(
    jaic: &str,
    manifest: &Manifest,
    target: &str,
    dir: &Path,
) -> Result<Vec<String>, String> {
    let checks = checks(manifest);
    let exe = |name: &str| dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    std::fs::write(dir.join("abi.c"), c_source(manifest, &checks, false)).unwrap();
    std::fs::write(dir.join("abi.jai"), jai_source(manifest, &checks, false)).unwrap();
    run(
        Command::new(host_c_compiler())
            .arg("abi.c")
            .arg("-o")
            .arg(exe("abi_c"))
            .current_dir(dir),
        "compiling the C side",
    )?;
    run(
        Command::new(jaic)
            .args(["build", "abi.jai", "-o"])
            .arg(exe("abi_jai"))
            .current_dir(dir),
        "building the Jai side",
    )?;
    let c = run(&mut Command::new(exe("abi_c")), "running the C side")?;
    let jai = run(&mut Command::new(exe("abi_jai")), "running the Jai side")?;
    Ok(compare(target, &checks, &c, &jai))
}

/// One cross target: its name (`macos-x64`), `jaic -target` triple and clang arguments.
pub struct CrossTarget {
    pub name: String,
    pub jaic_triple: String,
    pub clang_args: Vec<String>,
}

/// `os-cpu[:clang args]`, e.g. `macos-x64`, `windows-arm64:-isystem /path/include`.
pub fn cross_target(spec: &str) -> Result<CrossTarget, String> {
    let (name, extra) = spec.split_once(':').unwrap_or((spec, ""));
    let (os, cpu) = name
        .split_once('-')
        .ok_or_else(|| format!("cross target `{name}` is not os-cpu"))?;
    let arch = match cpu {
        "x64" => "x86_64",
        "arm64" => "aarch64",
        _ => return Err(format!("unknown cpu `{cpu}` (x64, arm64)")),
    };
    let (clang_triple, jaic_triple) = match os {
        "linux" => (
            format!("{arch}-linux-gnu"),
            format!("{arch}-unknown-linux-gnu"),
        ),
        "macos" => (
            format!("{arch}-apple-macos11"),
            format!("{arch}-apple-macos"),
        ),
        "windows" => (
            format!("{arch}-w64-mingw32"),
            format!("{arch}-pc-windows-msvc"),
        ),
        _ => return Err(format!("unknown os `{os}` (linux, macos, windows)")),
    };
    let mut clang_args = vec!["-target".to_string(), clang_triple];
    clang_args.extend(extra.split_whitespace().map(String::from));
    // One macOS SDK serves both CPUs; a clang outside Xcode needs to be told where it is.
    if os == "macos"
        && !extra.contains("sysroot")
        && let Ok(sdk) = Command::new("xcrun").arg("--show-sdk-path").output()
    {
        let sdk = String::from_utf8_lossy(&sdk.stdout).trim().to_string();
        if !sdk.is_empty() {
            clang_args.extend(["-isysroot".to_string(), sdk]);
        }
    }
    Ok(CrossTarget {
        name: name.to_string(),
        jaic_triple,
        clang_args,
    })
}

/// The `i64` initialisers of `@abi_values` in LLVM IR, in order.
fn values_from_ir(ir: &str) -> Result<Vec<i64>, String> {
    let line = ir
        .lines()
        .find(|l| l.starts_with("@abi_values ="))
        .ok_or("no @abi_values in the IR")?;
    let start = line.rfind("[i64").ok_or("unexpected @abi_values form")?;
    Ok(line[start..]
        .split("i64 ")
        .skip(1)
        .filter_map(|v| v.split([',', ']']).next()?.trim().parse().ok())
        .collect())
}

/// Check one cross target without running anything on it; returns the mismatches.
pub fn check_cross(
    jaic: &str,
    clang: &str,
    manifest: &Manifest,
    target: &CrossTarget,
    dir: &Path,
) -> Result<Vec<String>, String> {
    let checks = checks(manifest);
    std::fs::write(dir.join("abi.c"), c_source(manifest, &checks, true)).unwrap();
    std::fs::write(dir.join("abi.jai"), jai_source(manifest, &checks, true)).unwrap();
    let ir = run(
        Command::new(clang)
            .args(&target.clang_args)
            .args(["-S", "-emit-llvm", "-o", "-", "abi.c"])
            .current_dir(dir),
        "compiling the C side",
    )?;
    let values = values_from_ir(&ir)?;
    if values.len() != checks.len() {
        return Err(format!(
            "read {} values from the IR, expected {}",
            values.len(),
            checks.len()
        ));
    }
    let c: String = checks
        .iter()
        .zip(values)
        .map(|(check, v)| format!("{} {v}\n", check.key))
        .collect();
    let jai = run(
        Command::new(jaic)
            .args(["check", "abi.jai", "-target", &target.jaic_triple])
            .current_dir(dir),
        "checking the Jai side",
    )?;
    Ok(compare(&target.name, &checks, &c, &jai))
}
