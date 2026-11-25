use super::*;

#[test]
fn a_block_is_commands_each_followed_by_its_output() {
    let md = "# Title\n\n```console cwd=../examples/x\n$ jacket test\n  PASS one\n\n$ cat a.txt\nhi\n```\n";
    let blocks = blocks(md).unwrap();
    assert_eq!(blocks.len(), 1);
    let b = &blocks[0];
    assert_eq!((b.line, b.cwd.as_deref()), (3, Some("../examples/x")));
    assert_eq!(
        b.runs,
        vec![
            Run {
                command: "jacket test".into(),
                output: "  PASS one".into(),
            },
            Run {
                command: "cat a.txt".into(),
                output: "hi".into(),
            },
        ]
    );
    assert_eq!(
        &md[b.body.clone()],
        "$ jacket test\n  PASS one\n\n$ cat a.txt\nhi\n"
    );
}

#[test]
fn other_fences_are_not_console_blocks() {
    assert!(
        blocks("```sh\n$ jacket test\n```\n```text\nout\n```\n")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_block_must_start_with_a_command_and_take_only_cwd_and_cache() {
    let err = blocks("\n```console\noutput first\n```\n").unwrap_err();
    assert!(
        err.starts_with("line 2: a console block starts with a command"),
        "{err}"
    );
    assert!(blocks("```console cwd=x cache\n$ x\n```\n").unwrap()[0].cache);
    let err = blocks("```console,shell=bash\n$ x\n```\n").unwrap_err();
    assert!(
        err.contains("unknown console setting `shell=bash`"),
        "{err}"
    );
}

#[test]
fn the_text_of_a_block_round_trips() {
    let text = "$ jacket test\n  PASS one\n$ cat a\n";
    assert_eq!(body(&runs(text).unwrap()), text);
}

#[test]
fn commands_split_like_a_shell_but_without_one() {
    assert_eq!(
        words("jacket test --exact 'slugifies/A--B,a-b' \"two words\"").unwrap(),
        [
            "jacket",
            "test",
            "--exact",
            "slugifies/A--B,a-b",
            "two words"
        ]
    );
    assert_eq!(words("jacket test ''").unwrap(), ["jacket", "test", ""]);
    assert!(
        words("jacket test | head")
            .unwrap_err()
            .contains("without a shell")
    );
    assert!(words("jacket 'open").unwrap_err().contains("unclosed"));
}

#[test]
fn escapes_are_stripped_and_times_masked() {
    assert_eq!(strip("\x1b[1;32m    PASS\x1b[0m one"), "    PASS one");
    assert_eq!(
        normal("PASS [   0.057s] a  \r\nSummary [  12ms] b\n\n"),
        "PASS [time] a\nSummary [time] b"
    );
    assert_ne!(normal("got 1"), normal("got 2"));
}

#[test]
fn colours_become_spans_and_nothing_is_guessed() {
    assert_eq!(
        ansi_html("\x1b[1;32m    PASS\x1b[0m a <b>"),
        "<span class=\"ansi-bold ansi-green\">    PASS</span> a &lt;b&gt;"
    );
    assert_eq!(
        ansi_html("\x1b[2m28 |\x1b[0m x \x1b[91mE\x1b[39m"),
        "<span class=\"ansi-dim\">28 |</span> x <span class=\"ansi-bright-red\">E</span>"
    );
    // Text that looks like a status is plain unless the program coloured it.
    assert_eq!(ansi_html("FAIL error"), "FAIL error");
}

/// A guide with one block that runs `cat` in a folder beside it.
fn guide(dir: &Path, shown: &str) -> PathBuf {
    std::fs::create_dir_all(dir.join("ex")).unwrap();
    std::fs::write(dir.join("ex/a.txt"), "\x1b[31mred\x1b[0m\n").unwrap();
    let path = dir.join("guide.md");
    std::fs::write(
        &path,
        format!("Intro\n\n```console cwd=ex\n$ cat a.txt\n{shown}\n```\n"),
    )
    .unwrap();
    path
}

fn tools() -> Tools<'static> {
    Tools {
        jacket: Path::new("jacket"),
        jaic: Path::new("jaic"),
        stdlib: Path::new("stdlib"),
    }
}

#[test]
fn a_changed_output_fails_with_a_diff_and_accept_writes_it_and_its_colours() {
    let dir = test_support::TempDir::new("doc-console");
    let path = guide(&dir, "blue");
    let problems = check(&path, "guide.md", &tools(), false).unwrap();
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("guide.md:3: `cat a.txt` printed something else:"),
        "{}",
        problems[0]
    );
    assert!(problems[0].contains("-blue\n+red\n"), "{}", problems[0]);

    assert!(check(&path, "guide.md", &tools(), true).unwrap().is_empty());
    let md = std::fs::read_to_string(&path).unwrap();
    assert_eq!(md, "Intro\n\n```console cwd=ex\n$ cat a.txt\nred\n```\n");
    let capture = std::fs::read_to_string(capture_path(&path)).unwrap();
    assert!(capture.contains(r"\u001b[31mred\u001b[0m"), "{capture}");
    assert!(
        check(&path, "guide.md", &tools(), false)
            .unwrap()
            .is_empty()
    );

    let html = rendered(&path, &md);
    assert_eq!(
        html.get("$ cat a.txt\nred\n").map(String::as_str),
        Some("<span class=\"prompt\">$ </span>cat a.txt\n<span class=\"ansi-red\">red</span>")
    );
}

#[test]
fn a_cached_block_is_not_run_while_its_capture_matches() {
    let dir = test_support::TempDir::new("doc-console");
    let path = dir.join("guide.md");
    std::fs::create_dir_all(dir.join("ex")).unwrap();
    std::fs::write(dir.join("ex/a.txt"), "\x1b[31mred\x1b[0m\n").unwrap();
    std::fs::write(&path, "```console cwd=ex cache\n$ cat a.txt\nred\n```\n").unwrap();
    assert!(check(&path, "guide.md", &tools(), true).unwrap().is_empty());
    let capture = std::fs::read_to_string(capture_path(&path)).unwrap();

    // What the command prints now no longer matters: the capture stands for it.
    std::fs::write(dir.join("ex/a.txt"), "blue\n").unwrap();
    assert!(
        check(&path, "guide.md", &tools(), false)
            .unwrap()
            .is_empty()
    );
    assert!(check(&path, "guide.md", &tools(), true).unwrap().is_empty());
    assert_eq!(
        std::fs::read_to_string(capture_path(&path)).unwrap(),
        capture
    );

    // Edited text has no capture, so the command runs again.
    std::fs::write(&path, "```console cwd=ex cache\n$ cat a.txt\nred!\n```\n").unwrap();
    let problems = check(&path, "guide.md", &tools(), false).unwrap();
    assert!(problems[0].contains("-red!\n+blue\n"), "{problems:?}");
}

#[test]
fn matching_text_without_its_colours_fails_until_accepted() {
    let dir = test_support::TempDir::new("doc-console");
    let path = guide(&dir, "red");
    let problems = check(&path, "guide.md", &tools(), false).unwrap();
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("colours in"), "{}", problems[0]);
    assert!(rendered(&path, &std::fs::read_to_string(&path).unwrap()).is_empty());
}

#[test]
fn a_capture_without_blocks_is_stale_and_accept_removes_it() {
    let dir = test_support::TempDir::new("doc-console");
    let path = dir.join("guide.md");
    std::fs::write(&path, "No blocks.\n").unwrap();
    std::fs::write(capture_path(&path), "{\"blocks\": []}\n").unwrap();
    assert_eq!(check(&path, "guide.md", &tools(), false).unwrap().len(), 1);
    assert!(check(&path, "guide.md", &tools(), true).unwrap().is_empty());
    assert!(!capture_path(&path).exists());
}

/// A block that runs `script` with `sh` standing in for `jacket`.
#[cfg(unix)]
fn shell_block(script: &str) -> Block {
    Block {
        line: 1,
        cwd: None,
        cache: false,
        body: 0..0,
        runs: vec![Run {
            command: format!("jacket -c '{script}'"),
            output: String::new(),
        }],
    }
}

#[cfg(unix)]
#[test]
fn files_a_block_makes_are_removed_and_changing_one_is_an_error() {
    let dir = test_support::TempDir::new("doc-console");
    std::fs::write(dir.join("keep.txt"), "x").unwrap();
    let sh = Tools {
        jacket: Path::new("/bin/sh"),
        ..tools()
    };
    let block = shell_block(
        "mkdir -p made/inner && echo y > made/inner/new.txt && echo out && echo err >&2",
    );
    let runs = run_block(&block, &dir, &sh).unwrap();
    assert_eq!(runs[0].output, "out\nerr");
    assert!(!dir.join("made").exists());
    assert!(dir.join("keep.txt").exists());

    let err = run_block(&shell_block("echo more >> keep.txt"), &dir, &sh).unwrap_err();
    assert!(
        err.contains("keep.txt; a console block must leave its folder as it found it"),
        "{err}"
    );
}

#[test]
fn only_jacket_jaic_and_cat_run() {
    let dir = test_support::TempDir::new("doc-console");
    let err = run_command("rm -rf x", &dir, &tools()).unwrap_err();
    assert!(
        err.contains("console blocks run `jacket`, `jaic` and `cat`"),
        "{err}"
    );
}
