//! What `jacket dep` leaves in the repository, read back with git itself: the state must be the
//! one `git submodule` would have written, so plain git keeps working on the project.
use crate::dep_support;
use dep_support::{Origin, Project, git, run_in};

const MODULE: &[(&str, &str)] = &[("module.jai", "answer :: 42;\n")];

fn added() -> (Project, Origin) {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");
    let origin = Origin::new(&project.root, "jaison", MODULE);

    origin.release("v1.0.0", &[("module.jai", "answer :: 43;\n")]);
    let run = project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    (project, origin)
}

#[test]
fn an_added_dependency_is_a_submodule_git_agrees_with() {
    let (project, origin) = added();
    let status = git(&project.dir, &["submodule", "status"]);

    assert!(
        !status.starts_with(['-', '+', 'U']) && status.contains("modules/jaison"),
        "initialised and at the recorded commit: {status:?}"
    );
    let porcelain = git(&project.dir, &["status", "--porcelain"]);

    assert!(
        porcelain.contains("A  .gitmodules") && porcelain.contains("A  modules/jaison"),
        "staged: {porcelain}"
    );
    let modules = std::fs::read_to_string(project.dir.join(".gitmodules")).unwrap();

    assert!(
        modules.contains("[submodule \"modules/jaison\"]")
            && modules.contains("path = modules/jaison")
            && modules.contains(&format!("url = {}", origin.url())),
        "{modules}"
    );
    let sub = project.dir.join("modules/jaison");

    assert_eq!(git(&sub, &["status", "--porcelain"]), "");
    assert!(
        git(&sub, &["rev-parse", "--absolute-git-dir"]).ends_with(".git/modules/modules/jaison")
    );
    git(&sub, &["fsck", "--no-progress"]);
    assert_eq!(
        git(&project.dir, &["config", "submodule.modules/jaison.url"]),
        origin.url()
    );
}

#[test]
fn remove_leaves_git_with_nothing_of_the_submodule() {
    let (project, _) = added();
    let run = project.jacket(&["dep", "remove", "jaison"]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(git(&project.dir, &["submodule", "status"]), "");
    let modules = std::fs::read_to_string(project.dir.join(".gitmodules")).unwrap_or_default();

    assert!(!modules.contains("jaison"), "{modules}");
    assert!(!project.dir.join(".git/modules/modules/jaison").exists());
    assert!(
        !git(&project.dir, &["config", "--list", "--local"]).contains("submodule."),
        "the superproject's config forgets it"
    );
    assert!(!project.dir.join("modules/jaison").exists());
}

#[test]
fn sync_initialises_a_fresh_clone_as_git_would() {
    let (project, _) = added();

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
    let run = run_in(&clone, project.root.path(), &["dep", "sync"], &[]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let status = git(&clone, &["submodule", "status"]);

    assert!(!status.starts_with(['-', '+', 'U']), "{status:?}");
    assert_eq!(
        git(&clone.join("modules/jaison"), &["status", "--porcelain"]),
        ""
    );
}

#[test]
fn update_moves_the_checkout_and_stages_the_new_commit() {
    let (project, origin) = added();
    let next = origin.release("v1.1.0", &[("module.jai", "answer :: 44;\n")]);
    let run = project.jacket(&["dep", "update", "--to", "v1.1.0"]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let sub = project.dir.join("modules/jaison");

    assert_eq!(git(&sub, &["rev-parse", "HEAD"]), next);
    assert_eq!(
        std::fs::read_to_string(sub.join("module.jai")).unwrap(),
        "answer :: 44;\n"
    );
    assert_eq!(git(&sub, &["status", "--porcelain"]), "");
    assert!(git(&project.dir, &["ls-files", "-s"]).contains(&format!("160000 {next}")));
}

#[test]
fn an_edited_dependency_is_not_overwritten_by_an_update() {
    let (project, origin) = added();

    origin.release("v1.1.0", &[("module.jai", "answer :: 44;\n")]);
    let file = project.dir.join("modules/jaison/module.jai");

    std::fs::write(&file, "answer :: 0;\n").unwrap();
    let run = project.jacket(&["dep", "update", "--to", "v1.1.0"]);

    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "answer :: 0;\n");
}

#[test]
fn replacing_a_vendored_copy_needs_it_unmodified() {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");
    let origin = Origin::new(&project.root, "jaison", MODULE);

    origin.release("v1.0.0", &[]);
    std::fs::create_dir_all(project.dir.join("modules/jaison")).unwrap();
    std::fs::write(project.dir.join("modules/jaison/module.jai"), "old :: 1;\n").unwrap();
    git(&project.dir, &["add", "-A"]);
    git(&project.dir, &["commit", "-q", "-m", "vendored"]);
    std::fs::write(
        project.dir.join("modules/jaison/module.jai"),
        "edited :: 2;\n",
    )
    .unwrap();
    let spec = format!("{}@v1.0.0", origin.url());
    let run = project.jacket(&["dep", "add", &spec, "--name", "jaison"]);

    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.err.contains("uncommitted changes"), "{}", run.err);
    git(
        &project.dir,
        &["checkout", "--", "modules/jaison/module.jai"],
    );
    let run = project.jacket(&["dep", "add", &spec, "--name", "jaison"]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(git(&project.dir, &["ls-files", "-s"]).contains("160000"));
}

#[test]
fn a_package_that_names_itself_moves_its_submodule_with_git_still_agreeing() {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");
    let origin = Origin::new(
        &project.root,
        "jaison",
        &[
            ("module.jai", "answer :: 42;\n"),
            ("jacket.toml", "[package]\nname = \"json_lib\"\n"),
        ],
    );

    origin.release("v1.0.0", &[]);
    let run = project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(project.dir.join("modules/json_lib/module.jai").is_file());
    assert!(!project.dir.join("modules/jaison").exists());
    let modules = std::fs::read_to_string(project.dir.join(".gitmodules")).unwrap();

    assert!(modules.contains("path = modules/json_lib"), "{modules}");
    let status = git(&project.dir, &["submodule", "status"]);

    assert!(
        !status.starts_with(['-', '+', 'U']) && status.contains("modules/json_lib"),
        "{status:?}"
    );
    let sub = project.dir.join("modules/json_lib");

    assert_eq!(git(&sub, &["status", "--porcelain"]), "");
    assert!(git(&project.dir, &["ls-files", "-s"]).contains("modules/json_lib"));
    assert!(!git(&project.dir, &["ls-files", "-s"]).contains("modules/jaison"));
}
