//! `jacket dep` with tags and commits, against local bare repositories.
mod dep_support;
use dep_support::{Origin, Project, git, run_in};

const MODULE: &[(&str, &str)] = &[("module.jai", "answer :: 42;\n")];

fn one_tag() -> (Project, Origin, String) {
    let project = Project::new("[project]\nentry = [\"main.jai\"]\n");
    let origin = Origin::new(&project.root, "jaison", MODULE);
    let commit = origin.release("v1.0.0", &[("module.jai", "answer :: 43;\n")]);
    (project, origin, commit)
}

#[test]
fn add_by_tag_checks_out_the_commit_and_records_the_entry_without_committing() {
    let (project, origin, commit) = one_tag();
    origin.release("v1.1.0", &[("module.jai", "answer :: 44;\n")]);
    let before = project.commit_count();
    let run = project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(
        std::fs::read_to_string(project.dir.join("modules/jaison/module.jai")).unwrap(),
        "answer :: 43;\n"
    );
    assert_eq!(
        git(&project.dir.join("modules/jaison"), &["rev-parse", "HEAD"]),
        commit
    );
    let toml = project.toml();
    assert!(
        toml.contains(&format!(
            "jaison = {{ git = \"{}\", tag = \"v1.0.0\" }}",
            origin.url()
        )),
        "{toml}"
    );
    assert_eq!(project.commit_count(), before, "dep never commits");
    assert!(git(&project.dir, &["ls-files", "-s"]).contains(&format!("160000 {commit}")));
}

#[test]
fn add_without_a_version_pins_the_default_branch_by_rev() {
    let (project, origin, _) = one_tag();
    let tip = origin.commit("more");
    let run = project.jacket(&["dep", "add", &origin.url(), "--name", "j"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        project.toml().contains(&format!("rev = \"{}\"", &tip[..7])),
        "{}",
        project.toml()
    );
    assert_eq!(
        git(&project.dir.join("modules/j"), &["rev-parse", "HEAD"]),
        tip
    );
}

#[test]
fn add_refuses_an_unknown_tag_and_a_folder_that_is_not_a_repository() {
    let (project, origin, _) = one_tag();
    let run = project.jacket(&["dep", "add", &format!("{}@v9", origin.url())]);
    assert_eq!(run.code, 2);
    assert!(run.err.contains("has no tag `v9`"), "{}", run.err);
    assert!(!project.dir.join("modules").exists());
    let bare = Project::new("");
    std::fs::remove_dir_all(bare.dir.join(".git")).unwrap();
    let run = run_in(&bare.dir, &bare.root, &["dep", "check"], &[]);
    assert_eq!(run.code, 2);
    assert!(run.err.contains("git init"), "{}", run.err);
}

#[test]
fn remove_deinits_and_deletes_folder_and_entry() {
    let (project, origin, _) = one_tag();
    project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);
    let run = project.jacket(&["dep", "remove", "jaison"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(!project.dir.join("modules/jaison").exists());
    assert!(!project.toml().contains("jaison"));
    assert_eq!(
        git(&project.dir, &["ls-files", "-s"])
            .matches("160000")
            .count(),
        0
    );
    // Adding it again works: nothing was left behind.
    let run = project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(project.jacket(&["dep", "remove", "nope"]).code, 2);
}

#[test]
fn list_names_the_resolved_tag_and_commit() {
    let (project, origin, commit) = one_tag();
    project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);
    let run = project.jacket(&["dep", "list"]);
    assert_eq!(run.code, 0, "{}", run.err);
    let row = run.out.lines().find(|l| l.starts_with("jaison")).unwrap();
    assert!(
        row.contains("v1.0.0") && row.contains(&commit[..7]) && row.contains("modules/jaison"),
        "{}",
        run.out
    );
}

#[test]
fn a_fresh_clone_needs_sync_and_check_says_so() {
    let (project, origin, _) = one_tag();
    project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);
    git(&project.dir, &["add", "-A"]);
    git(&project.dir, &["commit", "-q", "-m", "deps"]);
    let clone = project.root.join("clone");
    git(
        &project.root,
        &[
            "clone",
            "-q",
            project.dir.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let env_root = project.root.path();
    let run = run_in(&clone, env_root, &["dep", "check"], &[]);
    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
    assert!(run.err.contains("jacket dep sync"), "{}", run.err);
    let run = run_in(&clone, env_root, &["dep", "sync"], &[]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(clone.join("modules/jaison/module.jai").is_file());
    let run = run_in(&clone, env_root, &["dep", "check"], &[]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
}

#[test]
fn a_moved_tag_fails_check_and_update_naming_both_commits() {
    let (project, origin, pinned) = one_tag();
    project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);
    let moved = origin.commit("rewritten");
    origin.tag("v1.0.0");
    let run = project.jacket(&["dep", "check"]);
    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
    assert!(
        run.err.contains("tag v1.0.0 of jaison has moved")
            && run.err.contains(&moved[..7])
            && run.err.contains(&pinned[..7]),
        "{}",
        run.err
    );
    assert_eq!(project.jacket(&["dep", "check", "--offline"]).code, 0);
    let run = project.jacket(&["dep", "update", "jaison"]);
    assert_eq!(run.code, 2);
    assert!(run.err.contains("has moved"), "{}", run.err);
}

#[test]
fn update_moves_a_rev_entry_to_the_newest_commit_and_a_tag_with_to() {
    let (project, origin, _) = one_tag();
    let first = origin.commit("one");
    project.jacket(&["dep", "add", &origin.url(), "--name", "j"]);
    let tip = origin.commit("two");
    let run = project.jacket(&["dep", "update", "j"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(
        git(&project.dir.join("modules/j"), &["rev-parse", "HEAD"]),
        tip
    );
    assert!(
        project.toml().contains(&format!("rev = \"{}\"", &tip[..7]))
            && !project.toml().contains(&first[..7])
    );
    assert!(
        project
            .jacket(&["dep", "update", "j"])
            .out
            .contains("up to date")
    );
    let run = project.jacket(&["dep", "update", "j", "--to", "v1.0.0"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        project.toml().contains("tag = \"v1.0.0\"") && !project.toml().contains("rev ="),
        "{}",
        project.toml()
    );
}

#[test]
fn nested_submodules_are_fetched_with_the_dependency() {
    let project = Project::new("");
    let inner = Origin::new(&project.root, "inner", &[("module.jai", "inner :: 1;\n")]);
    let outer = Origin::new(&project.root, "outer", MODULE);
    git(
        &outer.work,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &inner.url(),
            "modules/inner",
        ],
    );
    outer.release("v1.0.0", &[]);
    let run = project.jacket(&["dep", "add", &format!("{}@v1.0.0", outer.url())]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        project
            .dir
            .join("modules/outer/modules/inner/module.jai")
            .is_file()
    );
}

#[test]
fn init_adopts_submodules_with_the_tag_at_their_commit_and_lists_vendored_copies() {
    let (project, origin, _) = one_tag();
    git(
        &project.dir,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &origin.url(),
            "modules/jaison",
        ],
    );
    git(
        &project.dir.join("modules/jaison"),
        &["checkout", "-q", "v1.0.0"],
    );
    std::fs::create_dir_all(project.dir.join("modules/Copied")).unwrap();
    std::fs::write(project.dir.join("modules/Copied/module.jai"), "x :: 1;\n").unwrap();
    let run = project.jacket(&["dep", "init"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        run.out.contains("vendored, not managed: modules/Copied"),
        "{}",
        run.out
    );
    let toml = project.toml();
    assert!(
        toml.contains("jaison = { git = ") && toml.contains("tag = \"v1.0.0\""),
        "{toml}"
    );
    assert!(
        toml.contains("[resolver]") && toml.contains("seed = \""),
        "{toml}"
    );
    assert_eq!(project.jacket(&["dep", "check", "--offline"]).code, 0);
    let seed = toml
        .lines()
        .find(|l| l.starts_with("seed"))
        .unwrap()
        .to_string();
    project.jacket(&["dep", "init"]);
    assert!(project.toml().contains(&seed), "the seed is kept");
    project.jacket(&["dep", "init", "--reseed"]);
    assert!(!project.toml().contains(&seed), "--reseed writes a new one");
}

#[test]
fn check_reports_a_submodule_that_jacket_toml_does_not_list() {
    let (project, origin, _) = one_tag();
    git(
        &project.dir,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &origin.url(),
            "modules/jaison",
        ],
    );
    let run = project.jacket(&["dep", "check"]);
    assert_eq!(run.code, 1);
    assert!(run.err.contains("jacket dep init"), "{}", run.err);
}
