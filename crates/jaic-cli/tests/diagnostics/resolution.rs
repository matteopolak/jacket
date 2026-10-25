//! Unknown names, mismatches and missing modules and files.
use super::{assert_in_order, jaic_on, scratch, stderr};

#[test]
fn unknown_identifier_suggests_a_visible_name() {
    let dir = scratch("unknown-name");
    let output = jaic_on(
        &dir,
        "u.jai",
        "#import \"Basic\";\nmain :: () {\n    counter := 1;\n    print(\"%\\n\", countr);\n}\n",
        "-check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "u.jai:4:18: error: unknown identifier",
            "countr",
            "help: a similar name exists: `counter`",
        ],
    );
    let output = jaic_on(
        &dir,
        "p.jai",
        "#import \"Basic\";\nmain :: () {\n    prnt(\"x\\n\");\n}\n",
        "-check",
        &[],
    );
    assert_in_order(&stderr(&output), &["help: a similar name exists: `print`"]);
    let output = jaic_on(
        &dir,
        "n.jai",
        "main :: () { x := qqqqqq; }\n",
        "-check",
        &[],
    );
    assert!(!stderr(&output).contains("help:"), "{}", stderr(&output));
}

#[test]
fn unknown_identifier_names_the_module_or_metaprogram_that_declares_it() {
    let dir = scratch("unknown-module-name");
    // A standard-library name without its `#import`.
    let output = jaic_on(
        &dir,
        "i.jai",
        "main :: () {\n    print(\"hi\\n\");\n}\n",
        "-check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "i.jai:2:5: error: unknown identifier `print`",
            "help: `print` is declared in the `Basic` module: add `#import \"Basic\";` to this file",
        ],
    );
    assert!(
        !stderr(&output).contains("other modules"),
        "{}",
        stderr(&output)
    );
    // A name several modules declare: the most used first, then the others.
    let output = jaic_on(
        &dir,
        "l.jai",
        "main :: () {\n    x := log(2.0);\n}\n",
        "-check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "help: `log` is declared in the `Basic` module: add `#import \"Basic\";` to this file",
            "help: other modules that declare `log`: `Math`, `Extensions/Testing`",
        ],
    );
    // A module used as a namespace: bind it to its name.
    let output = jaic_on(
        &dir,
        "n.jai",
        "main :: () {\n    x := Math.sqrt(2.0);\n}\n",
        "-check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "help: `Math` is the name of a module: import it under that name with `Math :: #import \"Math\";`",
        ],
    );
    // A constant a build metaprogram adds with `add_build_string`, checked on its own.
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("build.jai"),
        "#import \"Compiler\";\n#run {\n    w := compiler_create_workspace();\n    add_build_string(\"MODE :: 2;\", w);\n    add_build_file(\"src/main.jai\", w);\n}\n",
    )
    .unwrap();
    let output = jaic_on(
        &dir,
        "src/main.jai",
        "#import \"Basic\";\nmain :: () { print(\"%\\n\", MODE); }\n",
        "-check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "error: unknown identifier `MODE`",
            "help: `MODE` is added by `build.jai` (with `add_build_string`) when it builds this file: build through it, as in `jaic build.jai`",
        ],
    );
}

#[test]
fn mismatches_point_at_the_value_and_say_what_was_expected() {
    let dir = scratch("mismatch");
    let cases: &[(&str, &[&str])] = &[
        (
            "main :: () {\n    x: int = \"hello\";\n}\n",
            &[
                "m.jai:2:14: error: type mismatch: expected `s64`, found `string`",
                "m.jai:2:8: note: expected because of this type",
            ],
        ),
        (
            "f :: (x: int) -> string {\n    return x;\n}\nmain :: () { f(1); }\n",
            &[
                "m.jai:2:12: error: type mismatch: expected `string`, found `s64`",
                "m.jai:1:18: note: expected because of this return type",
                "help: to turn a number into text, format it",
            ],
        ),
        (
            "g :: () -> int, bool {\n    return 1, \"no\";\n}\nmain :: () { g(); }\n",
            &[
                "m.jai:2:15: error: type mismatch: expected `bool`, found `string`",
                "m.jai:1:17: note: expected because of this return type",
            ],
        ),
        (
            "main :: () { a := 1.5; b: int = a; }\n",
            &[
                "error: type mismatch: expected `s64`, found `float32`",
                "help: convert with `cast(s64)`, which drops the fraction",
            ],
        ),
        (
            "f :: (a: int, b: float) {}\nmain :: () {\n    f(1, 2.0, 3);\n}\n",
            &[
                "m.jai:3:15: error: in call to `f`: too many arguments: it takes at most 2",
                "m.jai:1:6: note: `f` is declared here",
            ],
        ),
        (
            "f :: (count: int) {}\nmain :: () { f(cuont = 1); }\n",
            &[
                "error: in call to `f`: no parameter named `cuont`",
                "help: did you mean `count`?",
            ],
        ),
        (
            "f :: (a: int, b: int) {}\nmain :: () { f(1); }\n",
            &[
                "error: in call to `f`: missing argument for parameter `b`",
                "note: `f` is declared here",
                "help: pass a value for `b`",
            ],
        ),
        (
            "P :: struct { width: int; }\nmain :: () { p: P; p.widht = 1; }\n",
            &[
                "error: type `P` has no member `widht`",
                "help: a member with a similar name exists: `width`",
            ],
        ),
        (
            "main :: () { x := 1; x := 2; }\n",
            &[
                "m.jai:1:22: error: `x` is already declared in this scope",
                "m.jai:1:14: note: `x` is first declared here",
                "help: to change its value, assign with `x = ...`",
            ],
        ),
        (
            "f :: () {}\nmain :: () { r := f(); }\n",
            &[
                "error: cannot declare `r` from `f()`, which has no value",
                "help: a procedure without a return type returns nothing",
            ],
        ),
        (
            // `n + "0"` is a byte; `*` takes no one-character string.
            "twice :: (n: u8) -> u8 { return n * \"2\"; }\nmain :: () {}\n",
            &[
                "error: type mismatch: `u8` and `string` cannot be combined",
                "help: `\"2\"` is a string; for the character's code write `#char \"2\"`",
            ],
        ),
        (
            "#assert size_of(int) == 4;\nmain :: () {}\n",
            &["error: #assert failed: `size_of(int) == 4` is false"],
        ),
    ];
    for (source, expected) in cases {
        let output = jaic_on(&dir, "m.jai", source, "-check", &[]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert_in_order(&stderr(&output), expected);
    }
}

#[test]
fn modify_without_polymorph_variables_is_reported_at_the_block() {
    let dir = scratch("modify-without-polymorphs");
    let cases: &[(&str, &[&str])] = &[
        (
            // Called: the error is the declaration's, not the call's.
            "main :: () {\n    foo(5);\n}\nfoo :: (x: int) -> int #modify {\n    print(\"%\", x);\n    return x > 5;\n} {\n    return x + 1;\n}\n",
            &[
                "m.jai:4:24: error: `#modify` needs polymorph variables to modify, but `foo` has none",
                "help: if `x` should be known at compile time, bake it: `$x: int`",
                "help: `#modify` runs at compile time, once per polymorph, over the `$` parameters",
                "`x: int` is a runtime value",
            ],
        ),
        (
            // Never called.
            "main :: () {}\nbar :: () #modify { return true; } {}\n",
            &[
                "m.jai:2:11: error: `#modify` needs polymorph variables to modify, but `bar` has none",
                "add one for it to work on, or remove the block",
            ],
        ),
        (
            "main :: () {}\nFoo :: struct #modify { return true; } { a: int; }\n",
            &[
                "m.jai:2:15: error: `#modify` needs polymorph variables to modify, but `Foo` has none",
                "help: a struct's `#modify` runs at compile time over the struct's parameters",
            ],
        ),
    ];
    for (source, expected) in cases {
        let output = jaic_on(&dir, "m.jai", source, "-check", &[]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        let text = stderr(&output);
        assert_in_order(&text, expected);
        assert!(!text.contains("in call to"), "{text}");
    }
}

// rules: import.19
/// jaic's own modules live in stdlib/Extensions and are imported by that path: a bare name
/// (or the folder, or the old `Jaic_Extensions`) says which import to write.
#[test]
fn an_extension_module_without_its_path_names_the_extensions_import() {
    let dir = scratch("extension-module-path");
    let cases: [(&str, &str); 4] = [
        (
            "Long_Double",
            "help: `Long_Double` is a jaic extension (not official Jai), imported by its path: `#import \"Extensions/Long_Double\";`",
        ),
        ("Wasi_Runtime", "`#import \"Extensions/Wasi_Runtime\";`"),
        (
            "Jaic_Extensions",
            "help: `Jaic_Extensions` was replaced by `Extensions/Long_Double`: `#import \"Extensions/Long_Double\";`",
        ),
        (
            "Extensions",
            "help: `Extensions` is the folder of jaic's extension modules, not a module: import one of them by its path: `Extensions/Args`, ",
        ),
    ];
    for (i, (name, help)) in cases.iter().enumerate() {
        let file = format!("e{i}.jai");
        let output = jaic_on(
            &dir,
            &file,
            &format!("#import \"{name}\";\nmain :: () {{}}\n"),
            "-check",
            &[],
        );
        assert_eq!(output.status.code(), Some(1), "{name}");
        let text = stderr(&output);
        assert_in_order(&text, &[&format!("error: module `{name}` not found"), help]);
        assert!(!text.contains("looked for"), "{text}");
    }
    // With the path it resolves.
    let output = jaic_on(
        &dir,
        "ok.jai",
        "#import \"Extensions/Long_Double\";\nmain :: () { x: Long_Double = 1; }\n",
        "-check",
        &[],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn missing_module_lists_where_it_looked_and_a_close_name() {
    let dir = scratch("missing-module");
    let output = jaic_on(
        &dir,
        "m.jai",
        "#import \"Basik\";\nmain :: () {}\n",
        "-check",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "m.jai:1:1: error: module `Basik` not found",
            "note: looked for `Basik.jai` and `Basik/module.jai` in:",
            "/modules",
            "/stdlib",
            "help: a module with a similar name exists: `Basic`",
        ],
    );
    assert!(!text.contains("/../"), "{text}");
    // Nothing close: where modules go.
    let output = jaic_on(
        &dir,
        "n.jai",
        "#import \"Zzyzx_Engine\";\nmain :: () {}\n",
        "-check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &["help: a module of your own goes in a `modules` folder"],
    );
    // A module in a folder jaic does not search: say which `-import_dir` finds it.
    std::fs::create_dir_all(dir.join("libs/Gadgets")).unwrap();
    std::fs::write(dir.join("libs/Gadgets/module.jai"), "gadget :: 1;\n").unwrap();
    let output = jaic_on(
        &dir,
        "g.jai",
        "#import \"Gadgets\";\nmain :: () {}\n",
        "-check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "help: a module `Gadgets` exists in `libs`, which is not searched: pass `-import_dir libs`",
        ],
    );
}

#[test]
fn missing_import_folder_names_its_path() {
    let dir = scratch("missing-import-dir");
    std::fs::create_dir_all(dir.join("empty")).unwrap();
    for (source, expected) in [
        (
            "#import,dir \"gone\";",
            "error: folder `gone` does not exist",
        ),
        ("#import,dir \"empty\";", "error: `empty` has no module.jai"),
        (
            "#import,file \"gone.jai\";",
            "error: file `gone.jai` does not exist",
        ),
        // Module arguments after the path are not part of the path.
        (
            "#import,file \"../gone.jai\"(VALIDATION = true);",
            "error: file `../gone.jai` does not exist",
        ),
        (
            "#import,dir \"gone\"(VALIDATION = true);",
            "error: folder `gone` does not exist",
        ),
    ] {
        let output = jaic_on(
            &dir,
            "i.jai",
            &format!("{source}\nmain :: () {{}}\n"),
            "-check",
            &[],
        );
        assert_eq!(output.status.code(), Some(1));
        assert_in_order(
            &stderr(&output),
            &[
                expected,
                "help: `#import,file` and `#import,dir` paths are relative to the file that imports them",
            ],
        );
    }
}

#[test]
fn missing_load_file_says_where_it_looked() {
    let dir = scratch("missing-load");
    std::fs::write(dir.join("helper.jai"), "helper :: () {}\n").unwrap();
    let output = jaic_on(
        &dir,
        "l.jai",
        "#load \"helpers.jai\";\nmain :: () {}\n",
        "-check",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_in_order(
        &stderr(&output),
        &[
            "l.jai:1:7: error: file `helpers.jai` does not exist",
            "note: looked for ",
            "helpers.jai",
            "help: a file with a similar name exists: `helper.jai`",
            "help: `#load` paths are relative to the file that loads them",
        ],
    );
}
