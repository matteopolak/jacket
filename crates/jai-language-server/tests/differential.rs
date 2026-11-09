//! Incremental equals fresh: a seeded random sequence of edits, undos, project-graph edits, disk
//! rewrites with watch events, and open/close/rename churn is applied to one long-lived server.
//! After every step its answers are compared with those of a cold server that opens the same
//! final state, and the protocol's progress and refresh rules are checked over everything the
//! long-lived server sent.
//!
//! The default is a few short sequences. For a long run:
//!
//! ```sh
//! JAI_DIFF_STEPS=60 JAI_DIFF_SEEDS=20 cargo nextest run -p jai-language-server --test differential
//! ```
//!
//! `JAI_DIFF_SEED=n` replays one seed. A failure prints the seed, the step and the events so far.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use test_support::lsp::{environment, project, uri_of};

// Randomness

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

fn env_number(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// A client for one server: records what the server sent and answers its requests.

const REFRESHES: [&str; 4] = [
    "workspace/diagnostic/refresh",
    "workspace/semanticTokens/refresh",
    "workspace/inlayHint/refresh",
    "workspace/codeLens/refresh",
];

struct Client {
    json: JsonSession,
    next_id: u64,
    /// `$/progress` values by token, in the order they arrived.
    progress: BTreeMap<String, Vec<Value>>,
    /// Refresh requests sent and not answered yet: id and method.
    outstanding: Vec<(Value, String)>,
    /// Whether to hold refresh answers back until `answer_refreshes` (to exercise coalescing).
    hold_refreshes: bool,
    violations: Vec<String>,
}

impl Client {
    fn start(folder: &Path, hold_refreshes: bool) -> Self {
        let mut client = Self {
            json: JsonSession::with_environment(Limits::default(), environment()),
            next_id: 100,
            progress: BTreeMap::new(),
            outstanding: Vec::new(),
            hold_refreshes,
            violations: Vec::new(),
        };
        client.request(
            "initialize",
            json!({"capabilities": {
                "window": {"workDoneProgress": true},
                "textDocument": {"diagnostic": {}},
                "workspace": {
                    "diagnostics": {"refreshSupport": true},
                    "semanticTokens": {"refreshSupport": true},
                    "inlayHint": {"refreshSupport": true},
                    "codeLens": {"refreshSupport": true},
                }},
                "workspaceFolders": [{"uri": uri_of(folder).as_str(), "name": "w"}]}),
        );
        client.notify("initialized", json!({}));
        client
    }

    /// Feed one message to the server and handle what comes back.
    fn deliver(&mut self, message: &Value) -> Vec<Value> {
        let out = self.json.handle_json(&message.to_string()).unwrap();
        self.absorb(out)
    }

    /// The responses among `out`; everything else is the server talking to its client.
    fn absorb(&mut self, out: Vec<String>) -> Vec<Value> {
        let mut responses = Vec::new();
        for text in out {
            let message: Value = serde_json::from_str(&text).unwrap();
            match (message.get("method"), message.get("id")) {
                (Some(method), Some(id)) => {
                    let method = method.as_str().unwrap().to_string();
                    if REFRESHES.contains(&method.as_str()) {
                        if self.outstanding.iter().any(|(_, m)| *m == method) {
                            self.violations
                                .push(format!("`{method}` sent while one was unanswered"));
                        }
                        self.outstanding.push((id.clone(), method));
                        if !self.hold_refreshes {
                            self.answer_refreshes();
                        }
                    } else {
                        // `window/workDoneProgress/create` and any other request of ours.
                        let reply = json!({"jsonrpc": "2.0", "id": id, "result": null});
                        let more = self.json.handle_json(&reply.to_string()).unwrap();
                        responses.extend(self.absorb(more));
                    }
                }
                (Some(method), None) => {
                    if method == "$/progress" {
                        let token = message["params"]["token"].as_str().unwrap().to_string();
                        self.progress
                            .entry(token)
                            .or_default()
                            .push(message["params"]["value"].clone());
                    }
                }
                _ => responses.push(message),
            }
        }
        responses
    }

    /// Answer every refresh request the server is waiting on, which may make it send more.
    fn answer_refreshes(&mut self) {
        while let Some((id, _)) = self.outstanding.pop() {
            let reply = json!({"jsonrpc": "2.0", "id": id, "result": null});
            let more = self.json.handle_json(&reply.to_string()).unwrap();
            self.absorb(more);
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let responses =
            self.deliver(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        responses.into_iter().find(|r| r["id"] == json!(id)).map_or(
            Value::Null,
            |r| json!({"result": r["result"], "error": r["error"]}),
        )
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.deliver(&json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    /// What a native transport does while input is quiet, until the server has nothing left.
    fn settle(&mut self) {
        for _ in 0..500 {
            let working = self.json.indexing_pending() || self.json.compile_pending();
            let (awaiting, out) = self.json.indexing_progress(working).unwrap();
            self.absorb(out);
            if awaiting {
                continue;
            }
            if self.json.has_background_work() {
                let out = self.json.background_work().unwrap();
                self.absorb(out);
            } else if self.json.compile_pending() {
                let out = self.json.warm_pending().unwrap();
                self.absorb(out);
            } else if !working {
                return;
            }
        }
        panic!("the server did not settle");
    }

    /// The protocol's rules over everything sent so far; call once the server is settled and
    /// all refreshes are answered.
    fn check_protocol(&self) -> Vec<String> {
        let mut problems = self.violations.clone();
        for (token, values) in &self.progress {
            let kinds: Vec<&str> = values.iter().map(|v| v["kind"].as_str().unwrap()).collect();
            if kinds.first() != Some(&"begin") || kinds.last() != Some(&"end") {
                problems.push(format!("progress `{token}` is not begin..end: {kinds:?}"));
            }
            if kinds.iter().filter(|k| **k == "begin").count() != 1
                || kinds.iter().filter(|k| **k == "end").count() != 1
            {
                problems.push(format!(
                    "progress `{token}` begins or ends twice: {kinds:?}"
                ));
            }
            let mut last = 0;
            for value in values.iter().filter(|v| v["kind"] != "end") {
                let percentage = value["percentage"].as_u64().unwrap_or(0);
                if percentage < last || percentage > 100 {
                    problems.push(format!(
                        "progress `{token}` went from {last} to {percentage}"
                    ));
                }
                last = percentage;
            }
        }
        if !self.outstanding.is_empty() {
            problems.push(format!("refreshes left unanswered: {:?}", self.outstanding));
        }
        problems
    }
}

// The world: a project on disk and the buffers the editor has open.

const FILES: [&str; 5] = ["main.jai", "a.jai", "b.jai", "c.jai", "d.jai"];

fn initial() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "main.jai",
            "#import \"Basic\";\n#load \"a.jai\";\n#load \"b.jai\";\n\nmain :: () {\n    value := compute(2) + extra(3);\n    print(\"%\\n\", value);\n}\n",
        ),
        (
            "a.jai",
            "Point :: struct {\n    x: int;\n    y: int;\n}\n\ncompute :: (x: int) -> int {\n    return x * 2;\n}\n",
        ),
        (
            "b.jai",
            "extra :: (n: int) -> int {\n    p: Point;\n    p.x = n;\n    return p.x + compute(n);\n}\n",
        ),
        ("c.jai", "unused_helper :: () -> int {\n    return 7;\n}\n"),
    ]
}

struct World {
    dir: PathBuf,
    /// Buffers the editor has open: version, current text, and earlier texts for undo.
    open: BTreeMap<String, (i32, String, Vec<String>)>,
    log: Vec<String>,
}

impl World {
    fn disk(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.dir.join(name)).ok()
    }

    fn uri(&self, name: &str) -> String {
        uri_of(&self.dir.join(name)).as_str().to_string()
    }
}

const WORDS: [&str; 8] = [
    "compute",
    "extra",
    "Point",
    "value",
    "x",
    "n",
    "bogus",
    "unused_helper",
];

/// A random small change to `text`, as an editor or a refactoring might make.
fn mutate(rng: &mut Rng, name: &str, text: &str, counter: u64) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    if lines.is_empty() {
        return format!("gen_{counter} :: () -> int {{ return {counter}; }}\n");
    }
    let at = rng.below(lines.len());
    match rng.below(8) {
        0 => {
            lines.remove(at);
        }
        1 => {
            let copy = lines[at].clone();
            lines.insert(at, copy);
        }
        2 => {
            let from = rng.pick(&WORDS);
            let to = rng.pick(&WORDS);
            lines[at] = lines[at].replacen(from, to, 1);
        }
        3 => lines.insert(at, format!("// note {counter}")),
        4 => {
            if let Some(stripped) = lines[at].strip_suffix(';') {
                lines[at] = stripped.to_string();
            } else {
                lines[at].push(';');
            }
        }
        5 => lines.push(format!(
            "gen_{counter} :: () -> int {{ return {counter}; }}"
        )),
        6 if name == "main.jai" => {
            let load = format!(
                "#load \"{}\";",
                rng.pick(&["c.jai", "d.jai", "b.jai", "a.jai"])
            );
            if let Some(found) = lines.iter().position(|l| *l == load) {
                lines.remove(found);
            } else {
                lines.insert(1.min(lines.len()), load);
            }
        }
        _ => {
            let word = rng.pick(&WORDS);
            lines[at].push_str(&format!(" // {word}"));
        }
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

// Events

fn step(rng: &mut Rng, world: &mut World, client: &mut Client, counter: u64) {
    let name = rng.pick(&FILES).to_string();
    match rng.below(10) {
        // Edit an open buffer, opening it first when it is closed.
        0..=2 => {
            let Some(text) = world
                .open
                .get(&name)
                .map(|o| o.1.clone())
                .or_else(|| world.disk(&name))
            else {
                return;
            };
            if !world.open.contains_key(&name) {
                open(world, client, &name, &text);
            }
            let next = mutate(rng, &name, &text, counter);
            edit(world, client, &name, next, "edit");
        }
        // Undo the last edit of an open buffer.
        3 => {
            let Some(previous) = world.open.get_mut(&name).and_then(|o| o.2.pop()) else {
                return;
            };
            let (version, text, _) = world.open.get_mut(&name).unwrap();
            *version += 1;
            *text = previous.clone();
            let (version, uri) = (*version, world.uri(&name));
            world.log.push(format!("undo {name}"));
            client.notify(
                "textDocument/didChange",
                json!({"textDocument": {"uri": uri, "version": version},
                    "contentChanges": [{"text": previous}]}),
            );
        }
        // Rewrite or create a file on disk and send the watch event.
        4 | 5 => {
            let existed = world.disk(&name);
            let base = existed.clone().unwrap_or_else(|| "\n".to_string());
            let next = mutate(rng, &name, &base, counter);
            std::fs::write(world.dir.join(&name), &next).unwrap();
            world.log.push(format!("disk write {name}"));
            watch(
                world,
                client,
                &name,
                if existed.is_some() {
                    2
                } else {
                    1
                },
            );
        }
        // Delete a file.
        6 => {
            if world.disk(&name).is_none() || name == "main.jai" {
                return;
            }
            std::fs::remove_file(world.dir.join(&name)).unwrap();
            world.log.push(format!("disk delete {name}"));
            watch(world, client, &name, 3);
        }
        // Open a file from disk.
        7 => {
            if world.open.contains_key(&name) {
                return;
            }
            if let Some(text) = world.disk(&name) {
                open(world, client, &name, &text);
            }
        }
        // Close a file.
        8 => {
            if world.open.remove(&name).is_none() {
                return;
            }
            world.log.push(format!("close {name}"));
            client.notify(
                "textDocument/didClose",
                json!({"textDocument": {"uri": world.uri(&name)}}),
            );
        }
        // Rename a file on disk (closing it first), the way an explorer does.
        _ => {
            let to = rng.pick(&FILES).to_string();
            if name == "main.jai" || to == "main.jai" || name == to || world.disk(&to).is_some() {
                return;
            }
            let Some(text) = world.disk(&name) else {
                return;
            };
            if world.open.remove(&name).is_some() {
                client.notify(
                    "textDocument/didClose",
                    json!({"textDocument": {"uri": world.uri(&name)}}),
                );
            }
            std::fs::rename(world.dir.join(&name), world.dir.join(&to)).unwrap();
            std::fs::write(world.dir.join(&to), text).unwrap();
            world.log.push(format!("rename {name} -> {to}"));
            watch(world, client, &name, 3);
            watch(world, client, &to, 1);
        }
    }
}

fn open(world: &mut World, client: &mut Client, name: &str, text: &str) {
    world.log.push(format!("open {name}"));
    world
        .open
        .insert(name.to_string(), (1, text.to_string(), Vec::new()));
    client.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": world.uri(name), "languageId": "jai",
            "version": 1, "text": text}}),
    );
}

fn edit(world: &mut World, client: &mut Client, name: &str, next: String, what: &str) {
    let entry = world.open.get_mut(name).unwrap();
    entry.2.push(std::mem::replace(&mut entry.1, next.clone()));
    entry.0 += 1;
    let version = entry.0;
    world.log.push(format!("{what} {name}"));
    client.notify(
        "textDocument/didChange",
        json!({"textDocument": {"uri": world.uri(name), "version": version},
            "contentChanges": [{"text": next}]}),
    );
}

fn watch(world: &World, client: &mut Client, name: &str, kind: u32) {
    client.notify(
        "workspace/didChangeWatchedFiles",
        json!({"changes": [{"uri": world.uri(name), "type": kind}]}),
    );
}

// Snapshots

/// Sorted, with what legitimately differs between two servers (versions, result ids) removed.
fn normalize(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("resultId");
            map.remove("version");
            for v in map.values_mut() {
                normalize(v);
            }
        }
        Value::Array(items) => {
            for v in items.iter_mut() {
                normalize(v);
            }
        }
        _ => {}
    }
}

/// Byte offsets of up to `limit` identifiers of `text`, spread over the file.
fn identifiers(text: &str, limit: usize) -> Vec<(u32, u32)> {
    let mut found = Vec::new();
    for (line, content) in text.lines().enumerate() {
        let bytes = content.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                found.push((line as u32, start as u32));
            } else {
                i += 1;
            }
        }
    }
    let stride = (found.len() / limit.max(1)).max(1);
    found.into_iter().step_by(stride).take(limit).collect()
}

fn snapshot(world: &World, client: &mut Client) -> Value {
    let mut out = BTreeMap::new();
    for (name, (_, text, _)) in &world.open {
        let uri = world.uri(name);
        let doc = json!({"uri": uri});
        let mut entry = BTreeMap::new();
        let mut diagnostics =
            client.request("textDocument/diagnostic", json!({"textDocument": doc}));
        if let Some(items) = diagnostics["result"]["items"].as_array_mut() {
            items.sort_by_key(Value::to_string);
        }
        entry.insert("diagnostics", diagnostics);
        for method in [
            "textDocument/documentSymbol",
            "textDocument/semanticTokens/full",
        ] {
            entry.insert(method, client.request(method, json!({"textDocument": doc})));
        }
        let lines = text.lines().count() as u32 + 1;
        entry.insert(
            "inlay",
            client.request(
                "textDocument/inlayHint",
                json!({"textDocument": doc, "range": {
                    "start": {"line": 0, "character": 0}, "end": {"line": lines, "character": 0}}}),
            ),
        );
        let mut at = Vec::new();
        for (line, character) in identifiers(text, 6) {
            let params =
                json!({"textDocument": doc, "position": {"line": line, "character": character}});
            for method in ["textDocument/hover", "textDocument/definition"] {
                at.push(json!({"at": [line, character], "method": method,
                    "answer": client.request(method, params.clone())}));
            }
        }
        entry.insert("positions", Value::Array(at));
        out.insert(name.clone(), json!(entry));
    }
    let mut workspace = client.request("workspace/diagnostic", json!({"previousResultIds": []}));
    if let Some(items) = workspace["result"]["items"].as_array_mut() {
        for item in items.iter_mut() {
            if let Some(list) = item["items"].as_array_mut() {
                list.sort_by_key(Value::to_string);
            }
        }
        items.sort_by_key(|item| item["uri"].to_string());
    }
    out.insert("workspace".to_string(), workspace);
    let mut value = json!(out);
    normalize(&mut value);
    value
}

fn first_difference(path: &str, a: &Value, b: &Value) -> Option<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for key in x.keys().chain(y.keys()).collect::<BTreeSet<_>>() {
                let (left, right) = (
                    x.get(key).unwrap_or(&Value::Null),
                    y.get(key).unwrap_or(&Value::Null),
                );
                if let Some(found) = first_difference(&format!("{path}/{key}"), left, right) {
                    return Some(found);
                }
            }
            None
        }
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => x
            .iter()
            .zip(y)
            .enumerate()
            .find_map(|(i, (l, r))| first_difference(&format!("{path}/{i}"), l, r)),
        _ if a == b => None,
        _ => Some(format!("{path}\n  incremental: {a}\n  cold:        {b}")),
    }
}

// The test

fn run(seed: u64, steps: u64) {
    let dir = project("differential", &initial());
    let mut world = World {
        dir: dir.path().to_path_buf(),
        open: BTreeMap::new(),
        log: Vec::new(),
    };
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut client = Client::start(&world.dir, true);
    client.settle();
    for counter in 0..steps {
        // Refreshes held from the last step are answered now, between events and checks.
        step(&mut rng, &mut world, &mut client, counter);
        if rng.below(3) == 0 {
            step(&mut rng, &mut world, &mut client, counter + 1000);
        }
        client.settle();
        client.answer_refreshes();
        client.settle();
        let incremental = snapshot(&world, &mut client);
        let mut cold_client = Client::start(&world.dir, false);
        for (name, (_, text, _)) in &world.open {
            cold_client.notify(
                "textDocument/didOpen",
                json!({"textDocument": {"uri": world.uri(name), "languageId": "jai",
                    "version": 1, "text": text}}),
            );
        }
        cold_client.settle();
        let cold = snapshot(&world, &mut cold_client);
        if let Some(difference) = first_difference("", &incremental, &cold) {
            panic!(
                "seed {seed}, step {counter}: incremental and cold servers differ at {difference}\nevents: {:#?}\nproject kept in {}",
                world.log,
                dir.keep().display()
            );
        }
    }
    client.answer_refreshes();
    client.settle();
    let problems = client.check_protocol();
    assert!(
        problems.is_empty(),
        "seed {seed}: {problems:#?}\nevents: {:#?}",
        world.log
    );
}

#[test]
fn incremental_answers_equal_a_cold_servers() {
    let steps = env_number("JAI_DIFF_STEPS", 6);
    if let Some(seed) = std::env::var("JAI_DIFF_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        run(seed, steps);
        return;
    }
    for seed in 1..=env_number("JAI_DIFF_SEEDS", 2) {
        run(seed, steps);
    }
}
