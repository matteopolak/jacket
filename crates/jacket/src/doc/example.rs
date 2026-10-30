//! Examples: the ```` ```jai ```` blocks of doc comments, shown as `jacket test` reads them and
//! run in the browser as the very program it builds for them.
use super::highlight;
use super::markdown::esc;
use super::model::Module;
use super::site::Site;
use crate::test::doctest::{self, Block, Mode};
use crate::test::options::Target;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Why an example is shown without a working Run button, if it is.
fn why_not_run(site: &Site, block: &Block) -> Option<String> {
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
    if !site.runtime {
        return Some(
            "This copy of the docs has no browser compiler, so it can't run examples.".into(),
        );
    }
    None
}

/// Why `jacket test` would skip the block on WebAssembly, which is what the browser runs.
fn browser_skip(block: &Block) -> Option<String> {
    let target = Target {
        os: "wasm".into(),
        cpu: "x64".into(),
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

/// The block a fence describes, or `None` when the fence is not Jai.
fn block(info: &str, code: &str, file: &str, line: usize) -> Option<Block> {
    let mut lines = vec![format!("```{info}")];
    lines.extend(code.lines().map(str::to_owned));
    lines.push("```".into());
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let (_, mut found) = doctest::fenced(&refs).into_iter().next()?;
    found.file = PathBuf::from(format!("stdlib/{file}"));
    found.line = u32::try_from(line).unwrap_or(0);
    Some(found)
}

/// A code block that is not an example: highlighted when it is Jai (or says nothing).
pub fn plain_code(site: &Site, module: &Module, info: &str, code: &str) -> String {
    let lang = info.split([',', ' ']).next().unwrap_or("");
    let body = if lang.is_empty() || lang == "jai" {
        highlight::jai(code.trim_end(), &|name| site.call_link(module, name))
    } else {
        esc(code.trim_end())
    };
    format!("<pre class=\"code\"><code>{body}</code></pre>")
}

/// A fenced block of a doc comment: an example with buttons when it is ```` ```jai ````, else
/// plain code. `file` and `line` say where the comment is, for the program's header.
pub fn fence(
    site: &Site,
    module: &Module,
    (file, line): (&str, usize),
    (info, code): (&str, &str),
    compact: bool,
) -> String {
    let Some(block) = block(info, code, file, line) else {
        return plain_code(site, module, info, code);
    };
    let shown = highlight::jai(code.trim_end(), &|name| site.call_link(module, name));
    let note = why_not_run(site, &block);
    let program = (note.is_none()).then(|| {
        let text = doctest::generate(&block, &module.id).0;
        site.add_example(&text)
    });
    let standalone = (browser_skip(&block).is_none()
        && matches!(block.mode, Mode::Run | Mode::NoRun))
    .then(|| doctest::standalone(&block, &module.id))
    .flatten()
    .map(|text| format!("{}#code={}", site.playground, base64url(text.as_bytes())));
    let open = standalone.map_or_else(String::new, |href| {
        if compact {
            format!("<a class=\"btn sm\" href=\"{}\" aria-label=\"Open this example in the playground\" title=\"Open in the playground\">↗</a>", esc(&href))
        } else {
            format!("<a class=\"btn\" href=\"{}\">Open in playground ↗</a>", esc(&href))
        }
    });
    let run = match (&program, &note) {
        (Some(id), _) => format!(
            "<button type=\"button\" class=\"btn {} js-only\" data-act=\"run\" data-program=\"{id}\">Run ▸</button>",
            if compact {
                "sm"
            } else {
                "primary"
            }
        ),
        (None, Some(why)) => format!(
            "<button type=\"button\" class=\"btn {} js-only\" disabled title=\"{}\">Run ▸</button>",
            if compact {
                "sm"
            } else {
                "primary"
            },
            esc(why)
        ),
        (None, None) => String::new(),
    };
    let note_html = note
        .filter(|_| program.is_none())
        .map_or_else(String::new, |why| {
            format!("<p class=\"ex-note\">{}</p>", esc(&why))
        });
    if compact {
        format!(
            "<figure class=\"ex compact\"><div class=\"ex-row\"><pre class=\"code\"><code>{shown}</code></pre>{open}{run}</div>{note_html}<div class=\"ex-out\" hidden></div></figure>"
        )
    } else {
        format!(
            "<figure class=\"ex\"><div class=\"ex-bar\"><span class=\"ex-title\">Example</span><span class=\"spacer\"></span><button type=\"button\" class=\"btn js-only\" data-act=\"copy\">Copy</button>{open}{run}</div><pre class=\"code\"><code>{shown}</code></pre>{note_html}<div class=\"ex-out\" hidden></div></figure>"
        )
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
        let b = block("jai,skip_on(wasm)", "x := 1;\n", "a.jai", 3).unwrap();
        assert!(browser_skip(&b).is_some());
        let b = block("jai", "x := 1;\n", "a.jai", 3).unwrap();
        assert!(browser_skip(&b).is_none());
        assert!(block("text", "x", "a.jai", 3).is_none());
    }
}
