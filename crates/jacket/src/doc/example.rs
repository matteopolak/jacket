//! Examples: the ```` ```jai ```` blocks of doc comments, shown as `jacket test` reads them and
//! run in the browser as the very program it builds for them.
use super::model::Module;
use super::site::Site;
use crate::test::doctest::{self, Block, Mode};
use crate::test::options::Target;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Why an example is never run, if it is: the disabled Run button's hover.
fn why_not_run(block: &Block) -> Option<String> {
    if block.options.ignore {
        return Some("Not run: the example is marked ignore.".into());
    }
    match block.mode {
        Mode::NoRun => return Some("This one is only checked to compile, not run.".into()),
        Mode::CompileFail => return Some("This one is meant not to compile.".into()),
        Mode::ShouldFail => {
            return Some("This one is meant to fail, so there is nothing to run.".into());
        }
        Mode::Run => {}
    }
    if let Some(reason) = browser_skip(block) {
        return Some(format!("Doesn't run in the browser ({reason})."));
    }
    None
}

const NO_RUNTIME: &str = "This copy of the docs has no browser compiler, so it can't run examples.";

/// Why `jacket test` would skip the block on WebAssembly, which is what the browser runs.
fn browser_skip(block: &Block) -> Option<String> {
    let target = Target {
        os: "wasm".into(),
        cpu: "wasm".into(),
        interp: true,
    };
    block.options.skip_reason(&target, &[])
}

/// Base64 with the URL alphabet and no padding, for the playground's `#code=`.
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for k in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * k) & 63) as usize] as char);
        }
    }
    out
}

/// Whether the playground can show the example on its own.
fn standalone_ok(block: &Block, module: &str) -> bool {
    browser_skip(block).is_none()
        && matches!(block.mode, Mode::Run | Mode::NoRun)
        && doctest::standalone(block, module).is_some()
}

/// The block a fence describes, or `None` when the fence is not Jai.
fn block(info: &str, code: &str, (file, line): (&str, usize), notes: &[String]) -> Option<Block> {
    let mut lines = vec![format!("```{info}")];
    lines.extend(code.lines().map(str::to_owned));
    lines.push("```".into());
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let (_, mut found) = doctest::fenced(&refs).into_iter().next()?;
    found.file = PathBuf::from(format!("stdlib/{file}"));
    found.line = u32::try_from(line).unwrap_or(0);
    for note in notes {
        // A note that does not apply is `jacket test`'s to warn about.
        let _ = found.options.apply(note);
    }
    Some(found)
}

/// Example data for static builders. Extraction records eligibility and programs without executing them.
pub(super) fn exported(
    site: &Site,
    module: &Module,
    (file, line): (&str, usize),
    (info, code): (&str, &str),
    source: super::export::contract::ResolvedText,
) -> super::export::contract::Example {
    use super::export::contract::{Capability, Example as Data, Location, ProgramTemplate};
    let make_template = |t: doctest::Template| ProgramTemplate {
        head: t.head,
        middle: t.middle,
        tail: t.tail,
        wrapped: t.wrapped,
        pad: t.pad_blank,
    };
    let parsed = block(info, code, (file, line), &module.test_notes);
    let reason = parsed
        .as_ref()
        .and_then(why_not_run)
        .or_else(|| (parsed.is_some() && !site.runtime).then(|| NO_RUNTIME.to_owned()));
    let program = parsed
        .as_ref()
        .filter(|b| why_not_run(b).is_none())
        .map(|b| doctest::generate(b, &module.id).0);
    if let (Some(program), Some(block)) = (&program, &parsed) {
        site.add_runnable(super::example_check::Program {
            label: format!("{}:{}", block.file.display(), block.line),
            text: program.clone(),
            timeout: block.options.timeout,
        });
    }
    let run_template = parsed
        .as_ref()
        .filter(|_| reason.is_none())
        .map(|b| make_template(doctest::generate_template(b, &module.id)));
    let open_template = parsed
        .as_ref()
        .filter(|b| standalone_ok(b, &module.id))
        .map(|b| make_template(doctest::standalone_template(b, &module.id)));
    let playground_url = parsed
        .as_ref()
        .filter(|b| standalone_ok(b, &module.id) && !site.playground.is_empty())
        .and_then(|b| doctest::standalone(b, &module.id))
        .map(|text| format!("{}#code={}", site.playground, base64url(text.as_bytes())));
    let mut example = Data {
        id: String::new(),
        language: info.split([',', ' ']).next().unwrap_or("").into(),
        source,
        fence: info.into(),
        location: Location {
            file: file.into(),
            line: line as u64,
        },
        runnable: parsed.is_some() && reason.is_none(),
        reason,
        program,
        run_template,
        open_template,
        playground_url,
        timeout_ms: parsed
            .as_ref()
            .and_then(|b| b.options.timeout)
            .unwrap_or(60)
            .saturating_mul(1_000),
        capabilities: if parsed.is_some() {
            vec![Capability::BrowserRuntime, Capability::EditableExamples]
        } else {
            Vec::new()
        },
    };
    // Asset identity includes timeout, source links and templates as well as the executable.
    // The empty ID avoids hashing itself; execution checks separately key exact program and budget.
    example.id =
        asset_id(&serde_json::to_string(&example).expect("serialize documentation example"));
    example
}

/// A browser asset ID hashes the complete serialized example, including its budget.
/// For example, otherwise identical one- and two-second examples have distinct IDs.
fn asset_id(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().take(6).map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_programs_keep_distinct_example_payload_identity() {
        use super::super::export::contract::ResolvedText;
        let site = Site::new(
            vec![],
            vec![],
            "/".into(),
            String::new(),
            String::new(),
            String::new(),
            true,
        );
        let module = Module {
            id: "Tiny".into(),
            wasm: true,
            ..Module::default()
        };
        let source = || ResolvedText {
            source: "value := 1;".into(),
            spans: vec![],
        };
        let first = exported(
            &site,
            &module,
            ("first.jai", 3),
            ("jai,timeout(1)", "value := 1;"),
            source(),
        );
        let repeated = exported(
            &site,
            &module,
            ("first.jai", 3),
            ("jai,timeout(2)", "value := 1;"),
            source(),
        );
        assert_eq!((first.timeout_ms, repeated.timeout_ms), (1_000, 2_000));
        assert_eq!(
            first.program, repeated.program,
            "execution cache may share identical programs"
        );
        assert_ne!(
            first.id, repeated.id,
            "different execution timeouts must not collide in example assets"
        );
        let same = exported(
            &site,
            &module,
            ("first.jai", 3),
            ("jai,timeout(1)", "value := 1;"),
            source(),
        );
        assert_eq!(
            first.id, same.id,
            "identical complete payloads retain stable assets"
        );
    }

    #[test]
    fn example_validation_keeps_distinct_budgets_for_identical_programs() {
        use super::super::export::contract::ResolvedText;
        let site = Site::new(
            vec![],
            vec![],
            "/".into(),
            String::new(),
            String::new(),
            String::new(),
            true,
        );
        let module = Module {
            id: "Tiny".into(),
            wasm: true,
            ..Module::default()
        };
        for info in ["jai,timeout(1)", "jai,timeout(2)", "jai,timeout(1)"] {
            exported(
                &site,
                &module,
                ("example.jai", 3),
                (info, "value := 1;"),
                ResolvedText {
                    source: "value := 1;".into(),
                    spans: vec![],
                },
            );
        }
        let mut budgets = site
            .runnable()
            .into_iter()
            .map(|program| program.timeout)
            .collect::<Vec<_>>();
        budgets.sort_unstable();
        assert_eq!(
            budgets,
            [Some(1), Some(2)],
            "only identical execution program and budget may share a validation job"
        );
    }

    #[test]
    fn base64_url_has_no_padding() {
        assert_eq!(base64url(b"hi"), "aGk");
        assert_eq!(base64url(b"\xfb\xff"), "-_8");
        assert_eq!(base64url(b"abc"), "YWJj");
    }

    #[test]
    fn fence_options_decide_the_browser() {
        let b = block("jai,skip_on(wasm)", "x := 1;\n", ("a.jai", 3), &[]).unwrap();
        assert!(browser_skip(&b).is_some());
        let b = block("jai", "x := 1;\n", ("a.jai", 3), &[]).unwrap();
        assert!(browser_skip(&b).is_none());
        let notes = [r#"skip_on(.WASM, reason = "no threads")"#.to_owned()];
        let b = block("jai", "x := 1;\n", ("a.jai", 3), &notes).unwrap();
        assert!(browser_skip(&b).is_some());
        assert!(block("text", "x", ("a.jai", 3), &[]).is_none());
    }
}
