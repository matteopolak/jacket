//! The work-done progress a client shows while the workspace is indexed: begin, reports that
//! only move forward, end.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};
use test_support::lsp::{environment, project, uri_of};

fn messages(json: &mut JsonSession, message: &Value) -> Vec<Value> {
    json.handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect()
}

/// Runs the transport's idle loop (progress, then one piece of background work) to the end,
/// answering the client side of the token handshake, and returns the `$/progress` values.
fn progress_values(files: &[(&str, &str)]) -> Vec<Value> {
    let dir = project("progress", files);
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    let root = uri_of(dir.path());
    messages(
        &mut json,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "capabilities": {"window": {"workDoneProgress": true}},
            "workspaceFolders": [{"uri": root.as_str(), "name": "w"}]}}),
    );
    messages(
        &mut json,
        &json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    );
    let mut values = Vec::new();
    for _ in 0..200 {
        let working = json.indexing_pending();
        let (awaiting, out) = json.indexing_progress(working).unwrap();
        for message in out {
            let message: Value = serde_json::from_str(&message).unwrap();
            if message["method"] == "window/workDoneProgress/create" {
                messages(
                    &mut json,
                    &json!({"jsonrpc": "2.0", "id": message["id"], "result": null}),
                );
            } else if message["method"] == "$/progress" {
                values.push(message["params"]["value"].clone());
            }
        }
        if awaiting {
            // The transport serves input only until the client acknowledges the token.
            continue;
        }
        if !working {
            break;
        }
        if json.has_background_work() {
            json.background_work().unwrap();
        }
    }
    values
}

#[test]
fn progress_counts_entries_and_never_goes_back() {
    let values = progress_values(&[
        (
            "jacket.toml",
            "[project]\nentry = [\"a.jai\", \"b.jai\", \"c.jai\"]\n",
        ),
        ("a.jai", "a :: () {}\nmain :: () { a(); }\n"),
        ("b.jai", "b :: () {}\nmain :: () { b(); }\n"),
        ("c.jai", "c :: () {}\nmain :: () { c(); }\n"),
    ]);
    assert_eq!(
        values.first().map(|v| v["kind"].clone()),
        Some(json!("begin"))
    );
    assert_eq!(values.last().map(|v| v["kind"].clone()), Some(json!("end")));
    let reports: Vec<&Value> = values.iter().filter(|v| v["kind"] != "end").collect();
    let percentages: Vec<u64> = reports
        .iter()
        .map(|v| {
            v["percentage"]
                .as_u64()
                .expect("every begin and report has a percentage")
        })
        .collect();
    assert_eq!(percentages[0], 0, "begin starts at zero: {values:?}");
    assert!(
        percentages.windows(2).all(|w| w[0] <= w[1]),
        "never goes back: {percentages:?}"
    );
    assert!(
        percentages.last().unwrap() > &50 && percentages.len() > 3,
        "advances through the entries: {percentages:?}"
    );
    assert!(percentages.iter().all(|p| *p <= 100));
    let texts: Vec<&str> = reports
        .iter()
        .filter_map(|v| v["message"].as_str())
        .collect();
    assert!(
        texts.iter().any(|m| {
            m.split_once('/')
                .is_some_and(|(a, b)| a.parse::<u32>().is_ok() && b.parse::<u32>().is_ok())
        }),
        "messages read `done/total`: {texts:?}"
    );
}
