//! Compile errors in `Extensions/Http` routes: a route that cannot work is reported where it is
//! registered, in terms of the route and the handler, not as a failure inside the module.
//! The behavior is described in docs/stdlib/http-server.md.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("http_routes")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_on(dir: &Path, command: &str, source: &str) -> Output {
    let path = dir.join("case.jai");
    std::fs::write(&path, source).unwrap();
    Command::new(JAIC)
        .arg(command)
        .arg(&path)
        .current_dir(dir)
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("JAIC_DIAGNOSTICS")
        .output()
        .unwrap()
}

/// A program that registers one route with `handler` (and the declarations it needs in `defs`).
fn route_program(defs: &str, route: &str) -> String {
    format!(
        "#import \"Basic\";\n#import \"Extensions/Http\";\n\n\
         User :: struct {{ id: int; name: string; }}\n\
         Point :: struct {{ x: int; y: int; }}\n\
         {defs}\n\n\
         main :: () {{\n    app: App;\n    get(*app, {route});\n}}\n"
    )
}

/// The route is rejected with `message` as the first error, pointing at the `get(` call, and compilation stops there.
#[track_caller]
fn rejected(name: &str, defs: &str, route: &str, message: &str) {
    let dir = scratch(name);
    let output = run_on(&dir, "check", &route_program(defs, route));
    assert_eq!(
        output.status.code(),
        Some(1),
        "{name} should fail to compile"
    );
    let text = String::from_utf8_lossy(&output.stderr).into_owned();
    let first = text.lines().next().unwrap_or("");
    assert!(
        first.starts_with("case.jai:") && first.contains(":5: error: Http: route `GET "),
        "{name}: the error should point at the route:\n{text}"
    );
    assert!(
        first.contains(message),
        "{name}: expected {message:?} in:\n{text}"
    );
    assert!(
        text.contains("get(*app, "),
        "{name}: the route's source line is shown:\n{text}"
    );
}

#[test]
fn a_valid_route_compiles() {
    let dir = scratch("valid");
    let source = route_program(
        "h :: (id: Path(\"id\", int)) -> User { return .{ id.value, \"Ada\" }; }",
        "\"/users/{id}\", h",
    );
    let output = run_on(&dir, "check", &source);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn path_name_missing_from_the_template() {
    rejected(
        "path_name",
        "h :: (id: Path(\"uid\", int)) -> User { return .{}; }",
        "\"/users/{id}\", h",
        "the handler asks for Path(\"uid\", ...), but the route has no `{uid}` parameter (the route has `{id}`)",
    );
}

#[test]
fn path_name_when_the_route_has_no_parameters() {
    rejected(
        "path_none",
        "h :: (x: Path(\"id\", int)) -> User { return .{}; }",
        "\"/users\", h",
        "the route has no parameters",
    );
}

#[test]
fn params_field_missing_from_the_template() {
    rejected(
        "params_field",
        "Wanted :: struct { uid: int; }\nh :: (p: Params(Wanted)) -> User { return .{}; }",
        "\"/users/{id}\", h",
        "the Params struct has a field `uid`, but the route has no `{uid}` parameter",
    );
}

#[test]
fn path_type_that_text_cannot_become() {
    rejected(
        "path_type",
        "h :: (id: Path(\"id\", Point)) -> User { return .{}; }",
        "\"/users/{id}\", h",
        "that type cannot be parsed from text",
    );
}

#[test]
fn two_body_extractors() {
    rejected(
        "two_bodies",
        "h :: (a: Json(User), b: Text) -> User { return .{}; }",
        "\"/users\", h",
        "more than one body extractor",
    );
}

#[test]
fn a_parameter_that_is_not_an_extractor() {
    rejected(
        "not_extractor",
        "h :: (id: Path(\"id\", int), x: int) -> User { return .{}; }",
        "\"/users/{id}\", h",
        "handler parameter 2 has type `s64`, which is not an extractor",
    );
}

#[test]
fn a_handler_returning_too_many_values() {
    rejected(
        "returns",
        "h :: () -> User, int, int { return .{}, 0, 0; }",
        "\"/users\", h",
        "the handler returns 3 values; return a value and an optional Http_Error",
    );
}

#[test]
fn a_malformed_template() {
    rejected(
        "malformed",
        "h :: () -> User { return .{}; }",
        "\"/users/{id\", h",
        "`{id` is not a valid segment",
    );
}

#[test]
fn rest_must_be_the_last_segment() {
    rejected(
        "rest",
        "h :: (r: Rest(\"p\")) -> User { return .{}; }",
        "\"/a/{p...}/b\", h",
        "so it must be the last segment",
    );
}

/// `$loc := #caller_location` is the location of the call, not of the declaration. The route
/// errors above rely on it to point at the route.
#[test]
fn baked_caller_location_is_the_call_site() {
    let dir = scratch("caller_location");
    let output = run_on(
        &dir,
        "run",
        "#import \"Basic\";\nwhere :: ($loc := #caller_location) {\n    print(\"%:%\\n\", loc.line_number, loc.character_number);\n}\nmain :: () {\n    where();\n    where();\n}\n",
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "6:5\n7:5\n");
}
