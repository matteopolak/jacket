//! Original regressions derived from docs/specs/jai and distribution policy.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::path::Path;
use std::rc::Rc;

fn compiler(files: &[(&str, &str)], bootstrap: bool) -> Compiler {
    let mut fs = VirtualFs::default();
    for &(path, source) in files {
        fs.insert(path, source.as_bytes());
    }
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    if bootstrap {
        options.preload = Some("/lib/prelude/module.jai".into());
    }
    Compiler::new(options, Rc::new(fs))
}

fn succeeds(c: &mut Compiler) {
    if let Err(error) = c.compile_program(Path::new("/app/main.jai")) {
        panic!("{}", error.render(&c.sources));
    }
}

fn rejects(c: &mut Compiler, name: &str) {
    let error = c
        .compile_program(Path::new("/app/main.jai"))
        .expect_err("source violating visibility must be rejected");
    assert!(error.message.contains(name), "{}", error.render(&c.sources));
}

#[test]
fn qualified_constant_aggregate_members_are_not_writable_places() {
    for target in [
        "M.values[0]",
        "M.record.value",
        "M.records[0].value",
        "M.Alias.values[0]",
    ] {
        let source = format!("M :: #import \"Data\"; main :: () {{ {target} = 9; }}");
        let mut c = compiler(
            &[
                (
                    "/lib/Data/module.jai",
                    "R :: struct { value:int; } values :: int.[1,2]; record :: R.{3}; records :: R.[R.{4}]; Alias :: #import \"Data\";",
                ),
                ("/app/main.jai", &source),
            ],
            false,
        );
        rejects(&mut c, "constant");
    }
    let mut c = compiler(
        &[
            (
                "/lib/Data/module.jai",
                "R :: struct { value:int; } record :: R.{3};",
            ),
            (
                "/app/main.jai",
                "M :: #import \"Data\"; pair :: () -> (bool,int) { return true,9; } main :: () { ok:, M.record.value = pair(); }",
            ),
        ],
        false,
    );
    rejects(&mut c, "constant");
}

#[test]
fn aggregate_assignment_validation_keeps_macro_caller_scope() {
    let mut c = compiler(
        &[(
            "/app/main.jai",
            "R :: struct { value:int; } write :: () #expand { `value.value = 9; } check :: () -> int { value:R; write(); return value.value; } #assert(check()==9); main :: () {}",
        )],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn implicit_runtime_load_passes_only_reviewed_module_parameters() {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/Runtime_Support/module.jai",r#"
        #module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);
        #assert(DEFINE_SYSTEM_ENTRY_POINT);
        #assert(!DEFINE_INITIALIZATION);
        #assert(ENABLE_BACKTRACE_ON_CRASH);
    "#.as_bytes());
    let mut options = Options::host();
    options.runtime_support = true;
    options.runtime_entry_point = true;
    options.runtime_initialization = false;
    options.backtrace_on_crash = true;
    options.import_paths = vec!["/lib".into()];
    let mut c = Compiler::new(options, Rc::new(fs));
    if let Err(error) = c.load_bootstrap() {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn nondefault_runtime_storage_cannot_be_silently_ignored() {
    let mut options = Options::host();
    options.temporary_storage_size = 65536;
    options.import_paths = vec!["/lib".into()];
    let mut fs = VirtualFs::default();
    fs.insert("/lib/Runtime_Support/module.jai", "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);".as_bytes());
    fs.insert("/app/main.jai", "main :: () {}".as_bytes());
    let mut c = Compiler::new(options, Rc::new(fs));
    let error = c
        .compile_program(Path::new("/app/main.jai"))
        .expect_err("missing runtime configuration must diagnose");
    assert!(error.message.contains("temporary_storage_size"));
}

#[test]
fn library_modules_do_not_ignore_unknown_excess_or_duplicate_arguments() {
    for (module, args) in [
        (
            "#module_parameters(N:int=0); touch::()->int{return N;}",
            "UNKNOWN=1",
        ),
        (
            "#module_parameters(N:int=0); touch::()->int{return N;}",
            "1,2",
        ),
        (
            "#module_parameters(N:int=0); touch::()->int{return N;}",
            "1,N=2",
        ),
        ("touch::()->int{return 0;}", "UNKNOWN=1"),
        ("#if true { touch::()->int{return 0;} }", "UNKNOWN=1"),
        ("#module_parameters(N:int); touch::()->int{return 7;}", ""),
    ] {
        let source = format!("M::#import \"Contract\"({args}); main::(){{ n:=M.touch(); }}");
        let mut c = compiler(
            &[
                ("/lib/prelude/module.jai", ""),
                ("/lib/Contract/module.jai", module),
                ("/app/main.jai", &source),
            ],
            true,
        );
        let error = c
            .compile_program(Path::new("/app/main.jai"))
            .expect_err("library imports obey ordinary module binding rules");
        assert!(
            error.message.contains("parameter") || error.message.contains("argument"),
            "{}",
            error.render(&c.sources)
        );
    }
}

#[test]
fn loaded_constructor_and_generic_factory_keep_the_declaring_type_namespace() {
    let mut c = compiler(
        &[
            (
                "/lib/Record/module.jai",
                "#load \"types.jai\"; #load \"constructor.jai\"; Alias :: Buffer;",
            ),
            (
                "/lib/Record/types.jai",
                "Buffer :: struct { value:int=seed; next:*Buffer; } #scope_file seed :: 37;",
            ),
            (
                "/lib/Record/constructor.jai",
                "create :: () -> Buffer { return F.build(Buffer); } #scope_file F :: #import \"Factory\";",
            ),
            (
                "/lib/Factory/module.jai",
                "Buffer :: struct { other:u8=99; } build :: ($T:Type) -> T { return T.{}; }",
            ),
            (
                "/app/main.jai",
                "R :: #import \"Record\"; F :: #import \"Factory\"; #assert(R.Buffer==R.Alias); #assert(R.Buffer!=F.Buffer); #assert(type_of(R.create())==R.Buffer); #assert(R.create().value==37); main :: () {}",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn loaded_module_parameters_resolve_private_file_types_and_interfaces() {
    for (declaration, args, assertion) in [
        (
            "#module_parameters(N: C.Count = 7); #scope_export touch :: () -> C.Count { return N; }",
            "",
            "#assert(M.touch()==7); #assert(type_of(M.N)==D.Count);",
        ),
        (
            "#module_parameters(N: C.Count = 7); #scope_export touch :: () -> C.Count { return N; }",
            "N=8",
            "#assert(M.touch()==8); #assert(type_of(M.N)==D.Count);",
        ),
        (
            "#module_parameters(HOOKS: $I/interface C.Required = C.Required); #scope_export touch :: () -> int { return 7; }",
            "",
            "#assert(M.HOOKS==D.Required);",
        ),
        (
            "#module_parameters(HOOKS: $I/interface C.Required = C.Required); #scope_export touch :: () -> int { return 7; }",
            "HOOKS=Good",
            "#assert(M.HOOKS==Good);",
        ),
    ] {
        let header = format!("#scope_file C :: #import \"Contract\"; {declaration}");
        let source = format!(
            "D :: #import \"Contract\"; Good :: struct {{ run :: inline (value:int) {{}} }} M :: #import \"Record\"({args}); {assertion} main :: () {{}}"
        );
        let mut c = compiler(
            &[
                (
                    "/lib/Contract/module.jai",
                    "Count :: #type,distinct s32; Required :: struct { run :: inline (value:int) {} }",
                ),
                ("/lib/Record/module.jai", "#load \"parameters.jai\";"),
                ("/lib/Record/parameters.jai", &header),
                ("/app/main.jai", &source),
            ],
            false,
        );
        succeeds(&mut c);
    }
    let mut c = compiler(
        &[
            (
                "/lib/Contract/module.jai",
                "Required :: struct { run :: inline (value:int) {} }",
            ),
            ("/lib/Record/module.jai", "#load \"parameters.jai\";"),
            (
                "/lib/Record/parameters.jai",
                "#scope_file C :: #import \"Contract\"; #module_parameters(HOOKS: $I/interface C.Required); #scope_export touch :: () -> int { return 7; }",
            ),
            (
                "/app/main.jai",
                "Broken :: struct {} M :: #import \"Record\"(HOOKS=Broken); main :: () {}",
            ),
        ],
        false,
    );
    rejects(&mut c, "run");
}

#[test]
fn unused_program_parameters_can_bind_after_the_initial_import() {
    let mut c = compiler(
        &[
            ("/lib/Record/module.jai", "#module_parameters()(N:int);"),
            (
                "/app/main.jai",
                "A :: #import \"Record\"; B :: #import \"Record\"(N=9); #assert(A.N==9); #assert(B.N==9); main :: () {}",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn module_arguments_retain_literal_precision_and_use_real_conversion() {
    let mut c = compiler(
        &[
            ("/lib/typed/module.jai", "#module_parameters(P: float32);"),
            ("/app/main.jai", "M :: #import \"typed\"(0.123456789);"),
        ],
        false,
    );
    rejects(&mut c, "requires float64");
    let mut c = compiler(
        &[
            (
                "/lib/untyped/module.jai",
                "#module_parameters(P := 0.0); value :: P;",
            ),
            (
                "/app/main.jai",
                "M :: #import \"untyped\"(0.12345678); #assert(type_of(M.value) == float64);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn editor_completions_apply_import_and_transitive_reexport_filters() {
    use jaic::sema::ide::IdeReceiver;
    use jaic::sema::scope::ScopeId;
    use jaic::sema::value::ModuleId;
    let mut c = compiler(
        &[
            (
                "/lib/Impl/module.jai",
                "shown :: 7; hidden :: 9; extra :: 11;",
            ),
            (
                "/lib/Inner/module.jai",
                "using,except(hidden) I :: #import \"Impl\";",
            ),
            (
                "/lib/Facade/module.jai",
                "using,only(shown) I :: #import \"Inner\";",
            ),
            (
                "/app/main.jai",
                "using,only(shown) I :: #import \"Impl\"; F :: #import \"Facade\"; #assert(shown == F.shown);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
    let facade = c
        .modules
        .iter()
        .position(|m| m.path.as_deref() == Some(Path::new("/lib/Facade/module.jai")))
        .unwrap();
    let exported = c.ide_members(IdeReceiver::Module(ModuleId(facade as u32)));
    assert!(exported.iter().any(|n| n.name == "shown"));
    assert!(
        !exported
            .iter()
            .any(|n| n.name == "hidden" || n.name == "extra")
    );
    let file = (0..c.sources.len())
        .map(|i| jaic::source::FileId(i as u32))
        .find(|&f| c.sources.get(f).path == "/app/main.jai")
        .unwrap();
    let scope = c.scopes.iter().position(|s| s.file == Some(file)).unwrap();
    let visible = c.ide_visible(ScopeId(scope as u32), file, u32::MAX);
    assert!(visible.iter().any(|n| n.name == "shown"));
    assert!(
        !visible
            .iter()
            .any(|n| n.name == "hidden" || n.name == "extra"),
        "{visible:?}"
    );
}

#[test]
fn procedure_and_macro_using_parameters_retain_member_filters() {
    for qualifier in ["", "#expand"] {
        for filter in ["only(x)", "only(.[\"x\"])", "except(y)", "except(.[\"y\"])"] {
            let source = format!(
                "Point :: struct {{ x: int; y: int; }} update :: (using,{filter} p: *Point) {qualifier} {{ x = 7; y = 9; }} main :: () {{ p: Point; update(*p); }}"
            );
            let mut c = compiler(&[("/app/main.jai", &source)], false);
            rejects(&mut c, "y");
        }
    }
    for qualifier in ["", "#expand"] {
        for filter in ["map(renamed=x)", "map(rename)"] {
            let source = format!(
                "rename :: (names: [] string) {{ for *names if it.* == \"x\" it.* = \"renamed\"; }} Point :: struct {{ x: int; y: int; }} update :: (using,{filter} p: *Point) {qualifier} {{ renamed = 7; }} check :: () -> int {{ p: Point; update(*p); return p.x; }} #assert(check() == 7);"
            );
            let mut c = compiler(&[("/app/main.jai", &source)], false);
            succeeds(&mut c);
        }
    }
}

#[test]
fn embedded_computed_filters_keep_original_member_storage() {
    let shared = "Point :: struct { x: int; y: int; } Container :: struct { using,only(.[\"x\"]) point: Point; } object: Container; #assert(*object.x == *object.point.x);";
    let mut c = compiler(&[("/app/main.jai", shared)], false);
    succeeds(&mut c);
    let negative = format!("{shared} #assert(object.y == 0);");
    let mut c = compiler(&[("/app/main.jai", &negative)], false);
    rejects(&mut c, "y");
    let mapped = "rename :: (names: [] string) { for *names if it.* == \"x\" it.* = \"axis\"; } Point :: struct { x: int; y: int; } Container :: struct { using,map(rename) point: Point; } object: Container; #assert(*object.axis == *object.point.x);";
    let mut c = compiler(&[("/app/main.jai", mapped)], false);
    succeeds(&mut c);
}

#[test]
fn bootstrap_alias_preserves_types_and_storage() {
    let mut c = compiler(
        &[
            (
                "/lib/prelude/module.jai",
                "Record :: struct { n: int; } state: int;",
            ),
            (
                "/app/main.jai",
                "P :: #import \"Preload\"; #assert(P.Record == Record); #assert(*P.state == *state);",
            ),
        ],
        true,
    );
    succeeds(&mut c);
    assert_eq!(
        c.modules
            .iter()
            .filter(|m| m.path.as_deref() == Some(Path::new("/lib/prelude/module.jai")))
            .count(),
        1
    );
}

#[test]
fn selected_bootstrap_cannot_be_shadowed_by_local_preload() {
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", "Record :: struct { n: int; }"),
            (
                "/app/modules/Preload/module.jai",
                "Record :: struct { n: int; }",
            ),
            (
                "/app/main.jai",
                "P :: #import \"Preload\"; #assert(P.Record == Record);",
            ),
        ],
        true,
    );
    succeeds(&mut c);
}

#[test]
fn bootstrap_discovery_uses_only_canonical_entry() {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/module.jai", b"legacy".as_slice());
    assert_eq!(jaic::find_preload(&fs, Path::new("/lib")), None);
    fs.insert("/lib/prelude/module.jai", b"".as_slice());
    assert_eq!(
        jaic::find_preload(&fs, Path::new("/lib")),
        Some("/lib/prelude/module.jai".into())
    );
    assert_eq!(
        jaic::stdlib_root_from_preload(Path::new("/lib/prelude/module.jai")),
        Some(Path::new("/lib"))
    );
}

#[test]
fn module_private_declarations_are_not_qualified_exports() {
    let mut c = compiler(
        &[
            (
                "/lib/M/module.jai",
                "#scope_module hidden :: 11; #scope_export visible :: hidden + 2;",
            ),
            (
                "/app/main.jai",
                "M :: #import \"M\"; #assert(M.visible == 13); #assert(M.hidden == 11);",
            ),
        ],
        false,
    );
    rejects(&mut c, "hidden");
}

#[test]
fn private_imports_in_sibling_files_do_not_leak() {
    let mut c = compiler(
        &[
            ("/lib/Private/module.jai", "secret :: 17;"),
            (
                "/app/piece.jai",
                "#scope_file #import \"Private\"; #assert(secret == 17);",
            ),
            (
                "/app/main.jai",
                "#load \"piece.jai\"; #assert(secret == 17);",
            ),
        ],
        false,
    );
    rejects(&mut c, "secret");
}

#[test]
fn only_import_does_not_offer_filtered_out_names_as_fallback() {
    let mut c = compiler(
        &[
            ("/lib/M/module.jai", "shown :: 7; hidden :: 9;"),
            (
                "/app/main.jai",
                "using,only(shown) M :: #import \"M\"; #assert(shown == 7); #assert(M.hidden == 9); #assert(hidden == 9);",
            ),
        ],
        false,
    );
    rejects(&mut c, "hidden");
}

#[test]
fn loaded_file_has_independent_visibility_and_private_shadowing() {
    let mut c = compiler(
        &[
            (
                "/app/piece.jai",
                "shared :: 5; #scope_file shadow :: 19; #assert(shadow == 19); #assert(local == 23); #scope_module local :: 23;",
            ),
            (
                "/app/main.jai",
                "shadow :: 13; #scope_file #load \"piece.jai\"; #assert(shared == 5); #assert(shadow == 13); #assert(local == 23);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn imported_module_cannot_look_into_importer_scope() {
    let mut c = compiler(
        &[
            ("/lib/M/module.jai", "value :: application_only;"),
            (
                "/app/main.jai",
                "application_only :: 41; M :: #import \"M\"; #assert(M.value == 41);",
            ),
        ],
        false,
    );
    rejects(&mut c, "application_only");
}

#[test]
fn applications_cannot_import_or_load_library_internals() {
    for source in [
        "#import \"internal/Hidden\"; #assert(secret == 31);",
        "H :: #import,file \"../lib/internal/Hidden/module.jai\"; #assert(H.secret == 31);",
        "H :: #import,dir \"../lib/internal/Hidden\"; #assert(H.secret == 31);",
        "#load \"../lib/internal/Hidden/module.jai\"; #assert(secret == 31);",
        "#load \"../lib/Public/../internal/Hidden/module.jai\"; #assert(secret == 31);",
    ] {
        let mut c = compiler(
            &[
                ("/lib/prelude/module.jai", ""),
                ("/lib/internal/Hidden/module.jai", "secret :: 31;"),
                ("/app/main.jai", source),
            ],
            true,
        );
        rejects(&mut c, "internal");
    }
}

#[test]
fn library_facades_can_use_private_internals_without_exporting_import_alias() {
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", ""),
            (
                "/lib/internal/Hidden/module.jai",
                "Record :: struct { n: int; } secret :: 31;",
            ),
            (
                "/lib/Public/module.jai",
                "#scope_file Impl :: #import \"internal/Hidden\"; #scope_export Record :: Impl.Record; answer :: Impl.secret;",
            ),
            (
                "/app/main.jai",
                "P :: #import \"Public\"; #assert(P.answer == 31); #assert(size_of(P.Record) == 8);",
            ),
        ],
        true,
    );
    succeeds(&mut c);
}

#[test]
fn cached_internal_module_does_not_bypass_origin_check() {
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", ""),
            ("/lib/internal/Hidden/module.jai", "secret :: 31;"),
            (
                "/lib/Public/module.jai",
                "#scope_file I :: #import \"internal/Hidden\"; #scope_export answer :: I.secret;",
            ),
            (
                "/app/main.jai",
                "P :: #import \"Public\"; #assert(P.answer == 31); H :: #import \"internal/Hidden\"; #assert(H.secret == 31);",
            ),
        ],
        true,
    );
    rejects(&mut c, "internal");
}

#[test]
fn application_cannot_use_internal_source_as_main() {
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", ""),
            ("/lib/internal/Hidden/module.jai", "value :: 31;"),
        ],
        true,
    );
    let error = c
        .compile_program(Path::new("/lib/internal/Hidden/module.jai"))
        .expect_err("internal entry is private");
    assert!(error.message.contains("internal"));
}

#[test]
fn virtual_path_normalization_preserves_consecutive_parent_components() {
    use jaic::sema::FileSystem;
    let fs = VirtualFs::default();
    assert_eq!(
        fs.canonical(Path::new("../../lib/internal/Hidden/module.jai")),
        Path::new("../../lib/internal/Hidden/module.jai")
    );
    assert_eq!(
        fs.canonical(Path::new("../a/../../b")),
        Path::new("../../b")
    );
    assert_eq!(
        fs.canonical(Path::new("/../../lib/./internal/Hidden/module.jai")),
        Path::new("/lib/internal/Hidden/module.jai")
    );
}

#[cfg(unix)]
#[test]
fn native_symlink_cannot_expose_internal_source_as_public_module() {
    use std::os::unix::fs::symlink;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/cleanroom-fixtures")
        .join(format!("internal-symlink-{}", std::process::id()));
    std::fs::create_dir_all(root.join("lib/internal/Hidden")).unwrap();
    std::fs::create_dir_all(root.join("lib/prelude")).unwrap();
    std::fs::create_dir_all(root.join("app/modules")).unwrap();
    std::fs::write(root.join("lib/prelude/module.jai"), "").unwrap();
    std::fs::write(root.join("lib/internal/Hidden/module.jai"), "secret :: 31;").unwrap();
    let alias = root.join("app/modules/Exposed.jai");
    if alias.symlink_metadata().is_ok() {
        std::fs::remove_file(&alias).unwrap();
    }
    symlink(root.join("lib/internal/Hidden/module.jai"), &alias).unwrap();
    for source in [
        "H :: #import \"Exposed\"; #assert(H.secret == 31);",
        "#load \"modules/Exposed.jai\"; #assert(secret == 31);",
    ] {
        std::fs::write(root.join("app/main.jai"), source).unwrap();
        let mut options = Options::host();
        options.runtime_support = false;
        options.preload = Some(root.join("lib/prelude/module.jai"));
        options.import_paths = vec![root.join("lib")];
        let mut c = Compiler::new(options, Rc::new(jaic::sema::NativeFs));
        let error = c
            .compile_program(&root.join("app/main.jai"))
            .expect_err("canonical internal path must be private");
        assert!(
            error.message.contains("internal"),
            "{}",
            error.render(&c.sources)
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn named_filtered_reexport_preserves_qualified_alias_and_filter() {
    let mut c = compiler(
        &[
            ("/lib/Impl/module.jai", "shown :: 7; hidden :: 9;"),
            (
                "/lib/Facade/module.jai",
                "using,only(shown) Impl :: #import \"Impl\";",
            ),
            (
                "/app/main.jai",
                "F :: #import \"Facade\"; #assert(F.shown == 7); #assert(F.Impl.hidden == 9); #assert(F.hidden == 9);",
            ),
        ],
        false,
    );
    rejects(&mut c, "hidden");
}

#[test]
fn named_except_reexport_hides_excluded_name() {
    let mut c = compiler(
        &[
            ("/lib/Impl/module.jai", "shown :: 7; hidden :: 9;"),
            (
                "/lib/Facade/module.jai",
                "using,except(hidden, nonexistent) Impl :: #import \"Impl\";",
            ),
            (
                "/app/main.jai",
                "F :: #import \"Facade\"; #assert(F.shown == 7); #assert(F.hidden == 9);",
            ),
        ],
        false,
    );
    rejects(&mut c, "hidden");
}

#[test]
fn facade_procedure_alias_preserves_full_overload_set_and_defaults() {
    let mut c = compiler(
        &[
            (
                "/lib/Impl/module.jai",
                "choose :: (x: int, increment: int = 4) -> int #no_context { return x + increment; } choose :: (x: bool) -> int #no_context { if x return 11; return 13; }",
            ),
            (
                "/lib/Facade/module.jai",
                "#scope_file I :: #import \"Impl\"; #scope_export choose :: I.choose;",
            ),
            (
                "/app/main.jai",
                "F :: #import \"Facade\"; #assert(F.choose(3) == 7); #assert(F.choose(true) == 11); #assert(F.choose(3, increment=8) == 11);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn unfiltered_reexport_preserves_mutable_storage() {
    let mut c = compiler(
        &[
            (
                "/lib/Impl/module.jai",
                "state: int; Record :: struct { value: int; }",
            ),
            ("/lib/Facade/module.jai", "using I :: #import \"Impl\";"),
            (
                "/app/main.jai",
                "I :: #import \"Impl\"; F :: #import \"Facade\"; #assert(*I.state == *F.state); #assert(I.Record == F.Record);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn module_pointer_arguments_preserve_relocation_identity() {
    let mut c = compiler(
        &[
            (
                "/lib/M/module.jai",
                "#module_parameters(P: *int); state: int; pointer :: P;",
            ),
            (
                "/app/main.jai",
                "a: int; b: int; A :: #import \"M\"(P=*a); B :: #import \"M\"(P=*b); Again :: #import \"M\"(P=*a); #assert(A.pointer == *a); #assert(B.pointer == *b); #assert(*A.state != *B.state); #assert(*A.state == *Again.state);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn module_argument_order_and_omitted_defaults_are_distinct() {
    let mut c = compiler(
        &[
            (
                "/lib/M/module.jai",
                "#module_parameters(A: int = 2, B: int = 3); state: int;",
            ),
            (
                "/app/main.jai",
                "Default :: #import \"M\"; Explicit :: #import \"M\"(A=2, B=3); Reordered :: #import \"M\"(B=3, A=2); Again :: #import \"M\"(A=2, B=3); #assert(*Default.state != *Explicit.state); #assert(*Reordered.state != *Explicit.state); #assert(*Again.state == *Explicit.state);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn computed_import_filters_preserve_filter_kind() {
    for (filter, visible, hidden) in [
        ("only(.[\"a\", \"absent\"])", "a", "b"),
        ("except(.[\"a\", \"absent\"])", "b", "a"),
        ("only names", "a", "b"),
        ("except names", "b", "a"),
    ] {
        let source = format!(
            "names :: .[\"a\"]; using,{filter} M :: #import \"M\"; #assert({visible} > 0); #assert({hidden} > 0);"
        );
        let mut c = compiler(
            &[
                ("/lib/M/module.jai", "a :: 7; b :: 9;"),
                ("/app/main.jai", &source),
            ],
            false,
        );
        rejects(&mut c, hidden);
    }
}

#[test]
fn aggregate_mapper_changes_names_and_preserves_lvalues() {
    let mut c = compiler(
        &[(
            "/app/main.jai",
            "rename :: (names: [] string) #no_context { for *names { if it.* == \"x\" it.* = \"axis\"; if it.* == \"hidden\" it.* = \"\"; } } Point :: struct { x: int; y: int; hidden: int; } check :: () -> bool #no_context { point := Point.{1, 2, 3}; { using,map(rename) point; axis = 9; y = 11; } return point.x == 9 && point.y == 11 && point.hidden == 3; } #assert(check());",
        )],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn aggregate_only_filter_hides_other_members() {
    let mut c = compiler(
        &[(
            "/app/main.jai",
            "Point :: struct { x: int; y: int; } check :: () #no_context { point: Point; using,only(x) point; x = 7; y = 9; } #run check();",
        )],
        false,
    );
    rejects(&mut c, "y");
}

#[test]
fn aggregate_using_declaration_keeps_filter() {
    let mut c = compiler(
        &[(
            "/app/main.jai",
            "Point :: struct { x: int; y: int; } check :: () #no_context { using,only(x) point: Point; x = 7; y = 9; } #run check();",
        )],
        false,
    );
    rejects(&mut c, "y");
}

#[test]
fn file_scope_aggregate_using_keeps_computed_filter() {
    let mut c = compiler(
        &[(
            "/app/main.jai",
            "Point :: struct { x: int; y: int; } point: Point; using,except(.[\"y\"]) point; #assert(*x == *point.x); #assert(*y == *point.y);",
        )],
        false,
    );
    rejects(&mut c, "y");
}

#[test]
fn mapper_omission_hides_original_short_name() {
    let mut c = compiler(
        &[(
            "/app/main.jai",
            "omit :: (names: [] string) #no_context { for *names if it.* == \"y\" it.* = \"\"; } Point :: struct { x: int; y: int; } check :: () #no_context { point: Point; using,map(omit) point; x = 7; y = 9; } #run check();",
        )],
        false,
    );
    rejects(&mut c, "y");
}

#[test]
fn invalid_computed_filter_is_an_error() {
    let mut c = compiler(
        &[
            ("/lib/M/module.jai", "a :: 7;"),
            (
                "/app/main.jai",
                "using,only(.[7, 9]) M :: #import \"M\"; #assert(a == 7);",
            ),
        ],
        false,
    );
    rejects(&mut c, "strings");
}

#[test]
fn procedure_modification_rejects_unknown_body_record() {
    let mut c = compiler(&[], false);
    assert!(
        c.modify_procedure(&jaic::records::Records::default(), 999, &[])
            .is_err()
    );
}

#[test]
fn import_message_records_describe_actual_source_entry() {
    use jaic::records::{FieldRef, Records};
    let mut c = compiler(
        &[
            ("/lib/M/module.jai", "answer :: 47;"),
            (
                "/app/main.jai",
                "M :: #import \"M\"; #assert(M.answer == 47);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
    let mut records = Records::default();
    let events = c.export_file_events(&mut records);
    let record = events.into_iter().filter(|(kind, _)| *kind == 2).map(|(_, id)| id).find(|&id| matches!(records.field(id, "module_name"), Some(FieldRef::Str(s)) if &**s == b"M")).expect("import event for M");
    assert!(matches!(
        records.field(record, "module_type"),
        Some(FieldRef::Int(4))
    ));
    assert!(
        matches!(records.field(record, "fully_pathed_filename"), Some(FieldRef::Str(s)) if &**s == b"/lib/M/module.jai")
    );
}

#[test]
fn typechecked_events_use_selected_bootstrap_library_root() {
    use jaic::records::{FieldRef, Item, Records};
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", ""),
            ("/lib/M/module.jai", "library_value :: 47;"),
            (
                "/app/main.jai",
                "M :: #import \"M\"; application_value :: M.library_value;",
            ),
        ],
        true,
    );
    succeeds(&mut c);
    let mut records = Records::default();
    let message = c.export_typechecked(&mut records).unwrap();
    let Some(FieldRef::List(items)) = records.field(message, "declarations") else {
        panic!("missing declarations");
    };
    let names: Vec<Vec<u8>> = items
        .iter()
        .map(|item| {
            let Item::Ref(typed) = item else {
                panic!("invalid typechecked record");
            };
            let Some(FieldRef::Ref(decl)) = records.field(typed, "expression") else {
                panic!("missing expression");
            };
            let Some(FieldRef::Str(name)) = records.field(decl, "name") else {
                panic!("missing declaration name");
            };
            name.to_vec()
        })
        .collect();
    assert_eq!(names, vec![b"application_value".to_vec()]);
}

#[test]
fn failed_source_loads_cannot_become_successful_cache_hits() {
    let mut c = compiler(&[("/lib/Broken/module.jai", "value :: ;")], false);
    for _ in 0..2 {
        assert!(
            c.load_module(
                "Broken",
                Path::new("/lib/Broken/module.jai"),
                vec![],
                jaic::source::Span::NONE
            )
            .is_err()
        );
    }
    let module = c.new_module("direct", None, vec![]);
    for _ in 0..2 {
        assert!(
            c.load_file(
                Path::new("/lib/Broken/module.jai"),
                module,
                jaic::source::Span::NONE
            )
            .is_err()
        );
    }
}

#[path = "support/namespace_source.rs"]
mod namespace_source;

#[test]
fn loaded_pointer_constructor_keeps_public_type_and_storage_identity() {
    let mut files = namespace_source::FILES.to_vec();
    let main = format!("{}\n#assert(check()==0);", namespace_source::SOURCE);
    files.push(("/app/main.jai", &main));
    let mut c = compiler(&files, false);
    succeeds(&mut c);
}

#[test]
fn module_interface_constraints_use_the_declaring_private_file_namespace() {
    for (arguments, valid) in [
        ("", true),
        ("(HOOKS = Good)", true),
        ("(HOOKS = Broken)", false),
    ] {
        let main = format!(
            "Good :: struct {{ run :: inline (value: int) {{}} }} Broken :: struct {{}} M :: #import \"Contract\"{arguments}; #assert !#exists(M.Required); main :: () {{ result := M.touch(); }}"
        );
        let mut c = compiler(
            &[
                (
                    "/lib/Contract/module.jai",
                    "#module_parameters(HOOKS: $I/interface Required = Required) { #scope_file Required :: struct { run :: inline (value: int) {} } } touch :: () -> int { return 7; }",
                ),
                ("/app/main.jai", &main),
            ],
            false,
        );
        if valid {
            succeeds(&mut c);
        } else {
            rejects(&mut c, "run");
        }
    }
}

#[test]
fn module_parameter_defaults_use_the_declaring_private_file_namespace() {
    let mut c = compiler(
        &[
            (
                "/lib/Config/module.jai",
                "#module_parameters(value: int = hidden_default, Record: Type = Hidden_Record) { #scope_file hidden_default :: 37; Hidden_Record :: struct { value: u64; } } exposed :: value; record_size :: size_of(Record);",
            ),
            (
                "/app/main.jai",
                "M :: #import \"Config\"; #assert(M.exposed == 37); #assert(M.record_size == 8); #assert !#exists(M.hidden_default); #assert !#exists(M.Hidden_Record);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn bundled_modules_reject_unknown_and_excess_positional_arguments() {
    for arguments in ["misspelled = true", "true, false"] {
        let main = format!("M :: #import \"Config\"({arguments});");
        let mut c = compiler(
            &[
                ("/lib/prelude/module.jai", ""),
                (
                    "/lib/Config/module.jai",
                    "#module_parameters(enabled: bool);",
                ),
                ("/app/main.jai", &main),
            ],
            true,
        );
        rejects(&mut c, "not a parameter");
    }
}

#[test]
fn implicit_runtime_does_not_ignore_explicit_import_arguments() {
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", ""),
            (
                "/lib/Runtime_Support/module.jai",
                "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);",
            ),
            (
                "/app/main.jai",
                "R :: #import \"Runtime_Support\"(unknown = true);",
            ),
        ],
        true,
    );
    c.options.runtime_support = true;
    rejects(&mut c, "not a parameter");
}

#[test]
fn ide_selected_internal_entry_is_admitted_without_opening_application_imports() {
    let files = [
        ("/lib/prelude/module.jai", ""),
        ("/lib/internal/helper/module.jai", "value :: 4;"),
        ("/app/main.jai", "H :: #import \"internal/helper\";"),
    ];
    let mut c = compiler(&files, true);
    c.ide = Some(Box::default());
    c.compile_program(Path::new("/lib/internal/helper/module.jai"))
        .unwrap_or_else(|e| panic!("{}", e.render(&c.sources)));

    let mut c = compiler(&files, true);
    c.ide = Some(Box::default());
    rejects(&mut c, "private");
    let mut c = compiler(&files, true);
    assert!(
        c.compile_program(Path::new("/lib/internal/helper/module.jai"))
            .is_err()
    );
}

#[test]
fn bootstrap_runtime_passes_only_reviewed_parameters() {
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", ""),
            (
                "/lib/Runtime_Support/module.jai",
                "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);",
            ),
            ("/app/main.jai", ""),
        ],
        true,
    );
    c.options.runtime_support = true;
    succeeds(&mut c);
    assert_eq!(
        c.modules[c.runtime_support.unwrap().0 as usize]
            .params
            .len(),
        3
    );
}
