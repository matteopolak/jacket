//! Typed Postgres queries in the editor: the first check leaves the query unknown while the
//! describe service connects, and once its answer arrives a bad column is reported where it is.
//! Skipped unless `JACKET_DATABASE_URL` names a database holding the schema of
//! `stdlib/Extensions/Postgres/tests/typed/schema.sql`, for example
//! `postgres://postgres@127.0.0.1:5432/jai_typed_test`. The describe worker reads the variable
//! itself, so it has to be set for the whole test run.
use jai_language_server::services::EditorServices;
use jai_language_server::{DocumentUri, Limits, Session};
use jaic::service::{ServiceConfig, Services};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};
mod common;

fn built_jaic() -> Option<PathBuf> {
    let name = format!("jaic{}", std::env::consts::EXE_SUFFIX);
    let exe = std::env::current_exe().ok()?;
    exe.ancestors()
        .skip(1)
        .map(|dir| dir.join(&name))
        .find(|jaic| jaic.is_file())
}

fn database_url() -> Option<String> {
    std::env::var("JACKET_DATABASE_URL")
        .ok()
        .filter(|url| !url.is_empty())
}

fn messages(s: &Session, uri: &DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .map(|d| d.message)
        .collect()
}

const PROGRAM: &str = r#"#import "Basic";
Pg :: #import "Extensions/Postgres";
lookup :: (db: *Pg.Connection) {
    rows, error := Pg.query(db, "select id, emal from users");
}
main :: () {}
"#;

#[test]
fn a_bad_column_is_reported_once_the_describe_answer_arrives() {
    let (Some(jaic), Some(_)) = (built_jaic(), database_url()) else {
        eprintln!("skipped: needs a built jaic and JACKET_DATABASE_URL");
        return;
    };
    let dir = common::project("pg-typed-editor", &[("main.jai", PROGRAM)]);
    let mut environment = common::environment();
    environment.services = Some(Rc::new(EditorServices::new(Services::new(ServiceConfig {
        jaic: Some(jaic),
        ..ServiceConfig::default()
    }))));
    let mut s = Session::with_environment(Limits::default(), environment);
    s.set_workspace_folders(vec![dir.clone()]);
    let (uri, _) = common::open_from_disk(&mut s, &dir, "main.jai");
    // The first check does not wait for the database: no errors yet.
    assert_eq!(messages(&s, &uri), Vec::<String>::new());
    // Connecting and describing are separate answers: each arrival checks the file again, which
    // may ask the next question.
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut found = Vec::new();
    while !found.iter().any(|m: &String| m.contains("42703")) {
        assert!(Instant::now() < deadline, "no 42703 error: {found:?}");
        std::thread::sleep(Duration::from_millis(50));
        if s.take_service_arrivals() {
            found = messages(&s, &uri);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
