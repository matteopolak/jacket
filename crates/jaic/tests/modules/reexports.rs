//! Re-exports, filters and mappers keep names, storage and visibility.
use super::{compiler, rejects, succeeds};

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
fn anonymous_filtered_reexports_preserve_storage_and_visibility() {
    for filter in ["only(state)", "except(hidden, native)"] {
        let facade = format!("using,{filter} #import \"Impl\";");
        let files = [
            ("/lib/Impl/module.jai", "state: int; hidden: int;"),
            ("/lib/Facade/module.jai", facade.as_str()),
            (
                "/app/main.jai",
                "I :: #import \"Impl\"; F :: #import \"Facade\"; #assert(*I.state == *F.state); main :: () { F.state = 9; }",
            ),
        ];
        succeeds(&mut compiler(&files, false));
        let mut invalid = files;
        invalid[2].1 = "F :: #import \"Facade\"; main :: () { F.hidden = 9; }";
        rejects(&mut compiler(&invalid, false), "hidden");
        invalid[1].1 = "#scope_module using,only(state) #import \"Impl\";";
        invalid[2].1 = "F :: #import \"Facade\"; main :: () { F.state = 9; }";
        succeeds(&mut compiler(&invalid, false));
    }
    let files = [
        (
            "/lib/Impl/module.jai",
            "state: s32 #elsewhere native \"absent_storage\"; #scope_file native :: #library,system \"absent_library\";",
        ),
        (
            "/lib/Facade/module.jai",
            "using,only(state) #import \"Impl\";",
        ),
        (
            "/app/main.jai",
            "F :: #import \"Facade\"; #assert type_of(F.state) == s32; main :: () {}",
        ),
    ];
    succeeds(&mut compiler(&files, false));
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
