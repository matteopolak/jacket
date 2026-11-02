use super::*;

fn library(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jaic-doc-model-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (file, text) in files {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
}

#[test]
fn names_in_types_are_resolved_to_declarations() {
    assert_eq!(
        type_paths("[4] Hash.Table(string, $T)"),
        [vec!["Hash", "Table"], vec!["string"]]
    );
    assert_eq!(type_paths(":= Kind.MEMBER"), [vec!["Kind", "MEMBER"]]);
    let dir = library(
        "type-refs",
        &[
            (
                "module.jai",
                "Shapes :: #import,file \"shapes.jai\";\n\nPoint :: Shapes.Point;\nmove :: (p: *Point, by: Shapes.Point, other: Elsewhere, n: int) {}\n",
            ),
            ("shapes.jai", "Point :: struct { x: int; }\n"),
            ("extra.jai", "Elsewhere :: struct { y: int; }\n"),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let point = model.items.iter().find(|i| i.name == "Point").unwrap();
    let move_ = model.items.iter().find(|i| i.name == "move").unwrap();
    let params = &move_.overloads[0].params;
    let at = |p: &Param, name: &str| {
        p.refs
            .iter()
            .find(|r| r.path == name)
            .map(|r| (r.file.clone(), r.line))
    };
    // The alias and the written `Shapes.Point` both lead to the struct itself.
    assert_eq!(
        at(&params[0], "Point"),
        Some((point.file.clone(), point.line))
    );
    assert_eq!(
        at(&params[1], "Shapes.Point"),
        Some((point.file.clone(), point.line))
    );
    // A file of the library the entry does not load still declares names for the others.
    assert!(at(&params[2], "Elsewhere").is_some());
    assert!(params[3].refs.is_empty());
}

#[test]
fn aliases_overloads_and_members_become_items() {
    let dir = library(
        "aliases",
        &[
            (
                "module.jai",
                "// A module.\n\nImpl :: #import,file \"impl.jai\";\n\nsplit :: Impl.split;\nrenamed :: split;\nPoint :: Impl.Point;\nLIMIT :: Impl.LIMIT;\n\n#scope_module\n\nhidden :: () {}\n",
            ),
            (
                "impl.jai",
                "// Splits `s` at `byte`.\n//\n// - s: the text\n// - found: whether it was there\nsplit :: (s: string, byte: u8) -> (found: bool) { return true; }\n\n// Splits at a string.\nsplit :: (s: string, by: string, n := 1) -> bool { return true; }\n\nPoint :: struct {\n    x: int; // across\n    y: int;\n}\n\n// The most.\nLIMIT :: 4;\n",
            ),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    assert_eq!(model.docs, "A module.");
    let names: Vec<_> = model
        .items
        .iter()
        .map(|i| (i.name.as_str(), i.kind))
        .collect();
    assert_eq!(
        names,
        [
            ("split", "procedure"),
            ("renamed", "procedure"),
            ("Point", "struct"),
            ("LIMIT", "constant")
        ]
    );
    let split = &model.items[0];
    assert_eq!(split.overloads.len(), 2);
    let first = &split.overloads[0];
    assert_eq!(first.params[0].doc, "the text");
    assert_eq!(first.returns[0].doc, "whether it was there");
    assert_eq!(first.docs, "Splits `s` at `byte`.");
    assert_eq!(split.overloads[1].params[2].default, "1");
    assert_eq!(model.items[1].overloads.len(), 2);
    assert_eq!(model.items[2].members[0].doc, "across");
    assert_eq!(model.items[3].value, "4");
    assert_eq!(model.items[3].docs, "The most.");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn items_under_an_os_condition_name_their_systems() {
    let dir = library(
        "systems",
        &[
            (
                "module.jai",
                "everywhere :: Impl.everywhere;\nwindows_only :: Impl.windows_only;\nboth :: Impl.both;\nnot_windows :: Impl.not_windows;\n\n#scope_module\n\nImpl :: #import,file \"impl.jai\";\n",
            ),
            (
                "impl.jai",
                "everywhere :: () {}\n#if OS == .WINDOWS {\n    windows_only :: () {}\n    both :: () {}\n} else {\n    both :: (n: int) {}\n}\n#if OS != .WINDOWS && CPU == .X64 {\n    not_windows :: () {}\n}\n",
            ),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let systems = |name: &str| {
        model
            .items
            .iter()
            .find(|i| i.name == name)
            .map(|i| i.platforms.join(","))
    };
    assert_eq!(systems("everywhere").as_deref(), Some(""));
    assert_eq!(systems("windows_only").as_deref(), Some("WINDOWS"));
    assert_eq!(systems("both").as_deref(), Some(""));
    assert_eq!(
        systems("not_windows").as_deref(),
        Some("MACOS,LINUX,ANDROID,IOS,WASM")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_experimental_tag_is_a_flag_not_doc_text() {
    let dir = library(
        "experimental",
        &[
            (
                "module.jai",
                "// Does things.\n// @experimental\n\nsteady :: Impl.steady;\nmoving :: Impl.moving;\n\n#scope_module\n\nImpl :: #import,file \"impl.jai\";\n",
            ),
            (
                "impl.jai",
                "// Stays put.\nsteady :: () {}\n\n// Still changing.\n// @experimental\n// Second line.\nmoving :: () {}\n",
            ),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    assert!(model.experimental);
    assert_eq!(model.docs, "Does things.");
    let item = |name: &str| model.items.iter().find(|i| i.name == name).unwrap();
    assert!(!item("steady").experimental);
    assert!(item("moving").experimental);
    assert_eq!(
        item("moving").overloads[0].docs,
        "Still changing.\nSecond line."
    );
    let json = to_json(&model, &|_| None);
    assert!(json.contains("\"experimental\":true"));
    assert!(!json.contains("@experimental"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn declarations_of_foreign_procedures_are_listed_as_bindings() {
    let dir = library(
        "binding",
        &[
            (
                "module.jai",
                "#load \"declarations.jai\";\n\nown :: () {}\n",
            ),
            (
                "declarations.jai",
                "lib :: #library \"native\";\n\nNative_Mode :: enum s32 { A :: 0; B :: 1; }\nNative_Box :: struct { size: s32; }\nNATIVE_MAX :: 8;\nnative_open :: (mode: Native_Mode) -> s32 #foreign lib;\n",
            ),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let item = |name: &str| model.items.iter().find(|i| i.name == name).unwrap();
    for name in ["Native_Mode", "Native_Box", "NATIVE_MAX", "native_open"] {
        assert!(item(name).binding, "{name}");
    }
    assert_eq!(item("Native_Mode").members.len(), 2);
    assert_eq!(item("Native_Box").members.len(), 1);
    assert_eq!(item("native_open").overloads.len(), 1);
    assert!(!item("own").binding);
    let json = to_json(&model, &|_| None);
    let flagged = model.items.iter().filter(|i| i.binding).count();
    assert_eq!(json.matches("\"binding\":true").count(), flagged);
    assert_eq!(flagged, model.items.len() - 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn module_parameters_come_with_their_docs() {
    let dir = library(
        "parameters",
        &[(
            "module.jai",
            "// A module.\n\n#module_parameters(\n    // Checks more.\n    STRICT := false,\n    SIZE: s32 = 4, // Bytes.\n    UNSAID := true,\n);\n\nown :: () {}\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let rows: Vec<_> = model
        .parameters
        .iter()
        .map(|p| (&*p.name, &*p.ty, &*p.default, &*p.doc))
        .collect();
    assert_eq!(
        rows,
        [
            ("STRICT", "", "false", "Checks more."),
            ("SIZE", "s32", "4", "Bytes."),
            ("UNSAID", "", "true", ""),
        ]
    );
    let json = to_json(&model, &|_| None);
    assert!(json.contains("\"parameters\":[{\"name\":\"STRICT\""));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_procedure_comment_belongs_to_its_overload() {
    let dir = library(
        "proc-docs",
        &[(
            "module.jai",
            "// Scales a number.\n//\n// - by: the factor.\nscale :: (n: int, by: int) -> int { return n * by; }\n\n// Another name.\ntwice :: scale;\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let item = |name: &str| model.items.iter().find(|i| i.name == name).unwrap();
    // Shown once: the overload keeps the prose and moves the list into the parameters.
    assert_eq!(item("scale").docs, "");
    assert_eq!(item("scale").overloads[0].docs, "Scales a number.");
    assert_eq!(item("scale").overloads[0].params[1].doc, "the factor.");
    // An alias keeps its own comment.
    assert_eq!(item("twice").docs, "Another name.");
    let _ = std::fs::remove_dir_all(&dir);
}
