//! A client that lets the server register file watchers gets the server's own, so every client
//! (not only one that configures its own watcher) tells it about disk changes.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};

fn initialized(capabilities: &Value) -> Vec<Value> {
    let mut json = JsonSession::new(Limits::default());
    json.handle_json(
        &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": capabilities}})
        .to_string(),
    )
    .unwrap();
    json.handle_json(&json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}).to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect()
}

#[test]
fn sources_and_settings_are_registered_when_the_client_allows_it() {
    let sent = initialized(
        &json!({"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}),
    );
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0]["method"], "client/registerCapability");
    let registration = &sent[0]["params"]["registrations"][0];
    assert_eq!(registration["method"], "workspace/didChangeWatchedFiles");
    let globs: Vec<&str> = registration["registerOptions"]["watchers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["globPattern"].as_str().unwrap())
        .collect();
    assert_eq!(globs, ["**/*.jai", "**/jacket.toml"]);
}

#[test]
fn nothing_is_registered_for_a_client_that_does_not_allow_it() {
    assert!(initialized(&json!({})).is_empty());
    assert!(
        initialized(
            &json!({"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": false}}})
        )
        .is_empty()
    );
}
