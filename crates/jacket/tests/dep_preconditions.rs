//! What `jacket dep` does when git is missing or a remote cannot be read: it stops before
//! changing anything and says what to do.
use dep_support::{Origin, Project};

use crate::dep_support;

const MODULE: &[(&str, &str)] = &[("module.jai", "answer :: 42;\n")];

#[test]
fn a_machine_without_git_stops_with_how_to_install_it() {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");
    let nowhere = project.root.join("nowhere");

    std::fs::create_dir_all(&nowhere).unwrap();
    let before = project.toml();

    for args in [&["dep", "sync"][..], &["dep", "add", "a/b@v1"]] {
        let run = project.jacket_env(args, &[("PATH", nowhere.to_str().unwrap())]);

        assert_eq!(run.code, 2, "{}{}", run.out, run.err);
        assert!(
            run.err
                .starts_with("error: `git` is not installed, and `jacket dep` needs it\nnote: "),
            "{}",
            run.err
        );
        assert!(
            run.err.contains("install") || run.err.contains("git-scm.com"),
            "{}",
            run.err
        );
        assert!(!run.err.contains("could not run"), "{}", run.err);
    }

    assert_eq!(project.toml(), before);
    assert!(!project.dir.join("modules").exists());
}

#[test]
fn an_unreachable_remote_names_the_address_and_changes_nothing() {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");
    let before = project.toml();
    let url = "http://127.0.0.1:1/someone/lib.git";
    let run = project.jacket(&["dep", "add", &format!("{url}@v1")]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err.starts_with("error: could not reach "),
        "{}",
        run.err
    );
    assert!(run.err.contains(url), "{}", run.err);
    assert!(run.err.contains("note: check the address"), "{}", run.err);
    assert_eq!(project.toml(), before);
    assert!(!project.dir.join("modules").exists());
    assert_eq!(project.commit_count(), 1);
}

#[test]
fn a_release_age_policy_without_curl_stops_with_how_to_install_it() {
    let project =
        Project::new("[project]\nentry = [\"main.jai\"]\n\n[resolver]\nmin-release-age = \"3d\"\n");
    let git = std::process::Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .unwrap();
    let git_dir = std::path::Path::new(String::from_utf8_lossy(&git.stdout).trim())
        .parent()
        .unwrap()
        .to_path_buf();
    // A folder with git and nothing else, so only curl is missing.
    let only_git = project.root.join("only-git");

    std::fs::create_dir_all(&only_git).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(git_dir.join("git"), only_git.join("git")).unwrap();
    let before = project.toml();
    let run = project.jacket_env(
        &["dep", "add", "a/b@v1"],
        &[("PATH", only_git.to_str().unwrap())],
    );

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err
            .starts_with("error: `curl` is not installed, and `jacket dep` needs it\nnote: "),
        "{}",
        run.err
    );
    assert_eq!(project.toml(), before);
    // Without a policy, curl is not needed to get as far as the remote.
    let plain = Project::waived("[project]\nentry = [\"main.jai\"]\n");
    let run = plain.jacket_env(&["dep", "sync"], &[("PATH", only_git.to_str().unwrap())]);

    assert!(!run.err.contains("curl"), "{}", run.err);
}

fn released() -> (Project, dep_support::Origin) {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");
    let origin = dep_support::Origin::new(
        &project.root,
        "jaison",
        &[("module.jai", "answer :: 42;\n")],
    );

    origin.release("v1.0.0", &[]);
    (project, origin)
}

fn added() -> (Project, dep_support::Origin) {
    let (project, origin) = released();
    let run = project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    (project, origin)
}

#[test]
fn a_replaced_copy_is_not_touched_when_the_remote_cannot_be_read() {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");

    std::fs::create_dir_all(project.dir.join("modules/lib")).unwrap();
    std::fs::write(project.dir.join("modules/lib/module.jai"), "old :: 1;\n").unwrap();
    dep_support::git(&project.dir, &["add", "-A"]);
    dep_support::git(&project.dir, &["commit", "-q", "-m", "vendored"]);
    let run = project.jacket(&[
        "dep",
        "add",
        "http://127.0.0.1:1/someone/lib.git@v1",
        "--name",
        "lib",
    ]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(run.err.contains("could not reach"), "{}", run.err);
    assert!(project.dir.join("modules/lib/module.jai").is_file());
    assert_eq!(
        dep_support::git(&project.dir, &["status", "--porcelain"]),
        ""
    );
}

#[cfg(unix)]
#[test]
fn an_add_that_fails_after_the_clone_leaves_no_submodule_behind() {
    use std::os::unix::fs::PermissionsExt;
    let (project, origin) = released();
    let file = project.dir.join("jacket.toml");
    let mut mode = std::fs::metadata(&file).unwrap().permissions();

    mode.set_mode(0o444);
    std::fs::set_permissions(&file, mode).unwrap();
    let run = project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(run.err.contains("could not write"), "{}", run.err);
    assert_eq!(dep_support::git(&project.dir, &["submodule", "status"]), "");
    assert!(!project.dir.join(".gitmodules").exists());
    assert!(!project.dir.join("modules/jaison").exists());
    assert!(!project.dir.join(".git/modules/modules/jaison").exists());
    assert_eq!(
        dep_support::git(&project.dir, &["status", "--porcelain"]),
        ""
    );
}

#[test]
fn a_repository_git_cannot_read_is_not_blamed_on_a_missing_init() {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");

    std::fs::remove_dir_all(project.dir.join(".git")).unwrap();
    std::fs::write(project.dir.join(".git"), "garbage\n").unwrap();
    let run = project.jacket(&["dep", "sync"]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err
            .starts_with("error: git cannot use the repository at "),
        "{}",
        run.err
    );
    assert!(!run.err.contains("git init"), "{}", run.err);
}

#[test]
fn a_path_outside_the_project_is_refused_before_anything_runs() {
    let project = Project::waived(
        "[project]\nentry = [\"main.jai\"]\n\n[dependencies]\nlib = { git = \"a/b\", tag = \"v1\", path = \"../outside\" }\n",
    );
    let run = project.jacket(&["dep", "sync"]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err.contains("is not a folder inside the project"),
        "{}",
        run.err
    );
}

#[test]
fn an_entry_with_two_choices_is_refused() {
    let project = Project::waived(
        "[project]\nentry = [\"main.jai\"]\n\n[dependencies]\nlib = { git = \"a/b\", tag = \"v1\", rev = \"3f9c2a1\" }\n",
    );
    let run = project.jacket(&["dep", "list", "--offline"]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(run.err.contains("more than one"), "{}", run.err);
}

#[test]
fn a_hook_s_repository_variables_do_not_redirect_git() {
    let (project, _) = added();
    let elsewhere = project.root.join("elsewhere");

    std::fs::create_dir_all(&elsewhere).unwrap();
    let run = project.jacket_env(
        &["dep", "list", "--offline"],
        &[
            ("GIT_DIR", elsewhere.to_str().unwrap()),
            ("GIT_WORK_TREE", elsewhere.to_str().unwrap()),
            ("GIT_INDEX_FILE", elsewhere.join("index").to_str().unwrap()),
        ],
    );

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.out.contains("jaison"), "{}", run.out);
}

#[test]
fn a_relative_local_path_names_one_repository_for_every_command() {
    let (project, origin) = released();
    let name = origin
        .bare
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let run = project.jacket(&["dep", "add", &format!("../origins/{name}@v1.0.0")]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(project.dir.join("modules/jaison/module.jai").is_file());
    let toml = project.toml();

    assert!(toml.contains(&origin.url()), "{toml}");
    let run = project.jacket(&["dep", "check"]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
}

#[test]
fn non_ascii_names_keep_their_url_and_check_clean() {
    let (project, origin) = released();
    let run = project.jacket(&[
        "dep",
        "add",
        &format!("{}@v1.0.0", origin.url()),
        "--name",
        "caf\u{e9}",
    ]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let run = project.jacket(&["dep", "check", "--offline"]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(!run.err.contains("not a submodule"), "{}", run.err);
}

#[test]
fn a_removed_remote_is_reported_with_its_address_by_update_list_check_and_sync() {
    let (project, origin) = added();
    let url = origin.url();

    std::fs::remove_dir_all(&origin.bare).unwrap();
    let run = project.jacket(&["dep", "update"]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(run.err.contains(&url), "{}", run.err);
    assert!(run.err.contains("note: check the address"), "{}", run.err);
    let run = project.jacket(&["dep", "list"]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.out.contains("unreachable"), "{}", run.out);
    let run = project.jacket(&["dep", "check"]);

    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
    assert!(
        run.err.contains(&url) || run.out.contains(&url),
        "{}{}",
        run.out,
        run.err
    );
    // A fresh clone cannot fetch what is gone: the address in git's message is named.
    dep_support::git(&project.dir, &["add", "-A"]);
    dep_support::git(&project.dir, &["commit", "-q", "-m", "deps"]);
    let clone = project.root.join("clone");

    dep_support::git(
        &project.root,
        &[
            "clone",
            "-q",
            project.dir.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let run = dep_support::run_in(&clone, project.root.path(), &["dep", "sync"], &[]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(run.err.contains(&url), "{}", run.err);
}

#[cfg(unix)]
#[test]
fn a_refused_login_says_so_and_names_the_address() {
    let (project, _) = released();
    let ssh = project.root.join("fake-ssh");

    std::fs::write(
        &ssh,
        "#!/bin/sh\necho 'git@example.invalid: Permission denied (publickey).' >&2\nexit 255\n",
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
    let url = "git@example.invalid:someone/lib.git";
    let run = project.jacket_env(
        &["dep", "add", &format!("{url}@v1")],
        &[("GIT_SSH_COMMAND", ssh.to_str().unwrap())],
    );

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err.starts_with("error: git was not allowed to read "),
        "{}",
        run.err
    );
    assert!(run.err.contains(url), "{}", run.err);
    assert!(run.err.contains("SSH key"), "{}", run.err);
}

#[test]
fn listing_with_no_ssh_setting_anywhere_finishes() {
    let (project, _) = added();
    let run = project.jacket(&["dep", "list", "--offline"]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
}

#[cfg(unix)]
#[test]
fn ssh_from_the_projects_git_config_runs_without_prompting() {
    let (project, _) = released();
    let log = project.root.join("ssh-args");
    let ssh = project.root.join("logging-ssh");

    std::fs::write(
        &ssh,
        format!(
            "#!/bin/sh\necho \"$@\" > {}\necho 'denied: Permission denied (publickey).' >&2\nexit 255\n",
            log.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
    dep_support::git(
        &project.dir,
        &["config", "core.sshCommand", ssh.to_str().unwrap()],
    );
    let run = project.jacket(&["dep", "add", "git@example.invalid:a/b.git@v1"]);

    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    let args = std::fs::read_to_string(&log).unwrap();

    assert!(args.contains("BatchMode=yes"), "{args}");
}

#[test]
fn update_changes_nothing_when_a_later_dependency_cannot_be_reached() {
    let project = Project::waived("[project]\nentry = [\"main.jai\"]\n");
    let first = Origin::new(&project.root, "first", MODULE);

    first.release("v1.0.0", &[]);
    let second = Origin::new(&project.root, "second", MODULE);

    second.release("v1.0.0", &[]);

    for origin in [&first, &second] {
        let run = project.jacket(&["dep", "add", &format!("{}@v1.0.0", origin.url())]);

        assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    }

    first.release("v1.1.0", &[("module.jai", "answer :: 2;\n")]);
    std::fs::remove_dir_all(&second.bare).unwrap();
    let before = project.toml();
    let run = project.jacket(&["dep", "update"]);

    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(project.toml(), before, "{}", run.err);
    assert!(!run.out.contains("updated"), "{}", run.out);
    let sub = project.dir.join("modules/first");

    assert_eq!(
        std::fs::read_to_string(sub.join("module.jai")).unwrap(),
        "answer :: 42;\n"
    );
}

#[cfg(unix)]
#[test]
fn a_failed_clone_is_reported_even_when_looking_at_what_it_left_fails_too() {
    let (project, origin) = released();
    let path = dep_support::git_shim(
        project.root.path(),
        "case \"$*\" in\n\
         *submodule*add*) touch \"$SHIM_DIR/flag\"; echo 'fatal: injected clone failure' >&2; exit 128;;\n\
         *ls-files*) if [ -f \"$SHIM_DIR/flag\" ]; then echo 'fatal: injected listing failure' >&2; exit 128; fi;;\n\
         esac",
    );
    let run = project.jacket_env(
        &["dep", "add", &format!("{}@v1.0.0", origin.url())],
        &[("PATH", &path)],
    );

    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    let (first, rest) = run.err.split_once('\n').unwrap_or((&run.err, ""));

    assert!(
        first.contains("injected clone failure") || rest.contains("injected clone failure"),
        "{}",
        run.err
    );
    assert!(!first.contains("listing"), "{}", run.err);
}

#[cfg(unix)]
#[test]
fn a_replaced_vendored_copy_is_named_when_the_add_fails() {
    let (project, origin) = released();

    std::fs::create_dir_all(project.dir.join("modules/jaison")).unwrap();
    std::fs::write(project.dir.join("modules/jaison/module.jai"), "old :: 1;\n").unwrap();
    dep_support::git(&project.dir, &["add", "-A"]);
    dep_support::git(&project.dir, &["commit", "-q", "-m", "vendored"]);
    let path = dep_support::git_shim(
        project.root.path(),
        "case \"$*\" in *submodule*add*) echo 'fatal: injected' >&2; exit 128;; esac",
    );
    let run = project.jacket_env(
        &[
            "dep",
            "add",
            &format!("{}@v1.0.0", origin.url()),
            "--name",
            "jaison",
        ],
        &[("PATH", &path)],
    );

    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.err.contains("injected"), "{}", run.err);
    assert!(run.err.contains("git checkout HEAD --"), "{}", run.err);
}

#[cfg(unix)]
#[test]
fn a_remove_that_fails_leaves_the_entry_and_the_submodule() {
    let (project, _) = added();
    let before = (
        project.toml(),
        dep_support::git(&project.dir, &["ls-files", "-s"]),
    );
    let path = dep_support::git_shim(
        project.root.path(),
        "case \"$*\" in *deinit*) echo 'fatal: injected' >&2; exit 128;; esac",
    );
    let run = project.jacket_env(&["dep", "remove", "jaison"], &[("PATH", &path)]);

    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.err.contains("injected"), "{}", run.err);
    assert!(!run.out.contains("removed"), "{}", run.out);
    assert_eq!(
        (
            project.toml(),
            dep_support::git(&project.dir, &["ls-files", "-s"])
        ),
        before
    );
}
