use super::super::refs::type_paths;
use super::*;

fn library(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jaic-doc-model-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (file, text) in files {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
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
    // The alias chain retains the first hop and then the struct itself.
    assert!(
        params[0]
            .refs
            .iter()
            .any(|r| r.file == point.file && r.line == point.line)
    );
    assert_eq!(at(&params[0], "Point"), Some((dir.join("module.jai"), 3)));
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
fn private_foreign_helpers_do_not_make_own_declarations_bindings() {
    for scope in ["#scope_file", "#scope_module"] {
        let source = format!(
            "Case :: struct {{ value: int; }}\nMode :: enum {{ A; }}\nLIMIT :: 8;\npublic_call :: native_call;\n{scope}\nlib :: #library \"native\";\nnative_call :: () #foreign lib;\n"
        );
        let dir = library(
            &format!("private-binding-{scope}"),
            &[("module.jai", &source)],
        );
        let model = module(&dir, Path::new("/nowhere"));
        for name in ["Case", "Mode", "LIMIT"] {
            let item = model.items.iter().find(|item| item.name == name).unwrap();
            assert!(!item.binding, "{scope}: {name} is an own declaration");
        }
        assert!(
            model
                .items
                .iter()
                .find(|item| item.name == "public_call")
                .unwrap()
                .binding
        );
        assert!(model.items.iter().all(|item| item.name != "native_call"));
        let _ = std::fs::remove_dir_all(&dir);
    }
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

#[test]
fn imported_type_aliases_keep_their_identity_and_ordered_resolution_chain() {
    let dir = library(
        "imported-type-aliases",
        &[
            (
                "Shapes/module.jai",
                "// A point.\nPoint :: struct { x: int; }\nBox :: struct(T: Type) { value: T; }\nPixel :: Box(int);\n",
            ),
            (
                "Paint/module.jai",
                "S :: #import \"Shapes\";\n// A color.\nColor :: S.Point;\nSpecial :: S.Box(int);\nPixel_Color :: S.Pixel;\nuse :: (a: Color, b: Hidden) {}\n#scope_file\nHidden :: Middle;\nMiddle :: Color;\n",
            ),
        ],
    );
    let model = module(&dir.join("Paint"), &dir);
    let color = model.items.iter().find(|i| i.name == "Color").unwrap();
    assert_eq!(color.kind, "type");
    assert_eq!(color.file, dir.join("Paint/module.jai"));
    assert_eq!(color.line, 3);
    assert_eq!(color.value, "S.Point");
    assert_eq!(
        model
            .items
            .iter()
            .find(|i| i.name == "Special")
            .unwrap()
            .kind,
        "type"
    );
    let pixel = model
        .items
        .iter()
        .find(|i| i.name == "Pixel_Color")
        .unwrap();
    assert_eq!(pixel.kind, "type");
    assert_eq!(pixel.file, dir.join("Paint/module.jai"));
    assert_eq!(pixel.value, "S.Pixel");
    let params = &model
        .items
        .iter()
        .find(|i| i.name == "use")
        .unwrap()
        .overloads[0]
        .params;
    let sites = |p: &Param| {
        p.refs
            .iter()
            .map(|r| (r.file.clone(), r.line))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        sites(&params[0]),
        [
            (dir.join("Paint/module.jai"), 3),
            (dir.join("Shapes/module.jai"), 2)
        ]
    );
    assert_eq!(
        sites(&params[1]),
        [
            (dir.join("Paint/module.jai"), 8),
            (dir.join("Paint/module.jai"), 9),
            (dir.join("Paint/module.jai"), 3),
            (dir.join("Shapes/module.jai"), 2)
        ]
    );
    assert!(!model.items.iter().any(|i| i.name == "Hidden"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn type_aliases_in_conditional_branches_share_a_documented_item() {
    let dir = library(
        "alias-branches",
        &[(
            "module.jai",
            "Point :: struct {}\n#if OS == .MACOS {\n// A color.\nColor :: Point;\n} else {\n// A color.\nColor :: Point;\n}\nuse :: (c: Color) {}\n",
        )],
    );
    let model = module(&dir, &dir);
    let color = model.items.iter().find(|i| i.name == "Color").unwrap();
    assert_eq!(color.kind, "type");
    assert_eq!(color.also, [(dir.join("module.jai"), 7)]);
    assert!(color.platforms.is_empty());
    let refs = &model
        .items
        .iter()
        .find(|i| i.name == "use")
        .unwrap()
        .overloads[0]
        .params[0]
        .refs;
    assert_eq!(
        (refs[0].file.clone(), refs[0].line),
        (color.file.clone(), color.line)
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn type_resolution_uses_module_search_roots_and_the_loading_module_scope() {
    let dir = library(
        "module-scopes",
        &[
            ("Input/module.jai", "Key :: enum { ENTER; }\n"),
            (
                "Provider/module.jai",
                "#load \"fields.jai\";\nShared :: struct {}\n",
            ),
            (
                "Provider/fields.jai",
                "Record :: struct { key: Key; value: Shared; }\n#import \"Input\";\n",
            ),
            (
                "Consumer/module.jai",
                "#import \"Input\";\nP :: #import \"Provider\";\nRecord :: P.Record;\nuse :: (k: Key, r: Record) {}\n",
            ),
            // A sibling file does not shadow an import (even on case-insensitive file systems).
            ("Consumer/input.jai", "Unrelated :: struct {}\n"),
        ],
    );
    let model = module(&dir.join("Consumer"), &dir);
    let record = model.items.iter().find(|i| i.name == "Record").unwrap();
    assert_eq!(record.members[0].refs[0].file, dir.join("Input/module.jai"));
    assert_eq!(
        record.members[1].refs[0].file,
        dir.join("Provider/module.jai")
    );
    let use_ = model.items.iter().find(|i| i.name == "use").unwrap();
    assert_eq!(
        use_.overloads[0].params[0].refs[0].file,
        dir.join("Input/module.jai")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn type_values_are_classified_by_resolution_and_procedure_types_merge_branches() {
    let dir = library(
        "type-values",
        &[(
            "module.jai",
            "Word :: u32;\nPointer :: *Word;\nRecord :: struct(T: Type) {}\nInstance :: Record(int);\nvalue :: 4;\ncopy :: value;\n#if OS == .MACOS {\nCallback :: #type (x: Word);\n} else {\nCallback :: #type (x: Pointer);\n}\n",
        )],
    );
    let model = module(&dir, &dir);
    for name in ["Word", "Pointer", "Instance", "Callback"] {
        assert_eq!(
            model.items.iter().find(|i| i.name == name).unwrap().kind,
            "type",
            "{name}"
        );
    }
    assert_eq!(
        model.items.iter().find(|i| i.name == "copy").unwrap().kind,
        "constant"
    );
    assert_eq!(
        model
            .items
            .iter()
            .find(|i| i.name == "Callback")
            .unwrap()
            .also
            .len(),
        1
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn using_reexports_respect_filters() {
    let dir = library(
        "using-filters",
        &[
            (
                "Common/module.jai",
                "Point :: struct {}\nHidden :: struct {}\n",
            ),
            (
                "Facade/module.jai",
                "using,only(Point) Names :: #import \"Common\";\n",
            ),
            (
                "Consumer/module.jai",
                "#import \"Facade\";\n#module_parameters(MODE := Mode.ONE) { Mode :: enum { ONE; } }\nuse :: (p: Point, hidden: Hidden) {}\n",
            ),
        ],
    );
    let model = module(&dir.join("Consumer"), &dir);
    let refs = &model.parameters[0].refs;
    assert!(refs.iter().any(|r| r.path == "Mode" && !r.private));
    let params = &model
        .items
        .iter()
        .find(|i| i.name == "use")
        .unwrap()
        .overloads[0]
        .params;
    assert_eq!(params[0].refs[0].file, dir.join("Common/module.jai"));
    assert!(params[1].refs.is_empty());
    assert_eq!(
        type_paths("string.[\"Point\", \"escaped \\\" Hidden\"]"),
        [vec!["string"]]
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn declarations_in_a_parameter_body_are_items_of_the_module() {
    let dir = library(
        "parameter-body",
        &[(
            "module.jai",
            "#module_parameters(\n    // Which one.\n    MODE := Mode.ONE\n) {\n    // The choices.\n    Mode :: enum { ONE; TWO; }\n    #scope_file\n    Hidden :: struct {}\n}\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let mode = model.items.iter().find(|i| i.name == "Mode").unwrap();
    assert_eq!(mode.docs, "The choices.");
    assert!(model.items.iter().all(|i| i.name != "Hidden"));
    let refs = &model.parameters[0].refs;
    assert!(refs.iter().any(|r| r.path == "Mode" && !r.private));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_type_item_carries_its_declaration_without_comments() {
    let dir = library(
        "declaration",
        &[(
            "module.jai",
            "// A channel.\nChannel :: struct(T: Type, N := 4) {\n    buffer: *T; // trailing\n    url := \"http://x\"; // not a comment inside the string\n\n    /* block\n       comment */\n    count: int @Counted;\n    next: *Channel(T);\n}\n\nMode :: enum u8 {\n    // Slow.\n    SLOW :: 0;\n    FAST :: 1; // quick\n}\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let item = |name: &str| model.items.iter().find(|i| i.name == name).unwrap();
    assert_eq!(
        item("Channel").declaration,
        "Channel :: struct(T: Type, N := 4) {\n    buffer: *T;\n    url := \"http://x\";\n\n    count: int @Counted;\n    next: *Channel(T);\n}"
    );
    assert_eq!(
        item("Mode").declaration,
        "Mode :: enum u8 {\n    SLOW :: 0;\n    FAST :: 1;\n}"
    );
    // Struct parameters, field names and notes are not references to anything.
    assert!(item("Channel").decl_refs.iter().all(|r| r.path != "T"));
    let json = to_json(&model, &|_| None);
    assert!(json.contains("\"declaration\":\"Channel :: struct(T: Type, N := 4) {\\n"));
    // The comment is the field's docs, so it is not in the declaration too.
    assert!(!json.contains("*T; // trailing"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn comment_stripping_keeps_slashes_in_strings() {
    use super::declaration::stripped;
    let (text, places) = stripped(
        "a :: \"// kept\"; // gone\n// whole line\nb :: 1; /* x */ c :: 2;\n",
        &|_| true,
    );
    assert_eq!(text, "a :: \"// kept\";\nb :: 1;  c :: 2;");
    assert_eq!(places, [Some(0), None, Some(1), None]);
}

#[test]
fn comment_stripping_leaves_the_comments_it_is_not_asked_to_drop() {
    use super::declaration::stripped;
    let (text, places) = stripped("a :: 1; // gone\n// kept\nb :: 2; // kept too\n", &|at| {
        at < 15
    });
    assert_eq!(text, "a :: 1;\n// kept\nb :: 2; // kept too");
    assert_eq!(places, [Some(0), Some(1), Some(2), None]);
}

#[test]
fn a_declaration_keeps_the_comments_that_are_not_field_docs() {
    let dir = library(
        "declaration-comments",
        &[(
            "module.jai",
            "Packet :: struct {\n    // The size.\n    size: int; // in bytes\n\n    // How the body is stored.\n    Body :: []u8;\n\n    body: Body;\n    // That is all.\n}\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let item = model.items.iter().find(|i| i.name == "Packet").unwrap();
    // The size's docs are its member's; the rest, a trailing comment the docs above displace
    // included, stays where it was written.
    assert_eq!(
        item.declaration,
        "Packet :: struct {\n    size: int; // in bytes\n\n    // How the body is stored.\n    Body :: []u8;\n\n    body: Body;\n    // That is all.\n}"
    );
    let lines: Vec<_> = item
        .members
        .iter()
        .map(|m| (m.name.as_str(), m.line))
        .collect();
    assert_eq!(lines, [("size", Some(1)), ("body", Some(6))]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn private_types_a_public_item_names_become_private_items() {
    let dir = library(
        "private-types",
        &[
            (
                "module.jai",
                "#load \"sync.jai\";\n\n// A queue.\nQueue :: struct(T: Type) {\n    senders: Waiter;\n    items: [] T;\n    kind: Sort_Mode;\n}\n\nuse :: (q: *Queue(int)) -> Ticket {}\n\n#scope_module\n\nSort_Mode :: enum { UP; DOWN; }\nUnused :: struct { x: int; }\n",
            ),
            (
                "sync.jai",
                "#scope_module\n\n// How waiters line up.\nWaiter :: struct {\n    ticket: Ticket;\n    holder: *Holder;\n}\n\nTicket :: union { n: int; }\n\nHolder :: #type (n: int);\n\n#scope_file\n\nUnreachable :: struct {}\n",
            ),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let private: Vec<_> = model
        .items
        .iter()
        .filter(|i| i.private)
        .map(|i| (i.name.as_str(), i.kind))
        .collect();
    assert_eq!(
        private,
        [
            ("Waiter", "struct"),
            ("Ticket", "union"),
            ("Sort_Mode", "enum")
        ]
    );
    let queue = model.items.iter().find(|i| i.name == "Queue").unwrap();
    let waiter = model.items.iter().find(|i| i.name == "Waiter").unwrap();
    assert!(!queue.private);
    // The declaration's names lead to the private types' declarations.
    let at = |r: &TypeRef| (r.file.clone(), r.line);
    assert!(
        queue
            .decl_refs
            .iter()
            .any(|r| r.path == "Waiter" && at(r) == (waiter.file.clone(), waiter.line))
    );
    assert_eq!(waiter.docs, "How waiters line up.");
    assert!(waiter.declaration.starts_with("Waiter :: struct {"));
    // Only what is named is included, and private items do not take part in doc links.
    assert!(
        model
            .items
            .iter()
            .all(|i| i.name != "Unused" && i.name != "Unreachable")
    );
    let json = to_json(&model, &|_| None);
    assert_eq!(json.matches("\"private\":true").count(), 3);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_prelude_is_an_implicit_module() {
    let dir = library(
        "implicit",
        &[
            (
                "prelude/module.jai",
                "// Always there.\n\nAllocator :: struct { data: *void; }\n",
            ),
            (
                "Other/module.jai",
                "Thing :: struct { allocator: Allocator; }\n",
            ),
        ],
    );
    let stdlib = dir.clone();
    let prelude = module(&dir.join("prelude"), &stdlib);
    assert!(prelude.implicit);
    let thing = module(&dir.join("Other"), &stdlib);
    assert!(!thing.implicit);
    let field = &thing.items[0].members[0];
    // The field's type leads to the prelude's declaration.
    assert_eq!(field.refs.len(), 1);
    assert!(field.refs[0].file.ends_with("prelude/module.jai"));
    assert!(to_json(&prelude, &|_| None).contains("\"implicit\":true"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn links_that_name_nothing_are_listed_with_their_line() {
    let dir = library(
        "unresolved",
        &[(
            "module.jai",
            "// A module.\n\n// Cuts. See [cut], [missing_thing] and [Gone.item]; also [Hash.key] and [x].\n//\n// The [nonexistent_one] size.\ncut :: (n: int) {}\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let own = |names: &[String]| (names == ["cut"]).then(|| "jaidoc:m/cut".to_owned());
    let json = to_json(&model, &own);
    let tail = json.split("\"unresolved\":").nth(1).unwrap();
    // `[cut]` resolves and `[x]` is prose; the rest name nothing.
    for (line, text) in [
        (3, "[missing_thing]"),
        (3, "[Gone.item]"),
        (3, "[Hash.key]"),
        (5, "[nonexistent_one]"),
    ] {
        assert!(
            tail.contains(&format!("\"line\":{line},\"text\":\"{text}\"")),
            "{text} in {tail}"
        );
    }
    assert!(!tail.contains("[cut]") && !tail.contains("[x]"), "{tail}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn directives_are_not_same_spelled_declaration_references() {
    assert_eq!(type_paths("*#Context"), Vec::<Vec<String>>::new());
    assert_eq!(
        type_paths("#type (a: Context) -> Other.Context"),
        [vec!["Context"], vec!["Other", "Context"]]
    );
    assert_eq!(type_paths("#arbitrary Context"), [vec!["Context"]]);
    let dir = library(
        "context-directive",
        &[
            (
                "module.jai",
                "Other :: #import,file \"other.jai\";\nContext :: struct { local: int; }\nActive :: #Context;\nuse :: (active: *#Context, alias: *Active, ordinary: *Context, qualified: *Other.Context) {}\n",
            ),
            ("other.jai", "Context :: struct { remote: int; }\n"),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let alias = model.items.iter().find(|i| i.name == "Active").unwrap();
    assert!(alias.refs.is_empty());
    let procedure = model.items.iter().find(|i| i.name == "use").unwrap();
    let params = &procedure.overloads[0].params;
    assert!(params[0].refs.is_empty());
    assert!(params[1].refs.iter().any(|r| r.path == "Active"));
    assert!(
        params[2]
            .refs
            .iter()
            .any(|r| r.path == "Context" && r.file == dir.join("module.jai"))
    );
    assert!(
        params[3]
            .refs
            .iter()
            .any(|r| r.path == "Other.Context" && r.file == dir.join("other.jai"))
    );
}

/// A directive's operands are read in their own scope: a parameter's name and a `$T` bound in
/// the same type are not references, even when the library declares the same name.
#[test]
fn names_a_type_binds_for_itself_are_not_references() {
    assert_eq!(
        type_paths("#type (a: $T, b: *T, c: Other.T) -> T"),
        [vec!["Other", "T"]]
    );
    // A default value follows a type, which is still a reference.
    assert_eq!(
        type_paths("#type (flags: Flags = .NONE) -> void"),
        [vec!["Flags"], vec!["void"]]
    );
    assert_eq!(
        type_paths("(items: ..Item, flags: Flags = .NONE)"),
        [vec!["Item"], vec!["Flags"]]
    );
    assert_eq!(
        type_paths("Table(key = Key, $V, V)"),
        [vec!["Table"], vec!["Key"]]
    );
    let dir = library(
        "bound-names",
        &[(
            "module.jai",
            "T :: struct { x: int; }\na :: 1;\nuse :: (callback: #type (a: $T, b: T) -> T) {}\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let procedure = model.items.iter().find(|i| i.name == "use").unwrap();
    let refs: Vec<_> = procedure.overloads[0].params[0]
        .refs
        .iter()
        .map(|r| r.path.as_str())
        .collect();
    assert!(refs.is_empty(), "{refs:?}");
}

#[test]
fn language_type_directive_aliases_are_type_items() {
    let dir = library(
        "language-type-directives",
        &[(
            "module.jai",
            "Active :: #Context;\nExtended :: #jaic_type long_double;\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    for name in ["Active", "Extended"] {
        let item = model.items.iter().find(|i| i.name == name).unwrap();
        assert_eq!(item.kind, "type", "{name}");
    }
}

#[test]
fn fields_under_a_static_if_are_members_on_their_declaration_lines() {
    let dir = library(
        "static-if-fields",
        &[(
            "module.jai",
            "// Flagged.\nFlagged :: struct {\n    // Always there.\n    a: int;\n    #if FEATURE {\n        // Only with the feature.\n        b: int;\n    } else {\n        c: s32 = -1;\n    }\n    nested: struct {\n        inner: int;\n    };\n}\n\nFEATURE :: true;\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let item = model.items.iter().find(|i| i.name == "Flagged").unwrap();
    let members: Vec<_> = item
        .members
        .iter()
        .map(|m| (m.name.as_str(), m.doc.as_str(), m.line))
        .collect();
    assert_eq!(
        members,
        [
            ("a", "Always there.", Some(1)),
            ("b", "Only with the feature.", Some(3)),
            ("c", "", Some(5)),
            ("nested", "", Some(7)),
        ]
    );
    // The condition names a constant, not a type to follow.
    assert!(item.decl_refs.iter().all(|r| r.path != "FEATURE"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn declarations_in_exclusive_branches_are_one_procedure() {
    let dir = library(
        "exclusive-branches",
        &[
            (
                "module.jai",
                "#if OS == .MACOS {\n    #load \"mac.jai\";\n} else {\n    #load \"other.jai\";\n}\n#if FLAG {\n    flagged :: (a: int) {}\n} else {\n    flagged :: (a: int) {}\n}\n#if FLAG {\n    varied :: (a: int) {}\n} else {\n    varied :: (a: string) {}\n}\nreal :: (a: int) {}\nreal :: (a: string) {}\nFLAG :: true;\n",
            ),
            (
                "mac.jai",
                "// Starts it.\nstart :: (path: string) {}\n// Differs.\ndiffers :: () {}\n",
            ),
            (
                "other.jai",
                "// Starts it.\nstart :: (path: string) {}\n// Differs a lot.\ndiffers :: () {}\n",
            ),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let count = |name: &str| {
        model
            .items
            .iter()
            .find(|i| i.name == name)
            .unwrap()
            .overloads
            .len()
    };
    // The same signature in both branches is one procedure, whatever the condition.
    assert_eq!(count("flagged"), 1);
    assert_eq!(count("start"), 1);
    let start = model.items.iter().find(|i| i.name == "start").unwrap();
    assert!(start.overloads[0].platforms.is_empty());
    // Different signatures, or different docs, stay apart; so do real overloads.
    assert_eq!(count("varied"), 2);
    assert_eq!(count("differs"), 2);
    assert_eq!(count("real"), 2);
}

#[test]
fn a_type_defined_differently_in_each_branch_keeps_every_definition() {
    let dir = library(
        "branch-definitions",
        &[(
            "module.jai",
            "#if OS == .MACOS {\n    // Mac watcher.\n    Watcher :: struct {\n        stream: *void;\n    }\n} else {\n    // Other watcher.\n    Watcher :: struct {\n        handle: s32;\n    }\n}\n#if OS == .WINDOWS {\n    Same :: struct { a: int; }\n} else {\n    Same :: struct { a: int; }\n}\n",
        )],
    );
    let model = module(&dir, &dir);
    let watcher = model.items.iter().find(|i| i.name == "Watcher").unwrap();
    assert_eq!(watcher.members[0].name, "stream");
    assert_eq!(watcher.defined_on, ["MACOS"]);
    assert!(watcher.platforms.is_empty());
    let [other] = &watcher.variants[..] else {
        panic!("{:?}", watcher.variants)
    };
    assert_eq!(other.members[0].name, "handle");
    assert_eq!(other.docs, "Other watcher.");
    assert_eq!(
        other.platforms,
        ["LINUX", "WINDOWS", "ANDROID", "IOS", "WASM"]
    );
    assert!(other.declaration.contains("handle: s32"));
    // Branches that say the same thing are one definition.
    let same = model.items.iter().find(|i| i.name == "Same").unwrap();
    assert!(same.variants.is_empty());
    let json = to_json(&model, &|_| None);
    assert!(
        json.contains("\"variants\":[{\"name\":\"Watcher\""),
        "{json}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_modules_tests_and_benches_are_not_documented() {
    let dir = library(
        "test-folders",
        &[
            ("module.jai", "own :: () {}\n"),
            ("helpers/extra.jai", "extra :: () {}\n"),
            ("tests/check.jai", "check :: () {}\n"),
            ("benches/bench.jai", "measure :: () {}\n"),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let names: Vec<_> = model.items.iter().map(|i| &*i.name).collect();
    assert_eq!(names, ["own", "extra"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn directive_operands_are_not_type_references() {
    let dir = library(
        "directive-operands",
        &[(
            "module.jai",
            "Slot :: struct {\n    base: Header;\n    #place base;\n    items: [#run count_of(Count)] u8;\n    next: *Slot;\n}\n\nHeader :: struct {}\nbase :: struct {}\nCount :: struct {}\ncount_of :: (x: Type) -> int { return 1; }\n",
        )],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let item = model.items.iter().find(|i| i.name == "Slot").unwrap();
    let mut paths: Vec<_> = item.decl_refs.iter().map(|r| r.path.as_str()).collect();
    paths.sort_unstable();
    paths.dedup();
    // `base` and `Count` are named by directives, as an expression and a field, not as types.
    assert_eq!(paths, ["Header", "Slot"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_public_top_level_insert_the_model_cannot_expand_is_reported_with_its_place() {
    let dir = library(
        "top-level-insert",
        &[
            (
                "module.jai",
                "#load \"more.jai\";\n\nvisible :: () {}\n\n#insert #run make(\"made\");\n\n#scope_file\n\n#insert #run make(\"hidden\");\n",
            ),
            (
                "more.jai",
                "\n#if true {\n    #insert \"extra :: 1;\";\n}\n",
            ),
        ],
    );
    let model = module(&dir, Path::new("/nowhere"));
    let mut places: Vec<_> = model
        .unexpanded
        .iter()
        .map(|(file, line, _)| (file.file_name().unwrap().to_str().unwrap(), *line))
        .collect();
    places.sort_unstable();
    // The private one declares nothing a reader could use; the one in a branch is still public.
    assert_eq!(places, [("module.jai", 5), ("more.jai", 3)]);
    let json = to_json(&model, &|_| None);
    assert!(json.contains("\"unexpanded\":[{\"file\":"), "{json}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn what_a_top_level_insert_declares_is_documented_as_generated() {
    let dir = library(
        "generated-items",
        &[(
            "module.jai",
            "Local :: struct { n: int; }\n\n#insert #run make();\n\n#insert #run nothing();\n\n#insert #run broken();\n",
        )],
    );
    let origin = dir.join("module.jai");
    let model = module_expanding(&dir, Path::new("/nowhere"), &mut |_, line| {
        match line {
        3 => Ok("/// A point made by the generator.\nPoint :: struct { x: int; at: Local; }\n\n/// Moves it.\nmove :: (p: *Point) {}\n".to_owned()),
        5 => Ok("\n".to_owned()),
        _ => Err("make() needs a parameter".to_owned()),
    }
    });
    let point = model.items.iter().find(|i| i.name == "Point").unwrap();
    let moved = model.items.iter().find(|i| i.name == "move").unwrap();
    let local = model.items.iter().find(|i| i.name == "Local").unwrap();
    assert!(point.generated && moved.generated && !local.generated);
    assert_eq!(point.docs, "A point made by the generator.");
    assert_eq!(moved.overloads[0].docs, "Moves it.");
    assert_eq!(point.members.len(), 2);
    // Its place names the insert, which is where its source link leads.
    let name = point.file.to_string_lossy().into_owned();
    assert_eq!(generated_origin(&name), Some((origin.to_str().unwrap(), 3)));
    assert!(generated_origin(&local.file.to_string_lossy()).is_none());
    // Names of the file that holds the `#insert` resolve in what it generated.
    let at = point.members[1]
        .refs
        .iter()
        .find(|r| r.path == "Local")
        .unwrap();
    assert_eq!((&at.file, at.line), (&local.file, local.line));
    // An empty expansion is silent; a failed one is reported with its reason.
    assert_eq!(model.unexpanded.len(), 1);
    assert_eq!(model.unexpanded[0].1, 7);
    assert_eq!(model.unexpanded[0].2, "make() needs a parameter");
    let json = to_json(&model, &|_| None);
    assert!(json.contains("\"generated\":true"), "{json}");
    assert!(
        json.contains("\"reason\":\"make() needs a parameter\""),
        "{json}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
