//! Examples: the ```` ```jai ```` blocks of doc comments, shown as `jacket test` reads them and
//! run in the browser as the very program it builds for them.
use super::highlight;
use super::markdown::esc;
use super::model::Module;
use super::site::Site;
use super::view::Example;
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

fn template_json(t: &doctest::Template) -> serde_json::Value {
    serde_json::json!({
        "head": t.head, "middle": t.middle, "tail": t.tail,
        "wrapped": t.wrapped, "pad": t.pad_blank,
    })
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

/// A code block that is not an example: highlighted when it is Jai (or says nothing).
pub fn plain_code(site: &Site, module: &Module, info: &str, code: &str) -> Example {
    let lang = info.split([',', ' ']).next().unwrap_or("");
    let body = if lang.is_empty() || lang == "jai" {
        highlight::jai(code.trim_end(), &|name| site.call_link(module, name))
    } else {
        highlight::other(lang, code.trim_end()).unwrap_or_else(|| esc(code.trim_end()))
    };
    Example {
        plain: true,
        compact: false,
        code_html: body,
        id: None,
        note: None,
        playground_url: None,
    }
}

/// A fenced block of a doc comment: an example with buttons when it is ```` ```jai ````, else
/// plain code. `file` and `line` say where the comment is, for the program's header.
pub fn view(
    site: &Site,
    module: &Module,
    (file, line): (&str, usize),
    (info, code): (&str, &str),
    compact: bool,
) -> Example {
    let Some(block) = block(info, code, (file, line), &module.test_notes) else {
        return plain_code(site, module, info, code);
    };
    let shown = highlight::jai(code.trim_end(), &|name| site.call_link(module, name));
    let skipped = why_not_run(&block);
    let text = skipped
        .is_none()
        .then(|| doctest::generate(&block, &module.id).0);
    if let Some(text) = &text {
        site.add_runnable(
            &program_id(text),
            super::example_check::Program {
                label: format!("{}:{}", block.file.display(), block.line),
                text: text.clone(),
                timeout: block.options.timeout,
            },
        );
    }
    let note = skipped.or_else(|| (!site.runtime).then(|| NO_RUNTIME.to_owned()));
    let program = text.filter(|_| note.is_none()).map(|text| {
        let parts = serde_json::json!({
            "run": template_json(&doctest::generate_template(&block, &module.id)),
            "open": standalone_ok(&block, &module.id)
                .then(|| template_json(&doctest::standalone_template(&block, &module.id))),
            "playground": site.playground,
        });
        site.add_example(&text, parts.to_string())
    });
    let standalone = (standalone_ok(&block, &module.id) && !site.playground.is_empty())
        .then(|| doctest::standalone(&block, &module.id))
        .flatten()
        .map(|text| format!("{}#code={}", site.playground, base64url(text.as_bytes())));
    Example {
        plain: false,
        compact,
        code_html: shown,
        id: program,
        note,
        playground_url: standalone,
    }
}

/// The file name an example program is stored under: a digest of its text.
pub fn program_id(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().take(6).map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
