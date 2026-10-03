//! Inspect only freshly compiled own source and newly emitted native objects.
#![cfg(any(target_os = "linux", target_os = "macos"))]
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-cli-debug-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.jai"), "// Own primary source.\n#load \"answer.jai\";\nmain :: () -> int { return answer(); }\n").unwrap();
        fs::write(
            path.join("answer.jai"),
            "// Own loaded source.\n\nanswer :: () -> int { return 42; }\n",
        )
        .unwrap();
        Self(path)
    }
    fn compile(&self, action: &str, name: &str, flags: &[&str]) -> PathBuf {
        let output = self.0.join(name);
        let result = Command::new(env!("CARGO_BIN_EXE_jai-rs"))
            .arg(action)
            .arg(self.0.join("main.jai"))
            .arg(&output)
            .args(flags)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        output
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn dwarfdump_command() -> Command {
    let tool = std::env::var_os("LLVM_SYS_221_PREFIX")
        .map(PathBuf::from)
        .map(|prefix| prefix.join("bin/llvm-dwarfdump"))
        .filter(|path| path.is_file())
        .or_else(|| {
            std::env::var_os("PATH").and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|path| path.join("llvm-dwarfdump"))
                    .find(|path| path.is_file())
            })
        })
        .expect("installed LLVM dwarfdump is required")
        .canonicalize()
        .unwrap();
    for protected in ["reference", "corpus", ".git"] {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(protected);
        let root = root.canonicalize().unwrap_or(root);
        assert!(!tool.starts_with(root));
    }
    let mut command = Command::new(tool);
    for variable in [
        "LD_PRELOAD",
        "LD_LIBRARY_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_LIBRARY_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_FRAMEWORK_PATH",
    ] {
        command.env_remove(variable);
    }
    command
}
#[test]
fn source_files_names_and_declaration_lines_survive_native_object_emission() {
    let fixture = Fixture::new();
    let ir =
        fs::read_to_string(fixture.compile("emit-llvm", "program.ll", &["-g", "-O0"])).unwrap();
    assert!(ir.contains(&format!(
        "source_filename = \"{}\"",
        fixture.0.join("main.jai").canonicalize().unwrap().display()
    )));
    assert!(ir.contains("name: \"main\", linkageName:"));
    assert!(ir.contains("name: \"answer\", linkageName:"));
    assert!(ir.contains("DILocation(line: 3, column: 23"));
    let object = fixture.compile("emit-object", "program.o", &["-gline-tables-only"]);
    let output = dwarfdump_command()
        .args(["--debug-info", "--debug-line"])
        .arg(object)
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("main.jai"), "{text}");
    assert!(text.contains("answer.jai"), "{text}");
    let executable = fixture.compile("build", "program", &["-g", "-O0"]);
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
}
#[test]
fn explicit_debug_off_does_not_emit_source_debug_metadata() {
    let fixture = Fixture::new();
    let ir =
        fs::read_to_string(fixture.compile("emit-llvm", "program.ll", &["-g", "-g0"])).unwrap();
    assert!(!ir.contains("DICompileUnit"));
    assert!(!ir.contains("!dbg"));
}

#[test]
fn source_statement_stepping_and_shadowed_local_variables_survive_object_emission() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "compute :: (input: int) -> int {\n  answer := input;\n  {\n    answer := 2;\n    input += answer;\n  }\n  return input;\n}\nmain :: () -> int { return compute(40); }\n").unwrap();
    let ir = fs::read_to_string(fixture.compile("emit-llvm", "locals.ll", &["-g", "-O0"])).unwrap();
    assert!(ir.contains("DILexicalBlock"), "{ir}");
    assert!(ir.contains("name: \"input\", arg: 1"), "{ir}");
    assert_eq!(
        ir.matches("DILocalVariable(name: \"answer\"").count(),
        2,
        "{ir}"
    );
    for line in [2, 4, 5, 7] {
        assert!(
            ir.contains(&format!("DILocation(line: {line},")),
            "missing line {line}: {ir}"
        );
    }
    let object = fixture.compile("emit-object", "locals.o", &["-g", "-O0"]);
    let output = dwarfdump_command()
        .args(["--debug-info", "--debug-line"])
        .arg(object)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let dwarf = String::from_utf8(output.stdout).unwrap();
    assert!(dwarf.contains("DW_TAG_formal_parameter"), "{dwarf}");
    assert!(dwarf.contains("DW_TAG_lexical_block"), "{dwarf}");
    assert_eq!(
        dwarf.matches("DW_AT_name\t(\"answer\")").count(),
        2,
        "{dwarf}"
    );
    for line in [2, 4, 5, 7] {
        assert!(
            dwarf.lines().any(|row| {
                let cols: Vec<_> = row.split_whitespace().collect();
                cols.first().is_some_and(|v| v.starts_with("0x"))
                    && cols.get(1).is_some_and(|v| *v == line.to_string())
            }),
            "missing source line {line}: {dwarf}"
        );
    }
    let executable = fixture.compile("build", "locals", &["-g", "-O0"]);
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
    let line_only =
        fs::read_to_string(fixture.compile("emit-llvm", "lines.ll", &["-gline-tables-only"]))
            .unwrap();
    assert!(!line_only.contains("DILocalVariable"), "{line_only}");
    assert!(line_only.contains("DILocation(line: 5,"), "{line_only}");
}

#[test]
fn loop_local_scope_and_deferred_statement_locations_use_actual_ir_paths() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "main :: () -> int {\n  result := 40;\n  {\n    defer result += 1;\n    for index: 0..0 {\n      result += 1;\n    }\n  }\n  return result;\n}\n").unwrap();
    let ir =
        fs::read_to_string(fixture.compile("emit-llvm", "cleanup.ll", &["-g", "-O0"])).unwrap();
    assert_eq!(
        ir.matches("DILocalVariable(name: \"index\"").count(),
        1,
        "{ir}"
    );
    for line in [4, 5, 6, 9] {
        assert!(
            ir.contains(&format!("DILocation(line: {line},")),
            "missing line {line}: {ir}"
        );
    }
    let object = fixture.compile("emit-object", "cleanup.o", &["-g", "-O0"]);
    let output = dwarfdump_command()
        .args(["--debug-info", "--debug-line"])
        .arg(object)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let dwarf = String::from_utf8(output.stdout).unwrap();
    assert!(dwarf.contains("DW_AT_name\t(\"index\")"), "{dwarf}");
    let executable = fixture.compile("build", "cleanup", &["-g", "-O0"]);
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
}

#[test]
fn inserted_quote_keeps_original_file_and_line_in_caller_scope() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "#load \"quote.jai\";\nmain :: () -> int {\n value := 1;\n #insert,scope() BODY;\n return value;\n}\n").unwrap();
    fs::write(
        fixture.0.join("quote.jai"),
        "// original quote\nBODY :: #code {\n value += 41;\n};\n",
    )
    .unwrap();
    let ir = fs::read_to_string(fixture.compile("emit-llvm", "quote.ll", &["-g", "-O0"])).unwrap();
    assert!(ir.contains("filename: \"quote.jai\""), "{ir}");
    assert!(ir.contains("DILocation(line: 3, column: 2"), "{ir}");
    assert!(ir.contains("DILocalVariable(name: \"value\""), "{ir}");
    let object = fixture.compile("emit-object", "quote.o", &["-g", "-O0"]);
    let output = dwarfdump_command()
        .args(["--debug-info", "--debug-line"])
        .arg(object)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let dwarf = String::from_utf8(output.stdout).unwrap();
    assert!(dwarf.contains("quote.jai"), "{dwarf}");
    assert!(dwarf.contains("main.jai"), "{dwarf}");
    let mut current_file = None;
    let mut quoted_file = None;
    for line in dwarf.lines() {
        if let Some(index) = line.trim().strip_prefix("file_names[") {
            current_file = index
                .split(']')
                .next()
                .and_then(|index| index.trim().parse::<usize>().ok());
        } else if line.trim().starts_with("name:") && line.contains("\"quote.jai\"") {
            quoted_file = current_file;
        }
    }
    let quoted_file = quoted_file.expect("quote must have its own line-table file entry");
    assert!(
        dwarf.lines().any(|line| {
            let columns: Vec<_> = line.split_whitespace().collect();
            columns
                .first()
                .is_some_and(|address| address.starts_with("0x"))
                && columns.get(1) == Some(&"3")
                && columns
                    .get(3)
                    .is_some_and(|file| file.parse::<usize>() == Ok(quoted_file))
        }),
        "the inserted addition must step at quote.jai:3: {dwarf}"
    );
    let executable = fixture.compile("build", "quote", &["-g", "-O0"]);
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
}

#[test]
fn source_record_and_pointer_types_survive_o0_and_o2_native_debug() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "Point :: struct { pad: u8; x: int; }\nNode :: struct { next: *Node; value: int; }\ninspect :: (sample: Point, address: *Point) -> int { return sample.x + address.x; }\nmain :: () -> int {\n point: Point;\n point.pad = 3;\n point.x = 21;\n pointer := *point;\n node: Node;\n node.value = 21;\n node.next = *node;\n if node.next.value != 21 return 99;\n return no_inline inspect(point, pointer);\n}\n").unwrap();
    for optimization in ["-O0", "-O2"] {
        let ir = fs::read_to_string(fixture.compile(
            "emit-llvm",
            &format!("records{optimization}.ll"),
            &["-g", optimization],
        ))
        .unwrap();
        for name in ["Point", "Node", "pad", "x", "next", "value"] {
            assert!(
                ir.contains(&format!("name: \"{name}\"")),
                "missing {name}: {ir}"
            );
        }
        assert!(ir.contains("DW_TAG_pointer_type"), "{ir}");
        assert!(ir.contains("name: \"sample\", arg: 1"), "{ir}");
        assert!(ir.contains("name: \"address\", arg: 2"), "{ir}");
        assert!(!ir.contains("<temporary!>"), "{ir}");
        let object = fixture.compile(
            "emit-object",
            &format!("records{optimization}.o"),
            &["-g", optimization],
        );
        let output = dwarfdump_command()
            .args(["--debug-info", "--debug-line"])
            .arg(object)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let dwarf = String::from_utf8(output.stdout).unwrap();
        for tag in [
            "DW_TAG_structure_type",
            "DW_TAG_pointer_type",
            "DW_TAG_member",
            "DW_TAG_formal_parameter",
        ] {
            assert!(dwarf.contains(tag), "{optimization}: {dwarf}");
        }
        for name in [
            "Point", "Node", "pad", "x", "next", "value", "sample", "address", "point", "pointer",
            "node",
        ] {
            assert!(
                dwarf.contains(&format!("DW_AT_name\t(\"{name}\")")),
                "{optimization} missing {name}: {dwarf}"
            );
        }
        let executable = fixture.compile(
            "build",
            &format!("records{optimization}"),
            &["-g", optimization],
        );
        assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
    }
}

#[test]
fn source_layout_and_debug_descriptors_share_the_selected_cross_target() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "Packet :: struct { address:*u8; payload:int; }\nanswer :: (value:int) -> int #no_context { return value; }\nmain :: () -> int { packet:Packet; callback := answer; callback(42); return size_of(Packet); }\n").unwrap();
    for (triple, pointer_bits, record_bytes) in [
        ("i686-unknown-linux-gnu", 32, 12),
        ("x86_64-unknown-linux-gnu", 64, 16),
    ] {
        let flags = ["-g", "-O0", "--target", triple];
        let ir = fs::read_to_string(fixture.compile("emit-llvm", &format!("{triple}.ll"), &flags))
            .unwrap();
        assert!(
            ir.contains(&format!("ret i64 {record_bytes}")),
            "semantic size_of must use selected target: {ir}"
        );
        assert!(
            ir.lines().any(|line| line.contains("DW_TAG_pointer_type")
                && line.contains(&format!("size: {pointer_bits}"))),
            "{ir}"
        );
        assert!(
            ir.lines().any(|line| line.contains("DW_TAG_member")
                && line.contains("name: \"payload\"")
                && line.contains(&format!("offset: {pointer_bits}"))),
            "{ir}"
        );
        let callback = ir
            .lines()
            .find(|line| line.contains("DILocalVariable(name: \"callback\""))
            .unwrap_or_else(|| panic!("missing source callback: {ir}"));
        let pointer = referenced_node(&ir, callback, "type: ");
        assert!(pointer.contains("DW_TAG_pointer_type"), "{pointer}");
        assert!(
            pointer.contains(&format!("size: {pointer_bits}")),
            "source procedure pointer must use the selected target: {pointer}"
        );
        let signature = referenced_node(&ir, pointer, "baseType: ");
        assert!(
            signature.contains("DISubroutineType(flags: DIFlagPrototyped"),
            "{signature}"
        );
        let members = referenced_node(&ir, signature, "types: ");
        let references: Vec<_> = members
            .split_once("!{")
            .unwrap()
            .1
            .strip_suffix('}')
            .unwrap()
            .split(", ")
            .collect();
        assert_eq!(
            references.len(),
            2,
            "the source callback has one return and one parameter: {members}"
        );
        assert_eq!(references[0], references[1], "{members}");
        assert!(
            metadata_node(&ir, references[0])
                .contains("DIBasicType(name: \"s64\", size: 64, encoding: DW_ATE_signed)"),
            "{members}"
        );
        let object = fixture.compile("emit-object", &format!("{triple}.o"), &flags);
        let output = dwarfdump_command()
            .arg("--debug-info")
            .arg(object)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let dwarf = String::from_utf8(output.stdout).unwrap();
        assert!(
            dwarf.contains(&format!("addr_size = 0x{:02x}", pointer_bits / 8)),
            "{dwarf}"
        );
        assert!(
            dwarf.contains(&format!("DW_AT_byte_size\t(0x{record_bytes:02x})")),
            "{dwarf}"
        );
        assert!(
            dwarf.contains(&format!(
                "DW_AT_data_member_location\t(0x{:02x})",
                pointer_bits / 8
            )),
            "{dwarf}"
        );
        assert!(dwarf.contains("DW_AT_name\t(\"packet\")"), "{dwarf}");
        assert!(dwarf.contains("DW_AT_name\t(\"callback\")"), "{dwarf}");
        assert!(dwarf.contains("DW_TAG_subroutine_type"), "{dwarf}");
    }
}

fn metadata_node<'a>(ir: &'a str, reference: &str) -> &'a str {
    ir.lines()
        .find(|line| line.starts_with(&format!("{reference} = ")))
        .unwrap_or_else(|| panic!("missing referenced metadata {reference}: {ir}"))
}
fn referenced_node<'a>(ir: &'a str, node: &str, field: &str) -> &'a str {
    let reference = node
        .split_once(field)
        .unwrap_or_else(|| panic!("missing {field}: {node}"))
        .1
        .split([',', ')'])
        .next()
        .unwrap();
    metadata_node(ir, reference)
}

#[test]
fn indirect_procedure_locals_retain_the_checked_debug_signature() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "answer :: () -> int #no_context { return 42; }\nmain :: () -> int {\n callback := answer;\n return callback();\n}\n").unwrap();
    for optimization in ["-O0", "-O2"] {
        let ir = fs::read_to_string(fixture.compile(
            "emit-llvm",
            &format!("callback{optimization}.ll"),
            &["-g", optimization],
        ))
        .unwrap();
        assert!(ir.contains("DISubprogram(name: \"main\""), "{ir}");
        assert!(ir.contains("DILocation(line: 4,"), "{ir}");
        let variable = ir
            .lines()
            .find(|line| line.contains("DILocalVariable(name: \"callback\""))
            .unwrap_or_else(|| panic!("missing callback variable: {ir}"));
        let pointer = referenced_node(&ir, variable, "type: ");
        assert!(pointer.contains("DW_TAG_pointer_type"), "{pointer}");
        assert!(
            pointer.contains(&format!("size: {}", usize::BITS)),
            "{pointer}"
        );
        let signature = referenced_node(&ir, pointer, "baseType: ");
        assert!(
            signature.contains("DISubroutineType(flags: DIFlagPrototyped"),
            "{signature}"
        );
        assert!(signature.contains("cc: DW_CC_normal"), "{signature}");
        let members = referenced_node(&ir, signature, "types: ");
        let result = members
            .split_once("!{")
            .unwrap()
            .1
            .strip_suffix('}')
            .unwrap();
        assert!(
            !result.contains(','),
            "the source signature has no parameters: {members}"
        );
        let result_type = metadata_node(&ir, result);
        assert!(
            result_type.contains("DIBasicType(name: \"s64\", size: 64, encoding: DW_ATE_signed)"),
            "{result_type}"
        );
        let object = fixture.compile(
            "emit-object",
            &format!("callback{optimization}.o"),
            &["-g", optimization],
        );
        let output = dwarfdump_command()
            .arg("--debug-info")
            .arg(object)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let dwarf = String::from_utf8(output.stdout).unwrap();
        assert!(dwarf.contains("DW_TAG_subprogram"), "{dwarf}");
        assert!(dwarf.contains("DW_AT_name\t(\"callback\")"), "{dwarf}");
        assert!(dwarf.contains("DW_TAG_pointer_type"), "{dwarf}");
        assert!(dwarf.contains("DW_TAG_subroutine_type"), "{dwarf}");
        let executable = fixture.compile(
            "build",
            &format!("callback{optimization}"),
            &["-g", optimization],
        );
        assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
    }
}

#[test]
fn multiple_result_callback_debug_uses_the_checked_tuple_layout() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "pair :: (value:int) -> (int,int) #no_context { return value,value+2; }\nmain :: () -> int {\n callback := pair;\n first,second := callback(20);\n return first+second;\n}\n").unwrap();
    for optimization in ["-O0", "-O2"] {
        let ir = fs::read_to_string(fixture.compile(
            "emit-llvm",
            &format!("tuple{optimization}.ll"),
            &["-g", optimization],
        ))
        .unwrap();
        let variable = ir
            .lines()
            .find(|line| line.contains("DILocalVariable(name: \"callback\""))
            .unwrap_or_else(|| panic!("missing callback variable: {ir}"));
        let pointer = referenced_node(&ir, variable, "type: ");
        assert!(pointer.contains("DW_TAG_pointer_type"), "{pointer}");
        let signature = referenced_node(&ir, pointer, "baseType: ");
        assert!(
            signature.contains("DISubroutineType(flags: DIFlagPrototyped"),
            "{signature}"
        );
        let members = referenced_node(&ir, signature, "types: ");
        let references: Vec<_> = members
            .split_once("!{")
            .unwrap()
            .1
            .strip_suffix('}')
            .unwrap()
            .split(", ")
            .collect();
        assert_eq!(
            references.len(),
            2,
            "one result tuple and one source parameter: {members}"
        );
        let tuple = metadata_node(&ir, references[0]);
        assert!(
            tuple.contains("DW_TAG_structure_type, size: 128, align: 64, flags: DIFlagArtificial"),
            "{tuple}"
        );
        assert!(
            !tuple.contains("name:"),
            "result storage must not invent a source record name: {tuple}"
        );
        let parameter = metadata_node(&ir, references[1]);
        assert!(
            parameter.contains("DIBasicType(name: \"s64\", size: 64, encoding: DW_ATE_signed)"),
            "{parameter}"
        );
        let elements = referenced_node(&ir, tuple, "elements: ");
        let fields: Vec<_> = elements
            .split_once("!{")
            .unwrap()
            .1
            .strip_suffix('}')
            .unwrap()
            .split(", ")
            .collect();
        assert_eq!(fields.len(), 2, "{elements}");
        for (index, field) in fields.into_iter().enumerate() {
            let member = metadata_node(&ir, field);
            assert!(member.contains("DW_TAG_member"), "{member}");
            assert!(member.contains("size: 64, align: 64"), "{member}");
            assert!(member.contains("DIFlagArtificial"), "{member}");
            assert_eq!(referenced_node(&ir, member, "baseType: "), parameter);
            if index == 0 {
                assert!(!member.contains("offset:"), "{member}");
            } else {
                assert!(member.contains("offset: 64"), "{member}");
            }
        }
        let object = fixture.compile(
            "emit-object",
            &format!("tuple{optimization}.o"),
            &["-g", optimization],
        );
        let output = dwarfdump_command()
            .arg("--debug-info")
            .arg(object)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let dwarf = String::from_utf8(output.stdout).unwrap();
        assert!(dwarf.contains("DW_AT_name\t(\"callback\")"), "{dwarf}");
        assert!(dwarf.contains("DW_TAG_subroutine_type"), "{dwarf}");
        assert!(dwarf.contains("DW_AT_byte_size\t(0x10)"), "{dwarf}");
        assert!(
            dwarf.contains("DW_AT_data_member_location\t(0x08)"),
            "{dwarf}"
        );
        let executable = fixture.compile(
            "build",
            &format!("tuple{optimization}"),
            &["-g", optimization],
        );
        assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
    }
}

#[test]
fn no_debug_suppresses_whole_procedure_and_all_suppressed_compile_units() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "hidden :: () -> int #no_debug { ignored := 42; return ignored; }\nmain :: () -> int { return no_inline hidden(); }\n").unwrap();
    let ir =
        fs::read_to_string(fixture.compile("emit-llvm", "suppressed.ll", &["-g", "-O0"])).unwrap();
    assert!(ir.contains("DISubprogram(name: \"main\""), "{ir}");
    assert!(!ir.contains("DISubprogram(name: \"hidden\""), "{ir}");
    assert!(!ir.contains("DILocalVariable(name: \"ignored\""), "{ir}");
    let executable = fixture.compile("build", "suppressed", &["-g", "-O0"]);
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
    fs::write(
        fixture.0.join("main.jai"),
        "unused :: () -> int { return 0; }\nmain :: () -> int #no_debug { ignored := 42; return ignored; }\n",
    )
    .unwrap();
    let ir = fs::read_to_string(fixture.compile("emit-llvm", "all-suppressed.ll", &["-g", "-O0"]))
        .unwrap();
    assert!(!ir.contains("DICompileUnit"), "{ir}");
    assert!(!ir.contains("!dbg"), "{ir}");
    let executable = fixture.compile("build", "all-suppressed", &["-g", "-O0"]);
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
}

#[test]
fn no_debug_macro_omits_body_but_keeps_caller_argument_source() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "#load \"quiet.jai\";\nanswer :: () -> int { return 42; }\nmain :: () -> int {\n quiet(answer());\n return 42;\n}\n").unwrap();
    fs::write(
        fixture.0.join("quiet.jai"),
        "quiet :: (argument:int) #expand #no_debug {\n temporary := argument;\n}\n",
    )
    .unwrap();
    let ir = fs::read_to_string(fixture.compile("emit-llvm", "quiet.ll", &["-g", "-O0"])).unwrap();
    assert!(!ir.contains("filename: \"quiet.jai\""), "{ir}");
    assert!(!ir.contains("DILocalVariable(name: \"temporary\""), "{ir}");
    assert!(!ir.contains("DILocalVariable(name: \"argument\""), "{ir}");
    assert!(
        ir.contains("DILocation(line: 4,"),
        "caller argument must remain mapped: {ir}"
    );
    let executable = fixture.compile("build", "quiet", &["-g", "-O0"]);
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
}

#[test]
fn no_debug_macro_does_not_leak_inlined_callee_locations() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.jai"), "#load \"quiet.jai\";\n#load \"step.jai\";\n#program_export \"visible_run\" visible_run :: (input:int) -> int #c_call {\n output:int;\n quiet(input, *output);\n return output;\n}\nmain :: () -> int { return visible_run(41); }\n").unwrap();
    fs::write(
        fixture.0.join("quiet.jai"),
        "quiet :: (argument:int, result:*int) #expand #no_debug {\n result.* = inline step(argument);\n}\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("step.jai"),
        "step :: (argument:int) -> int #no_context {\n return argument + 1;\n}\n",
    )
    .unwrap();
    for optimization in ["-O0", "-O2"] {
        let ir = fs::read_to_string(fixture.compile(
            "emit-llvm",
            &format!("inline{optimization}.ll"),
            &["-g", optimization],
        ))
        .unwrap();
        let function = ir
            .lines()
            .skip_while(|line| !line.starts_with("define ") || !line.contains("@visible_run("))
            .take_while(|line| *line != "}")
            .collect::<Vec<_>>();
        assert!(!function.is_empty(), "exported function must remain: {ir}");
        assert!(
            !function
                .iter()
                .any(|line| line.contains(" call ") && !line.contains("@llvm.")),
            "step must actually inline: {ir}"
        );
        for line in function {
            for reference in line.split('!').skip(1) {
                let number: String = reference.chars().take_while(char::is_ascii_digit).collect();
                if number.is_empty() {
                    continue;
                }
                let Some(mut node) = ir
                    .lines()
                    .find(|line| line.starts_with(&format!("!{number} = ")))
                else {
                    continue;
                };
                if !node.contains("DILocation(") && !node.contains("DILocalVariable(") {
                    continue;
                }
                loop {
                    assert!(
                        !node.contains("inlinedAt:"),
                        "suppressed macro acquired callee location: {node}\n{ir}"
                    );
                    if node.contains("DISubprogram(") {
                        assert!(
                            node.contains("name: \"visible_run\""),
                            "callee debug scope leaked: {node}\n{ir}"
                        );
                        break;
                    }
                    if let Some((_, file)) = node.split_once("file: !") {
                        let file: String = file.chars().take_while(char::is_ascii_digit).collect();
                        let file = ir
                            .lines()
                            .find(|line| line.starts_with(&format!("!{file} = ")))
                            .expect("scope file");
                        assert!(
                            !file.contains("quiet.jai") && !file.contains("step.jai"),
                            "suppressed source file leaked: {file}\n{ir}"
                        );
                    }
                    let scope = node.split_once("scope: !").expect("lexical scope").1;
                    let scope: String = scope.chars().take_while(char::is_ascii_digit).collect();
                    node = ir
                        .lines()
                        .find(|line| line.starts_with(&format!("!{scope} = ")))
                        .expect("parent scope");
                }
            }
        }
        let object = fixture.compile(
            "emit-object",
            &format!("inline{optimization}.o"),
            &["-g", optimization],
        );
        let output = dwarfdump_command()
            .args(["--name=visible_run", "--show-children"])
            .arg(object)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let dwarf = String::from_utf8(output.stdout).unwrap();
        assert!(dwarf.contains("DW_TAG_subprogram"), "{dwarf}");
        assert!(
            !dwarf.contains("DW_TAG_inlined_subroutine"),
            "callee DWARF leaked into suppressed macro: {dwarf}"
        );
        assert!(
            !dwarf.contains("quiet.jai") && !dwarf.contains("step.jai"),
            "suppressed DWARF source leaked: {dwarf}"
        );
        let executable = fixture.compile(
            "build",
            &format!("inline{optimization}"),
            &["-g", optimization],
        );
        assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
    }
}
