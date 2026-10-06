//! Objective-C selectors in the stdlib name methods that exist and take the arguments they are
//! called with.
//!
//! A selector is a string, so the type-checker cannot see a wrong one: `convertPointToScreen`
//! called with an argument (the method is `convertPointToScreen:`) or a method the class does not
//! have both type-check and then stop the program with "unrecognized selector sent to instance".
//! These tests read every `sel_registerName("...")` in `stdlib/`:
//!
//! - every selector passed to a call must have one `:` per argument after it, and every selector
//!   table entry (`_sel.setTitle_ = sel_registerName("setTitle:")`) must be named after its
//!   selector, on every host;
//! - on macOS, every class or protocol a binding calls a selector on must have that method in the
//!   Objective-C runtime, except the gaps listed in `tests/objc-selectors.txt`.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn stdlib() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap()
}

fn jai_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            jai_files(&path, out);
        } else if path.extension().is_some_and(|x| x == "jai") {
            out.push(path);
        }
    }
}

/// The end of the string literal that starts at `start` (just past its closing quote).
fn skip_string(s: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < s.len() && s[i] != b'"' {
        i += if s[i] == b'\\' {
            2
        } else {
            1
        };
    }
    i + 1
}

/// A literal selector: `sel_registerName(cast(*u8) "name\0")` or `sel_registerName("name")`.
struct Selector {
    name: String,
    /// Offset of `sel_registerName`.
    start: usize,
}

fn selectors(text: &str) -> Vec<Selector> {
    const CALL: &str = "sel_registerName(";
    let s = text.as_bytes();
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(found) = text[from..].find(CALL) {
        let start = from + found;
        from = start + CALL.len();
        let mut i = from;
        let skip_space = |i: &mut usize| {
            while *i < s.len() && s[*i].is_ascii_whitespace() {
                *i += 1;
            }
        };
        skip_space(&mut i);
        if text[i..].starts_with("cast(*u8)") {
            i += "cast(*u8)".len();
            skip_space(&mut i);
        }
        if s.get(i) != Some(&b'"') {
            continue; // Not a literal: a selector built at run time.
        }
        let close = skip_string(s, i);
        let name = text[i + 1..close - 1].trim_end_matches("\\0").to_string();
        let mut j = close;
        skip_space(&mut j);
        if s.get(j) != Some(&b')') {
            continue;
        }
        out.push(Selector {
            name,
            start,
        });
    }
    out
}

/// The `(` of the call whose arguments contain `pos`, if `pos` is an argument of a call.
fn enclosing_call(s: &[u8], pos: usize) -> Option<usize> {
    let mut depth = 0;
    let mut i = pos;
    while i > 0 {
        i -= 1;
        match s[i] {
            b')' => depth += 1,
            b'(' if depth == 0 => return Some(i),
            b'(' => depth -= 1,
            b';' | b'{' | b'}' if depth == 0 => return None,
            _ => {}
        }
    }
    None
}

/// The top-level arguments of the call whose `(` is at `open`, and where each one starts.
fn call_arguments(text: &str, open: usize) -> Vec<(usize, String)> {
    let s = text.as_bytes();
    let mut args = Vec::new();
    let mut depth = 0;
    let mut start = open + 1;
    let mut i = open + 1;
    while i < s.len() {
        match s[i] {
            b'"' => {
                i = skip_string(s, i);
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' if depth == 0 => {
                args.push((start, text[start..i].trim().to_string()));
                break;
            }
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                args.push((start, text[start..i].trim().to_string()));
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    args
}

fn line_of(text: &str, pos: usize) -> usize {
    text[..pos].matches('\n').count() + 1
}

/// Who receives a selector: the methods of a class (`objc_getClass("X")` as the receiver) or of
/// its instances (`self` in a binding).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Receiver {
    class: String,
    instance: bool,
}

/// The struct whose body contains `pos`, if any.
fn enclosing_struct(text: &str, pos: usize) -> Option<String> {
    let s = text.as_bytes();
    let mut depth = 0;
    let mut i = pos;
    while i > 0 {
        i -= 1;
        match s[i] {
            b'}' => depth += 1,
            b'{' if depth > 0 => depth -= 1,
            b'{' => {
                let line_start = text[..i].rfind('\n').map_or(0, |n| n + 1);
                let head = &text[line_start..i];
                if let Some(at) = head.find(":: struct") {
                    return head[..at].split_whitespace().last().map(str::to_string);
                }
            }
            _ => {}
        }
    }
    None
}

/// The type of the `self` parameter of the procedure that contains `pos`.
fn self_type(text: &str, pos: usize) -> Option<String> {
    let header = text[..pos].rfind(":: (")?;
    let params_end = text[header..].find(')')? + header;
    let params = &text[header + 4..params_end];
    let ty = params
        .split(',')
        .find_map(|p| p.trim().strip_prefix("self:"))?
        .trim();
    Some(ty.to_string())
}

/// The Objective-C class `Objective_C.class` uses for a binding struct.
fn runtime_class(name: &str) -> &str {
    match name {
        "LightweightOpenGLView" => "NSOpenGLView",
        "LightweightRenderingView" | "LightweightMetalView" => "NSView",
        other => other,
    }
}

fn receiver(text: &str, call_pos: usize, receiver_arg: &str) -> Option<Receiver> {
    if let Some(at) = receiver_arg.find("objc_getClass(") {
        let rest = &receiver_arg[at..];
        let quote = rest.find('"')?;
        let name = rest[quote + 1..].split(['"', '\\']).next()?;
        return Some(Receiver {
            class: runtime_class(name).to_string(),
            instance: false,
        });
    }
    if receiver_arg.trim_start_matches("cast(*void)").trim() != "self" {
        return None; // A local object: only its arity is checked.
    }
    let declared = self_type(text, call_pos).unwrap_or_default();
    let named = declared
        .strip_prefix('*')
        .map(|t| t.trim())
        .filter(|t| !t.starts_with('$') && *t != "void")
        .map(|t| {
            let t = t.split('(').next().unwrap_or(t);
            t.rsplit('.').next().unwrap_or(t).to_string()
        });
    let class = match named {
        Some(class) => class,
        // `self: id`, `self: *void` or a polymorphic `self`: the struct the method is declared
        // in, else a method every object has.
        None => enclosing_struct(text, call_pos).unwrap_or_else(|| "NSObject".to_string()),
    };
    Some(Receiver {
        class: runtime_class(&class).to_string(),
        instance: true,
    })
}

/// A table entry `x.field = cast(T) sel_registerName("...")`: the field name. The selector must be
/// the value assigned, not an argument of a call on the right.
fn table_field(text: &str, sel_start: usize) -> Option<&str> {
    let line_start = text[..sel_start].rfind('\n').map_or(0, |n| n + 1);
    let head = &text[line_start..sel_start];
    let (target, value) = head.split_once('=')?;
    let target = target.trim();
    if target.ends_with(':') || value.starts_with('=') {
        return None; // A declaration (`x := ...`) or a comparison.
    }
    // What may stand between `=` and the selector: casts and a module prefix.
    let mut rest = value.trim();
    while let Some(after) = rest.strip_prefix("cast(") {
        rest = after.split_once(')')?.1.trim();
    }
    if !rest
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    {
        return None;
    }
    target.rsplit('.').next().filter(|f| !f.is_empty())
}

struct Uses {
    problems: Vec<String>,
    receivers: BTreeSet<(Receiver, String)>,
    calls: usize,
    tables: usize,
}

fn scan() -> Uses {
    let root = stdlib();
    let mut files = Vec::new();
    jai_files(&root, &mut files);
    let mut uses = Uses {
        problems: Vec::new(),
        receivers: BTreeSet::new(),
        calls: 0,
        tables: 0,
    };
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        let s = text.as_bytes();
        let shown = file.strip_prefix(&root).unwrap().display().to_string();
        for sel in selectors(&text) {
            let colons = sel.name.matches(':').count();
            let at = format!("stdlib/{shown}:{}", line_of(&text, sel.start));
            if let Some(field) = table_field(&text, sel.start) {
                uses.tables += 1;
                if field != sel.name.replace(':', "_") {
                    uses.problems.push(format!(
                        "{at}: table entry `{field}` holds selector `{}`",
                        sel.name
                    ));
                }
                continue;
            }
            let Some(open) = enclosing_call(s, sel.start) else {
                continue;
            };
            uses.calls += 1;
            let args = call_arguments(&text, open);
            // The argument the selector is in: the last one that starts before it.
            let Some(index) = args.iter().rposition(|(start, _)| *start <= sel.start) else {
                continue;
            };
            let after = args.len() - index - 1;
            if after != colons {
                uses.problems.push(format!(
                    "{at}: `{}` takes {colons} argument(s) but is sent with {after}",
                    sel.name
                ));
            }
            if index > 0
                && let Some(r) = receiver(&text, sel.start, &args[index - 1].1)
            {
                uses.receivers.insert((r, sel.name.clone()));
            }
        }
    }
    uses
}

#[test]
fn selectors_take_the_arguments_they_are_sent_with() {
    let uses = scan();
    // The stdlib's bindings call over a thousand selectors; finding far fewer means the scan
    // stopped recognising them.
    assert!(
        uses.calls > 1000 && uses.tables > 250,
        "only {} calls and {} table entries found",
        uses.calls,
        uses.tables
    );
    assert!(
        uses.problems.is_empty(),
        "{} selector(s) do not match their use:\n{}",
        uses.problems.len(),
        uses.problems.join("\n")
    );
}

/// A Jai program that prints `missing <class> <instance|class> <selector>` for each method the
/// runtime lacks, and `unknown <class>` for names that are neither a class nor a protocol.
fn check_program(receivers: &BTreeSet<(Receiver, String)>) -> String {
    let mut entries = String::new();
    for (r, selector) in receivers {
        entries.push_str(&format!(
            "    .{{ \"{}\", \"{selector}\", {} }},\n",
            r.class, r.instance
        ));
    }
    format!(
        r#"#import "Basic";
#import "String";
libc :: #system_library "libc";
libobjc :: #system_library "libobjc";
dlopen :: (path: *u8, mode: s32) -> *void #foreign libc;
objc_getClass :: (name: *u8) -> *void #foreign libobjc;
objc_getProtocol :: (name: *u8) -> *void #foreign libobjc;
sel_registerName :: (name: *u8) -> *void #foreign libobjc;
class_getInstanceMethod :: (cls: *void, sel: *void) -> *void #foreign libobjc;
class_getClassMethod :: (cls: *void, sel: *void) -> *void #foreign libobjc;
Method_Description :: struct {{ name: *void; types: *u8; }}
protocol_getMethodDescription :: (p: *void, sel: *void, required: bool, instance: bool) -> Method_Description #foreign libobjc;
protocol_copyProtocolList :: (p: *void, count: *u32) -> **void #foreign libobjc;
free :: (p: *void) #foreign libc;
class_getSuperclass :: (cls: *void) -> *void #foreign libobjc;
class_copyPropertyList :: (cls: *void, count: *u32) -> **void #foreign libobjc;
property_getName :: (property: *void) -> *u8 #foreign libobjc;
property_getAttributes :: (property: *void) -> *u8 #foreign libobjc;
object_getClass :: (object: *void) -> *void #foreign libobjc;
class_respondsToSelector :: (cls: *void, sel: *void) -> bool #foreign libobjc;
objc_msgSend :: () #foreign libobjc;

Entry :: struct {{ class: string; selector: string; instance: bool; }}
ENTRIES :: Entry.[
{entries}];
FRAMEWORKS :: string.["Foundation", "AppKit", "QuartzCore", "Metal", "GameController"];

protocol_has :: (p: *void, sel: *void, instance: bool) -> bool {{
    for required: bool.[true, false] {{
        if protocol_getMethodDescription(p, sel, required, instance).name return true;
    }}
    count: u32;
    inherited := protocol_copyProtocolList(p, *count);
    defer free(inherited);
    for 0..cast(s64) count - 1 if protocol_has(inherited[it], sel, instance) return true;
    return false;
}}

// A declared property of `cls` or a superclass whose getter or setter is `selector`. Classes that
// implement their properties in a hidden subclass (Metal's descriptors) declare them here only.
// Class properties are declared on the metaclass.
property_declares :: (cls: *void, selector: string) -> bool {{
    c := cls;
    while c {{
        count: u32;
        properties := class_copyPropertyList(c, *count);
        defer free(properties);
        for 0..cast(s64) count - 1 {{
            name := to_string(property_getName(properties[it]));
            getter := name;
            capitalized := copy_temporary_string(name);
            capitalized[0] = to_upper(capitalized[0]);
            setter := tprint("set%:", capitalized);
            for attribute: split(to_string(property_getAttributes(properties[it])), ",") {{
                if attribute.count == 0 continue;
                if attribute[0] == #char "G" getter = slice(attribute, 1, attribute.count - 1);
                if attribute[0] == #char "S" setter = slice(attribute, 1, attribute.count - 1);
                if attribute == "R" setter = "";
            }}
            if selector == getter || selector == setter return true;
        }}
        c = class_getSuperclass(c);
    }}
    return false;
}}

// Class clusters (`NSString`) answer for the subclass `alloc` returns. The object is not
// initialized and not released, so no code of the class runs but `alloc`.
allocated_responds :: (cls: *void, sel: *void) -> bool {{
    send: (receiver: *void, selector: *void) -> *void #c_call;
    send = cast(type_of(send)) objc_msgSend;
    object := send(cls, sel_registerName("alloc"));
    return object && class_respondsToSelector(object_getClass(object), sel);
}}

main :: () {{
    for FRAMEWORKS {{
        path := tprint("/System/Library/Frameworks/%1.framework/%1\0", it);
        if !dlopen(path.data, 2) print("cannot load %\n", it);
    }}
    for ENTRIES {{
        class_name := tprint("%\0", it.class);
        sel := sel_registerName(tprint("%\0", it.selector).data);
        cls := objc_getClass(class_name.data);
        if cls {{
            method := ifx it.instance then class_getInstanceMethod(cls, sel) else class_getClassMethod(cls, sel);
            if method continue;
            if property_declares(ifx it.instance then cls else object_getClass(cls), it.selector) continue;
            if it.instance && allocated_responds(cls, sel) continue;
        }} else {{
            protocol := objc_getProtocol(class_name.data);
            if !protocol {{
                print("unknown %\n", it.class);
                continue;
            }}
            if protocol_has(protocol, sel, it.instance) continue;
        }}
        print("missing % % %\n", it.class, ifx it.instance then "instance" else "class", it.selector);
    }}
}}
"#
    )
}

#[test]
fn selectors_exist_in_the_objective_c_runtime() {
    let uses = scan();
    let program = check_program(&uses.receivers);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("objc-selectors");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("check.jai");
    std::fs::write(&source, program).unwrap();
    if !cfg!(target_os = "macos") {
        // Elsewhere there is no Objective-C runtime to ask; the program must still type-check.
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg("check")
            .arg(&source)
            .args(["-os", "macos"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("run")
        .arg(&source)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let listed_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/objc-selectors.txt");
    let listed: BTreeSet<String> = std::fs::read_to_string(&listed_path)
        .unwrap()
        .lines()
        .map(|l| l.split(" #").next().unwrap_or("").trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    let found: BTreeSet<String> = stdout.lines().map(str::to_string).collect();
    let mut problems = Vec::new();
    for line in found.difference(&listed) {
        problems.push(format!("not in the runtime: {line}"));
    }
    for line in listed.difference(&found) {
        problems.push(format!(
            "found now, remove from tests/objc-selectors.txt: {line}"
        ));
    }
    assert!(
        problems.is_empty(),
        "{} of {} selector uses differ from tests/objc-selectors.txt:\n{}",
        problems.len(),
        uses.receivers.len(),
        problems.join("\n")
    );
}
