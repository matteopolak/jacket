//! What bounds compile-time code in the editor: a budget of blocks for each evaluation, and a
//! wall-clock limit for the whole check. Both are the client's settings
//! (`compileTime.evaluationBudget`, `compileTime.checkTimeLimit`); a project cannot set them.
use jai_language_server::{Diagnostic, DiagnosticSeverity, JsonSession, Limits, Session};
use serde_json::{Value, json};
mod common;

const SOURCE: &str = r"spin :: () { while true {} }
#run spin();

Info :: ($T: Type) -> *Type_Info {
    TI :: type_info(T);
    return TI;
}

Point :: struct { x: int; }

main :: () {
    info := Info(Point);
    sum := #run 1 + 2;
}
";

fn errors(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect()
}

#[test]
fn only_the_runaway_run_is_reported_where_it_was_started() {
    let mut s = common::session();
    // The budget must run out first; on a slow machine 200M blocks outlast the default time limit.
    s.set_check_time_limit(0.0);
    s.open(common::uri(), 1, SOURCE.into()).unwrap();
    let errors = errors(s.diagnostics(&common::uri()).unwrap());
    let lines: Vec<u32> = errors.iter().map(|d| d.range.start.line).collect();
    assert_eq!(lines, [1], "{errors:#?}");
    let message = &errors[0].message;
    assert!(
        message.contains("execution budget exhausted (66000000 steps)"),
        "{message}"
    );
    assert!(message.contains("still running `spin` at"), "{message}");
    assert!(message.contains("main.jai:1"), "{message}");
}

/// Runs of 100 thousand blocks pass under the default budget but not under a smaller one.
const HEAVY: &str = r"count :: (n: int) -> int {
    total := 0;
    for 0..n { total += it; }
    return total;
}
light :: #run count(10);
heavy :: #run count(100000);
main :: () { total := light + heavy; total += 1; }
";

fn heavy_session() -> (Session, jai_language_server::DocumentUri) {
    let dir = common::project("budget-config", &[("main.jai", HEAVY)]);
    let (s, uri, _) = common::open_file(&dir, "main.jai");
    (s, uri)
}

#[test]
fn the_evaluation_budget_bounds_each_evaluation() {
    let (mut s, uri) = heavy_session();
    assert_eq!(errors(s.diagnostics(&uri).unwrap()), []);
    s.set_evaluation_budget(20_000);
    let found = errors(s.diagnostics(&uri).unwrap());
    let lines: Vec<u32> = found.iter().map(|d| d.range.start.line).collect();
    assert_eq!(lines, [6], "{found:#?}");
    assert!(
        found[0].message.contains("(20000 steps)"),
        "{}",
        found[0].message
    );
    assert!(
        found[0]
            .message
            .contains("raise the compile-time evaluation budget"),
        "{}",
        found[0].message
    );
    // Zero is no limit.
    s.set_evaluation_budget(0);
    assert_eq!(errors(s.diagnostics(&uri).unwrap()), []);
}

#[test]
fn a_project_cannot_raise_the_budget() {
    // `[check]` is not a setting of the project: the budget stays what the client set.
    let dir = common::project(
        "budget-toml",
        &[
            ("main.jai", HEAVY),
            ("jacket.toml", "[check]\nmax_blocks = 0\nmax_seconds = 0\n"),
        ],
    );
    let (mut s, uri, _) = common::open_file(&dir, "main.jai");
    s.set_evaluation_budget(20_000);
    let found = errors(s.diagnostics(&uri).unwrap());
    assert_eq!(found.len(), 1, "{found:#?}");
}

#[test]
fn the_wall_clock_limit_stops_the_check_with_one_notice() {
    let dir = common::project("wall-clock", &[("main.jai", SOURCE)]);
    let (mut s, uri, _) = common::open_file(&dir, "main.jai");
    s.set_evaluation_budget(0);
    s.set_check_time_limit(0.3);
    let found = s.diagnostics(&uri).unwrap();
    let notices: Vec<_> = found
        .iter()
        .filter(|d| d.message.contains("checking stopped"))
        .collect();
    assert_eq!(notices.len(), 1, "{found:#?}");
    assert_eq!(notices[0].severity, DiagnosticSeverity::Warning);
    assert!(
        notices[0]
            .message
            .starts_with("checking stopped after 0.3 s; results are partial"),
        "{}",
        notices[0].message
    );
    assert_eq!(errors(found), []);
}

#[test]
fn costly_evaluations_get_a_lens_and_a_ranked_list() {
    let dir = common::project("cost-lens", &[("main.jai", HEAVY)]);
    let (mut s, uri, _) = common::open_file(&dir, "main.jai");
    s.set_cost_lens_blocks(5000);
    let lenses: Vec<_> = s
        .code_lenses(&uri)
        .unwrap()
        .into_iter()
        .filter(|l| l.command.command == "jai.showCompileTimeCosts")
        .collect();
    // Only `heavy` is above the threshold; `light` stays clean.
    assert_eq!(lenses.len(), 1, "{lenses:#?}");
    assert_eq!(lenses[0].range.start.line, 6);
    let title = &lenses[0].command.title;
    assert!(title.contains(" blocks · "), "{title}");
    let costs = s.compile_time_costs(&uri);
    assert!(costs.len() >= 2, "{} evaluations", costs.len());
    assert_eq!(costs[0].location.range.start.line, 6);
    assert!(costs.windows(2).all(|w| w[0].blocks >= w[1].blocks));
    assert!(costs[0].blocks >= 100_000, "{}", costs[0].blocks);
}

#[test]
fn an_isolated_host_run_is_in_the_cost_list_at_its_own_directive() {
    let source = "#import \"Basic\";\n\n\n\n\nvalue :: #run,host (() -> int { total := 0; for 1..20000 total += it; return total; })();\nmain :: () { print(\"%\", value); }\n";
    let dir = common::project("cost-host-run", &[("main.jai", source)]);
    // A program for another OS: `#run,host` runs in a compiler of its own.
    let other = if cfg!(windows) {
        jaic::sema::TargetOs::Linux
    } else {
        jaic::sema::TargetOs::Windows
    };
    let mut s = Session::with_environment(
        Limits::default(),
        common::environment_at(common::stdlib(), Some(other)),
    );
    s.set_workspace_folders(vec![dir.clone()]);
    let (uri, _) = common::open_from_disk(&mut s, &dir, "main.jai");
    s.set_cost_lens_blocks(5000);
    let costs = s.compile_time_costs(&uri);
    assert!(
        costs
            .iter()
            .any(|c| c.location.range.start.line == 5 && c.blocks >= 20_000),
        "{:#?}",
        costs
            .iter()
            .take(5)
            .map(|c| (
                c.location.uri.as_str().to_string(),
                c.location.range.start.line,
                c.blocks
            ))
            .collect::<Vec<_>>()
    );
    let lenses = s.code_lenses(&uri).unwrap();
    assert!(
        lenses
            .iter()
            .any(|l| l.command.command == "jai.showCompileTimeCosts" && l.range.start.line == 5),
        "{lenses:#?}"
    );
}

#[test]
fn cheap_evaluations_show_no_lens_by_default() {
    let dir = common::project("cost-lens-default", &[("main.jai", HEAVY)]);
    let (s, uri, _) = common::open_file(&dir, "main.jai");
    let lenses = s.code_lenses(&uri).unwrap();
    assert!(
        lenses
            .iter()
            .all(|l| l.command.command != "jai.showCompileTimeCosts"),
        "{lenses:#?}"
    );
}

fn send(session: &mut JsonSession, id: Option<u64>, method: &str, params: Value) -> Vec<Value> {
    let mut message = json!({ "jsonrpc": "2.0", "method": method, "params": params });
    if let Some(id) = id {
        message["id"] = json!(id);
    }
    session
        .handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect()
}

fn limit_notices(messages: &[Value]) -> Vec<&Value> {
    messages
        .iter()
        .filter(|m| m["method"] == "jai/compileTimeLimit")
        .collect()
}

const RUNAWAY: &str = "spin :: () { while true {} }\n#run spin();\nmain :: () {}\n";

fn open_runaway(s: &mut JsonSession, uri: &str) -> Vec<Value> {
    let mut messages = send(
        s,
        None,
        "textDocument/didOpen",
        json!({ "textDocument": { "uri": uri, "languageId": "jai", "version": 1, "text": RUNAWAY } }),
    );
    messages.extend(send(
        s,
        Some(2),
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": uri } }),
    ));
    messages
}

#[test]
fn the_client_hears_of_a_limit_at_the_evaluation_that_hit_it() {
    let mut s = JsonSession::with_environment(Limits::default(), common::environment());
    send(
        &mut s,
        Some(1),
        "initialize",
        json!({
            "capabilities": {},
            "initializationOptions": {
                "compileTime": { "evaluationBudget": 50_000, "limitNotices": true },
            },
        }),
    );
    send(&mut s, None, "initialized", json!({}));
    let uri = "file:///limit-notice/main.jai";
    let messages = open_runaway(&mut s, uri);
    let notices = limit_notices(&messages);
    assert_eq!(notices.len(), 1, "{messages:#?}");
    let params = &notices[0]["params"];
    assert_eq!(params["kind"], "budget");
    assert_eq!(params["uri"], uri);
    assert_eq!(params["range"]["start"]["line"], 1);
    let message = params["message"].as_str().unwrap();
    assert!(message.contains("(50000 steps)"), "{params}");
    // Raising the setting re-checks, and the evaluation no longer hits it.
    let after = send(
        &mut s,
        None,
        "workspace/didChangeConfiguration",
        json!({ "settings": { "jai": { "compileTime": { "evaluationBudget": 200_000 } } } }),
    );
    let notices = limit_notices(&after);
    assert_eq!(notices.len(), 1, "{after:#?}");
    assert!(
        notices[0]["params"]["message"]
            .as_str()
            .unwrap()
            .contains("(200000 steps)"),
        "{after:#?}"
    );
}

#[test]
fn a_client_that_did_not_ask_gets_no_limit_notice() {
    let mut s = JsonSession::with_environment(Limits::default(), common::environment());
    send(
        &mut s,
        Some(1),
        "initialize",
        json!({
            "capabilities": {},
            "initializationOptions": { "compileTime": { "evaluationBudget": 50_000 } },
        }),
    );
    let messages = open_runaway(&mut s, "file:///limit-quiet/main.jai");
    assert_eq!(limit_notices(&messages).len(), 0, "{messages:#?}");
}
