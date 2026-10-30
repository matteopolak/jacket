//! Compile-time services in the editor: an answer that is not in yet leaves the code that asked
//! for it unknown (no diagnostics), and once it arrives the file is checked again with it.
use jai_language_server::services::EditorServices;
use jai_language_server::{DocumentUri, Limits, Session};
use jaic::service::{ServiceConfig, Services};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// The `jaic` the workspace build made in the target folder this test runs from, if any.
fn built_jaic() -> Option<PathBuf> {
    let name = format!("jaic{}", std::env::consts::EXE_SUFFIX);
    let exe = std::env::current_exe().ok()?;
    exe.ancestors()
        .skip(1)
        .map(|dir| dir.join(&name))
        .find(|jaic| jaic.is_file())
}

fn session_with_services(jaic: PathBuf, folder: &std::path::Path) -> Session {
    let mut environment = test_support::lsp::environment();
    environment.services = Some(Rc::new(EditorServices::new(Services::new(ServiceConfig {
        jaic: Some(jaic),
        ..ServiceConfig::default()
    }))));
    let mut s = Session::with_environment(Limits::default(), environment);
    s.set_workspace_folders(vec![folder.to_path_buf()]);
    s
}

fn messages(s: &Session, uri: &DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .map(|d| d.message)
        .collect()
}

const PROGRAM: &str = r#"#import "Basic";
Service :: #import "Extensions/Service";
UPPER :: "{UPPER}";
ANSWER :: #run ask("{REQUEST}");
ask :: (request: string) -> string {
    status, reply := Service.call(UPPER, request);
    if status != .OK  Service.unknown(tprint("% %", status, reply));
    return reply;
}
#assert ANSWER != "LATE" "the service answered";
main :: () {}
"#;

fn program(request: &str) -> String {
    let upper = test_support::lsp::repo_path("stdlib/Extensions/Service/tests/services/upper.jai");
    PROGRAM
        .replace("{UPPER}", &jacket_config::slashed(&upper))
        .replace("{REQUEST}", request)
}

#[test]
fn a_pending_answer_is_unknown_until_it_arrives() {
    let Some(jaic) = built_jaic() else {
        eprintln!("skipped: no jaic built next to the tests");
        return;
    };
    let dir =
        test_support::lsp::project("services-pending", &[("main.jai", &program("slow:late"))]);
    let mut s = session_with_services(jaic, &dir);
    let (uri, _) = test_support::lsp::open_from_disk(&mut s, &dir, "main.jai");
    // The service takes 300 ms (and starting it longer): the first check does not wait.
    assert_eq!(messages(&s, &uri), Vec::<String>::new());
    assert!(s.services_waiting());
    let deadline = Instant::now() + Duration::from_secs(60);
    while !s.take_service_arrivals() {
        assert!(Instant::now() < deadline, "the answer never arrived");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!s.services_waiting());
    let after = messages(&s, &uri);
    assert!(
        after.iter().any(|m| m.contains("the service answered")),
        "{after:?}"
    );
}

#[test]
fn an_unavailable_service_reports_nothing_in_the_editor() {
    let Some(jaic) = built_jaic() else {
        eprintln!("skipped: no jaic built next to the tests");
        return;
    };
    let dir = test_support::lsp::project(
        "services-gone",
        &[("main.jai", &program("gone:no database"))],
    );
    let mut s = session_with_services(jaic, &dir);
    let (uri, _) = test_support::lsp::open_from_disk(&mut s, &dir, "main.jai");
    let deadline = Instant::now() + Duration::from_secs(60);
    while s.services_waiting() || s.take_service_arrivals() {
        assert!(Instant::now() < deadline, "the answer never arrived");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(messages(&s, &uri), Vec::<String>::new());
}

#[test]
fn an_untrusted_workspace_never_asks() {
    let Some(jaic) = built_jaic() else {
        eprintln!("skipped: no jaic built next to the tests");
        return;
    };
    let dir = test_support::lsp::project("services-untrusted", &[("main.jai", &program("late"))]);
    let mut s = session_with_services(jaic, &dir);
    s.set_trusted(false);
    let (uri, _) = test_support::lsp::open_from_disk(&mut s, &dir, "main.jai");
    assert_eq!(messages(&s, &uri), Vec::<String>::new());
    assert!(!s.services_waiting());
}

const READS_PATH: &str = r#"#import "Basic";
libc :: #system_library "libc";
c_getenv :: (name: *u8) -> *u8 #foreign libc "getenv";
SEEN :: #run c_getenv("PATH") != null;
#assert !SEEN "the environment is visible";
main :: () {}
"#;

#[test]
fn trusted_compile_time_code_reads_the_real_environment() {
    let dir = test_support::lsp::project("services-getenv", &[("main.jai", READS_PATH)]);
    let mut s = test_support::lsp::workspace_session(&dir);
    let (uri, _) = test_support::lsp::open_from_disk(&mut s, &dir, "main.jai");
    let found = messages(&s, &uri);
    assert!(
        found
            .iter()
            .any(|m| m.contains("the environment is visible")),
        "{found:?}"
    );
}
