//! Whole-pointer releases whose inferred must effect matches the removed contract.
use super::*;

const CARRIERS: &[(&str, &str)] = &[
    ("internal/ipc/channel.jai", "release_registration"),
    ("internal/thread/lifecycle.jai", "release_thread_state"),
    ("internal/bindings/native/module.jai", "release"),
    ("Extensions/Regex/program.jai", "free_trie"),
    ("Extensions/Http/test_client.jai", "free_test_client"),
    ("Extensions/Io/ownership.jai", "release_owned"),
    ("Extensions/Tls/config.jai", "release_native"),
    ("Extensions/Tls/stream.jai", "release_stream"),
    ("Extensions/Io/resolver.jai", "free_address_info"),
    ("internal/media/mpeg/audio.jai", "free_private"),
    ("Extensions/Io/sync.jai", "free_channel"),
    (
        "internal/developer-tools/performance-report.jai",
        "release_report",
    ),
    ("internal/bindings/arrays/module.jai", "release_values"),
];

#[test]
fn whole_pointer_cleanup_contracts_match_checked_bodies_on_host_and_windows() {
    for windows in [false, true] {
        let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../stdlib")
            .canonicalize()
            .unwrap();
        let mut options = Options::host();
        options.import_paths = vec![stdlib.clone()];
        options.set_stdlib(&NativeFs, &stdlib);
        if windows {
            options.os = jaic::target::TargetOs::Windows;
            options.cpu = jaic::target::TargetCpu::X64;
        }
        let mut c = Compiler::new(options, Rc::new(NativeFs));
        let mut facts = IdeFacts::new(vec![IdeRoot::Virtual, IdeRoot::under(&stdlib)]);
        facts.lint = true;
        c.ed.record(facts);
        c.begin_sources(&[ProgramSource::String(
            r#"
            #import "Basic";
            S :: #import "Shared_Memory_Channel";
            T :: #import "Thread";
            N :: #import "GL";
            R :: #import "Extensions/Regex";
            H :: #import "Extensions/Http";
            Io :: #import "Extensions/Io";
            L :: #import "Extensions/Tls";
            M :: #import "pl_mpeg";
            P :: #import "Performance_Report";
            A :: #import "Vulkan";
            main :: () {
                Io.io_run(() {
                    channel := Io.make_channel(int, 1);
                    Io.free_channel(channel);
                });
            }
        "#
            .into(),
        )])
        .unwrap();
        c.finish_program().unwrap();
        let diagnostics = c.ide_check_all();
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let mut ids = Vec::new();
        let mut seen = vec![0; CARRIERS.len()];
        for (i, p) in c.pr.procs.iter().enumerate() {
            if p.is_poly || p.body_state != jaic::sema::procs::BodyState::Done {
                continue;
            }
            let path = c.src.sources.get(p.lit.header.span.file).path.to_string();
            if let Some(index) = CARRIERS
                .iter()
                .position(|(file, name)| path.ends_with(file) && p.name.as_str() == *name)
            {
                seen[index] += 1;
                ids.push((ProcId(i as u32), index));
            }
        }
        for (i, count) in seen.iter().enumerate() {
            assert!(*count > 0, "unchecked {:?}, windows={windows}", CARRIERS[i]);
        }
        without_body_notes(&mut c);
        c.release_lowered_code();
        let all = Summaries::build(&c);
        for (id, index) in ids {
            let summary = &all.procs[&id];
            assert!(
                summary.guaranteed.keys().any(|e| e.kind == Kind::Free
                    && e.guard.is_none()
                    && e.target.root == Root::Parameter(0)
                    && e.target.steps.is_empty()),
                "{:?}, windows={windows}: {summary:?}",
                CARRIERS[index]
            );
        }
    }
}
